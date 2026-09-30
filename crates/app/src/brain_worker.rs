//! Command worker thread: utterances → Brain (plan, confirm, execute) → outcome events.
//! Also implements the executor's `Assistant` hooks (confirm dialog, modes, timers, UI).

use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use jarvis_core::brain::{Brain, Outcome};
use jarvis_core::commands::{self, AssistantMode, Pack};
use jarvis_core::executor::{Assistant, Executor};
use jarvis_core::ipc::{ConfirmRequest, CoreEvent, EventSink, UiCommand};
use jarvis_core::modes::ModeCommand;
use jarvis_core::scheduler::{Job, Scheduler};
use jarvis_core::{Config, Db};
use jarvis_win::apps::SystemApps;
use jarvis_win::backend::WinBackend;

use crate::engine::{self, Engine};
use crate::speaker::Speaker;

/// Auto-cancel for confirmations (§4.6).
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(30);

pub enum Work {
    /// Phrase + heard without the wake word (unknown ones are then dropped silently).
    Utterance(String, bool),
    Job(Job),
    Repeat,
    /// Editor saved: swap the command set.
    Reload(Vec<commands::Command>),
    /// Editor «▶ Тест»: unsaved command + sample phrase for its slots.
    Test(Box<commands::Command>, String),
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

    fn ask(&self, sink: &dyn EventSink, speaker: &Speaker, question: &str) -> bool {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut p) = self.pending.lock() {
            *p = Some(tx);
        }
        sink.emit(CoreEvent::Confirm(ConfirmRequest {
            question: question.to_owned(),
            timeout_sec: CONFIRM_TIMEOUT.as_secs() as u32,
        }));
        // pre-recorded question when the pack has it, else the generic «подтвердите»
        speaker.say_text(&["confirm"], question);
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
    speaker: Arc<Speaker>,
    confirm: Arc<Confirm>,
    engine: Engine,
    work: Sender<Work>,
    scheduler: Mutex<Scheduler>,
    quit: Quit,
}

/// Exits the app (tray icon cleaned up by Tauri).
pub type Quit = Arc<dyn Fn() + Send + Sync>;

impl Assistant for AppAssistant {
    fn speak(&self, text: Option<&str>, clip: Option<&str>) -> Result<(), String> {
        self.speaker.say(&jarvis_core::brain::Line {
            clips: clip.map(|c| vec![c.to_owned()]).unwrap_or_default(),
            text: text.map(Into::into),
        });
        Ok(())
    }

    fn ask(&self, question: &str) -> Result<bool, String> {
        Ok(self.confirm.ask(&*self.sink, &self.speaker, question))
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
        self.speaker.stop();
        self.confirm.answer(false);
        if let Ok(s) = self.scheduler.lock() {
            s.cancel_all();
        }
        Ok(())
    }

    fn quit(&self) -> Result<(), String> {
        let (speaker, quit) = (self.speaker.clone(), self.quit.clone());
        // the goodbye line is queued after the actions run: let it play first
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            let until = std::time::Instant::now() + Duration::from_secs(5);
            while speaker.is_playing() && std::time::Instant::now() < until {
                std::thread::sleep(Duration::from_millis(100));
            }
            quit();
        });
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

/// Default packs + installed add-ons + the user's commands.json (overrides by id,
/// switched-off dropped).
pub fn load_commands(
    packs_dir: Option<&Path>,
    installed: &[String],
    user_file: &Path,
) -> Vec<commands::Command> {
    let packs: Vec<Pack> = packs_dir
        .map(|d| commands::builtin_packs(d, installed))
        .unwrap_or_default();
    let cmds = commands::Library::merge(packs, commands::read_user(user_file)).active();
    tracing::info!(count = cmds.len(), "commands loaded");
    cmds
}

/// Everything the worker talks to.
pub struct Deps {
    pub sink: Arc<dyn EventSink>,
    pub speaker: Arc<Speaker>,
    pub db: Arc<Mutex<Db>>,
    pub confirm: Arc<Confirm>,
    pub engine: Engine,
    pub quit: Quit,
    pub config: Arc<Mutex<Config>>,
}

pub fn spawn(
    commands: Vec<commands::Command>,
    deps: Deps,
    work_tx: Sender<Work>,
    work_rx: Receiver<Work>,
) {
    let Deps {
        sink,
        speaker,
        db,
        confirm,
        engine,
        quit,
        config,
    } = deps;
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
        speaker: speaker.clone(),
        confirm,
        engine,
        work: work_tx,
        scheduler: Mutex::new(scheduler),
        quit,
    });
    let spawned = std::thread::Builder::new()
        .name("jarvis-brain".into())
        .spawn(move || {
            let executor = || {
                Executor::new(
                    Arc::new(WinBackend {
                        config: Some(config.clone()),
                    }),
                    assistant.clone(),
                    Arc::new(SystemApps),
                )
            };
            let mut brain = Brain::new(commands, executor(), assistant.clone());
            let mut last: Option<String> = None;
            let publish = |outcome: Outcome| {
                let entry = db
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .record(&outcome, now_ms());
                match entry {
                    Ok(e) => sink.emit(CoreEvent::History(e)),
                    Err(e) => tracing::warn!("history: {e:#}"),
                }
                for line in outcome.voice_lines() {
                    speaker.say(&line);
                }
                sink.emit(CoreEvent::Outcome(outcome));
            };
            for work in work_rx {
                match work {
                    Work::Utterance(text, unprompted) => {
                        let outcome = brain.handle(&text);
                        // not understood + no «Джарвис» or a long sentence = TV/room talk:
                        // a «не понял» reply would reopen the follow-up window and loop
                        if outcome.commands.is_empty()
                            && (unprompted || text.split_whitespace().count() > 6)
                        {
                            tracing::info!(%text, "ignored: not a command");
                            continue;
                        }
                        if !outcome.commands.is_empty() {
                            last = Some(text);
                        }
                        publish(outcome);
                    }
                    Work::Repeat => {
                        if let Some(t) = last.clone() {
                            publish(brain.handle(&t));
                        }
                    }
                    Work::Job(Job::RunCommand(id)) => match brain.run_by_id(&id) {
                        Some(o) => publish(Outcome {
                            phrase: format!("таймер: {}", o.name),
                            commands: vec![o],
                        }),
                        None => tracing::warn!(%id, "timer: unknown command"),
                    },
                    Work::Test(cmd, sample) => {
                        let o = brain.test(&cmd, &sample);
                        publish(Outcome {
                            phrase: format!("тест: {}", o.name),
                            commands: vec![o],
                        });
                    }
                    Work::Reload(cmds) => brain = Brain::new(cmds, executor(), assistant.clone()),
                    Work::Job(Job::RunPhrase(text)) => {
                        let mut o = brain.handle(&text);
                        o.phrase = format!("таймер: {text}");
                        publish(o);
                    }
                    Work::Job(Job::Remind(text)) => {
                        // «Сэр, напоминаю» clip, then the reminder itself (online voice / Piper)
                        speaker.say_text(&["remind"], "Сэр, напоминаю");
                        if !text.trim().is_empty() {
                            speaker.say_text(&[], &text);
                        }
                    }
                }
            }
        });
    if let Err(e) = spawned {
        tracing::error!(%e, "brain thread spawn failed");
    }
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}
