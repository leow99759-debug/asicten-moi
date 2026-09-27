//! Command worker thread: utterances → Brain (plan, confirm, execute) → outcome events.
//! Also implements the executor's `Assistant` hooks (confirm dialog, modes, timers, UI).

use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jarvis_core::brain::Brain;
use jarvis_core::commands::{self, AssistantMode, Pack};
use jarvis_core::executor::{Assistant, Executor};
use jarvis_core::ipc::{ConfirmRequest, CoreEvent, EventSink, UiCommand};
use jarvis_core::modes::ModeCommand;
use jarvis_core::scheduler::{Job, Scheduler};
use jarvis_win::apps::SystemApps;
use jarvis_win::backend::WinBackend;

use crate::engine::{self, Engine};

/// Auto-cancel for confirmations (§4.6).
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(30);

pub enum Work {
    Utterance(String),
    Job(Job),
    Repeat,
}

/// Pending confirmation shared by the worker (asks), the voice loop and the UI (answer).
#[derive(Default)]
pub struct Confirm {
    pending: Mutex<Option<Sender<bool>>>,
}

impl Confirm {
    pub fn is_pending(&self) -> bool {
        self.pending.lock().map(|p| p.is_some()).unwrap_or(false)
    }

    /// Deliver an answer; false if nothing was pending.
    pub fn answer(&self, yes: bool) -> bool {
        let tx = self.pending.lock().ok().and_then(|mut p| p.take());
        tx.is_some_and(|tx| tx.send(yes).is_ok())
    }

    fn ask(&self, sink: &dyn EventSink, question: &str) -> bool {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut p) = self.pending.lock() {
            *p = Some(tx);
        }
        sink.emit(CoreEvent::Confirm(ConfirmRequest {
            question: question.to_owned(),
            timeout_sec: CONFIRM_TIMEOUT.as_secs() as u32,
        }));
        sink.emit(CoreEvent::Say(question.to_owned()));
        let yes = rx.recv_timeout(CONFIRM_TIMEOUT).unwrap_or(false);
        if let Ok(mut p) = self.pending.lock() {
            *p = None;
        }
        sink.emit(CoreEvent::ConfirmClosed);
        yes
    }
}

struct AppAssistant {
    sink: Arc<dyn EventSink>,
    confirm: Arc<Confirm>,
    engine: Engine,
    work: Sender<Work>,
    scheduler: Mutex<Scheduler>,
}

impl Assistant for AppAssistant {
    fn speak(&self, text: Option<&str>, clip: Option<&str>) -> Result<(), String> {
        // voice output arrives in M3; until then the UI shows the line
        let line = text.or(clip).unwrap_or_default();
        self.sink.emit(CoreEvent::Say(line.to_owned()));
        Ok(())
    }

    fn ask(&self, question: &str) -> Result<bool, String> {
        Ok(self.confirm.ask(&*self.sink, question))
    }

    fn set_mode(&self, mode: AssistantMode, on: bool) -> Result<(), String> {
        let cmd = match (mode, on) {
            (AssistantMode::Prefix, true) => ModeCommand::PrefixOn,
            (AssistantMode::Prefix, false) => ModeCommand::PrefixOff,
            (AssistantMode::Silent, true) => ModeCommand::SilentOn,
            (AssistantMode::Silent, false) => ModeCommand::SilentOff,
            (AssistantMode::MicOff, true) => ModeCommand::MicOff,
            (AssistantMode::MicOff, false) => ModeCommand::MicOn,
        };
        self.engine.send(engine::Msg::Mode(cmd));
        Ok(())
    }

    fn set_theme(&self, color: &str) -> Result<(), String> {
        self.sink
            .emit(CoreEvent::Ui(UiCommand::SetTheme(color.to_owned())));
        Ok(())
    }

    fn open_page(&self, page: &str) -> Result<(), String> {
        self.sink
            .emit(CoreEvent::Ui(UiCommand::OpenPage(page.to_owned())));
        Ok(())
    }

    fn repeat(&self) -> Result<(), String> {
        self.work.send(Work::Repeat).map_err(|e| e.to_string())
    }

    fn cancel(&self) -> Result<(), String> {
        self.confirm.answer(false);
        if let Ok(s) = self.scheduler.lock() {
            s.cancel_all();
        }
        Ok(())
    }

    fn schedule(&self, after: Duration, job: Job) -> Result<(), String> {
        self.scheduler
            .lock()
            .map(|mut s| {
                s.add(after, job);
            })
            .map_err(|e| e.to_string())
    }
}

/// Built-in packs + the user's commands.json.
pub fn load_commands(packs_dir: Option<&Path>, user_file: &Path) -> Vec<commands::Command> {
    let mut packs: Vec<Pack> = packs_dir
        .map(|d| commands::load_dir(d).0)
        .unwrap_or_default();
    if let Ok(text) = std::fs::read_to_string(user_file) {
        match commands::parse_pack(&text) {
            Ok((p, errs)) => {
                for e in errs {
                    tracing::warn!("user commands: {e}");
                }
                packs.push(p);
            }
            Err(e) => tracing::warn!("user commands: {e}"),
        }
    }
    let cmds: Vec<_> = packs.into_iter().flat_map(|p| p.commands).collect();
    tracing::info!(count = cmds.len(), "commands loaded");
    cmds
}

pub fn spawn(
    commands: Vec<commands::Command>,
    sink: Arc<dyn EventSink>,
    confirm: Arc<Confirm>,
    engine: Engine,
    work_tx: Sender<Work>,
    work_rx: Receiver<Work>,
) {
    let (scheduler, fired) = Scheduler::start();
    let fwd = work_tx.clone();
    std::thread::spawn(move || {
        for job in fired {
            if fwd.send(Work::Job(job)).is_err() {
                break;
            }
        }
    });
    let assistant = Arc::new(AppAssistant {
        sink: sink.clone(),
        confirm,
        engine,
        work: work_tx,
        scheduler: Mutex::new(scheduler),
    });
    let spawned = std::thread::Builder::new()
        .name("jarvis-brain".into())
        .spawn(move || {
            let executor = Executor::new(
                Arc::new(WinBackend),
                assistant.clone(),
                Arc::new(SystemApps),
            );
            let brain = Brain::new(commands, executor, assistant);
            let mut last: Option<String> = None;
            for work in work_rx {
                match work {
                    Work::Utterance(text) => {
                        let outcome = brain.handle(&text);
                        if outcome.commands.is_empty() {
                            sink.emit(CoreEvent::Say("Простите, сэр, не понял команду".into()));
                        } else {
                            last = Some(text);
                        }
                        sink.emit(CoreEvent::Outcome(outcome));
                    }
                    Work::Repeat => {
                        if let Some(t) = last.clone() {
                            sink.emit(CoreEvent::Outcome(brain.handle(&t)));
                        }
                    }
                    Work::Job(Job::RunCommand(id)) => match brain.run_by_id(&id) {
                        Some(o) => sink.emit(CoreEvent::Outcome(jarvis_core::brain::Outcome {
                            phrase: format!("таймер: {}", o.name),
                            commands: vec![o],
                        })),
                        None => tracing::warn!(%id, "timer: unknown command"),
                    },
                    Work::Job(Job::Remind(text)) => {
                        sink.emit(CoreEvent::Say(format!("Сэр, напоминаю: {text}")));
                    }
                }
            }
        });
    if let Err(e) = spawned {
        tracing::error!(%e, "brain thread spawn failed");
    }
}
