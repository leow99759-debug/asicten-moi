//! Executor (SPEC §4.4): fills slots and `%VARS%`, runs actions in order through a
//! [`Backend`] with a timeout per action, stops at the first failure.
//! `jarvis-win` provides the real backend; [`DryRun`] records actions for tests.

use std::collections::BTreeMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use ts_rs::TS;

use crate::commands::{expand, Action, AppLocator, AssistantMode, Command, Num};
use crate::nlu::matcher::SlotValue;
use crate::scheduler::Job;

/// Performs one side effect. `Ok(Some(text))` = something to say (time, clipboard…).
/// Pauses are handled by the executor itself.
pub trait Backend: Send + Sync {
    fn perform(&self, action: &Action) -> Result<Option<String>, String>;
}

/// Assistant-level actions (Speak, Ask, Timer, Reminder, Assistant.*), implemented by the app.
pub trait Assistant: Send + Sync {
    fn speak(&self, text: Option<&str>, clip: Option<&str>) -> Result<(), String>;
    /// Voice/UI yes-no question; `false` stops the command.
    fn ask(&self, question: &str) -> Result<bool, String>;
    fn set_mode(&self, mode: AssistantMode, on: bool) -> Result<(), String>;
    fn set_theme(&self, color: &str) -> Result<(), String>;
    fn open_page(&self, page: &str) -> Result<(), String>;
    fn repeat(&self) -> Result<(), String>;
    fn cancel(&self) -> Result<(), String>;
    fn schedule(&self, after: Duration, job: Job) -> Result<(), String>;
}

/// Records actions instead of touching the machine.
#[derive(Default)]
pub struct DryRun {
    pub log: Mutex<Vec<Action>>,
}

impl DryRun {
    pub fn actions(&self) -> Vec<Action> {
        self.log.lock().map(|l| l.clone()).unwrap_or_default()
    }

    fn record(&self, a: Action) -> Result<(), String> {
        if let Ok(mut l) = self.log.lock() {
            l.push(a);
        }
        Ok(())
    }
}

impl Assistant for DryRun {
    fn speak(&self, text: Option<&str>, clip: Option<&str>) -> Result<(), String> {
        self.record(Action::Speak {
            text: text.map(Into::into),
            clip: clip.map(Into::into),
        })
    }
    fn ask(&self, question: &str) -> Result<bool, String> {
        self.record(Action::Ask {
            question: question.into(),
        })?;
        Ok(!question.contains("[нет]"))
    }
    fn set_mode(&self, mode: AssistantMode, on: bool) -> Result<(), String> {
        self.record(Action::AssistantSetMode { mode, on })
    }
    fn set_theme(&self, color: &str) -> Result<(), String> {
        self.record(Action::AssistantSetTheme {
            color: color.into(),
        })
    }
    fn open_page(&self, page: &str) -> Result<(), String> {
        self.record(Action::AssistantOpenPage { page: page.into() })
    }
    fn repeat(&self) -> Result<(), String> {
        self.record(Action::AssistantRepeat)
    }
    fn cancel(&self) -> Result<(), String> {
        self.record(Action::AssistantCancel)
    }
    fn schedule(&self, after: Duration, job: Job) -> Result<(), String> {
        let sec = Num::Value(after.as_secs_f64());
        self.record(match job {
            Job::RunCommand(id) => Action::Timer {
                sec,
                then_command: id,
            },
            Job::Remind(text) => Action::Reminder { sec, text },
        })
    }
}

impl Backend for DryRun {
    fn perform(&self, action: &Action) -> Result<Option<String>, String> {
        if let Ok(mut l) = self.log.lock() {
            l.push(action.clone());
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct StepResult {
    /// Action type, e.g. `Launch.File`.
    pub action: String,
    pub ok: bool,
    pub error: Option<String>,
    /// Text to speak/show (System.Info, clipboard…).
    pub output: Option<String>,
}

pub struct Executor {
    backend: Arc<dyn Backend>,
    assistant: Arc<dyn Assistant>,
    apps: Arc<dyn AppLocator + Send + Sync>,
    timeout: Duration,
    /// Real sleeps for `Lux.Pause*` (off in tests).
    pub pauses: bool,
}

fn type_name(a: &Action) -> String {
    serde_json::to_value(a)
        .ok()
        .and_then(|v| v.get("type").and_then(Value::as_str).map(str::to_owned))
        .unwrap_or_default()
}

fn num(n: &Num) -> Option<f64> {
    match n {
        Num::Value(v) => Some(*v),
        Num::Slot(_) => None,
    }
}

impl Executor {
    pub fn new(
        backend: Arc<dyn Backend>,
        assistant: Arc<dyn Assistant>,
        apps: Arc<dyn AppLocator + Send + Sync>,
    ) -> Self {
        Self {
            backend,
            assistant,
            apps,
            timeout: Duration::from_secs(10),
            pauses: true,
        }
    }

    /// Run a command's actions with matched slot values.
    pub fn run(&self, cmd: &Command, slots: &BTreeMap<String, SlotValue>) -> Vec<StepResult> {
        let mut out = Vec::new();
        for raw in &cmd.actions {
            let result = self.resolve(raw, slots).and_then(|a| self.step(&a));
            let ok = result.is_ok();
            let (output, error) = match result {
                Ok(o) => (o, None),
                Err(e) => (None, Some(e)),
            };
            out.push(StepResult {
                action: type_name(raw),
                ok,
                error,
                output,
            });
            if !ok {
                break;
            }
        }
        out
    }

    /// Substitute `{slot}` placeholders and expand `%VARS%` in string params.
    pub fn resolve(
        &self,
        action: &Action,
        slots: &BTreeMap<String, SlotValue>,
    ) -> Result<Action, String> {
        let mut v = serde_json::to_value(action).map_err(|e| e.to_string())?;
        if let Value::Object(map) = &mut v {
            for (k, val) in map.iter_mut() {
                if k == "type" {
                    continue;
                }
                if let Value::String(s) = val {
                    *val = fill(s, slots, self.apps.as_ref());
                }
            }
        }
        serde_json::from_value(v).map_err(|e| format!("bad params: {e}"))
    }

    fn step(&self, action: &Action) -> Result<Option<String>, String> {
        let pause = match action {
            Action::PauseMs { ms } => Some(num(ms).ok_or("pause without value")? / 1000.0),
            Action::PauseSec { s } => Some(num(s).ok_or("pause without value")?),
            _ => None,
        };
        if let Some(secs) = pause {
            if self.pauses {
                std::thread::sleep(Duration::from_secs_f64(secs.clamp(0.0, 600.0)));
            }
            return Ok(None);
        }
        let a = &self.assistant;
        let secs = |n: &Num| num(n).map(|s| Duration::from_secs_f64(s.clamp(0.0, 31_536_000.0)));
        let assistant = match action {
            Action::Speak { text, clip } => Some(a.speak(text.as_deref(), clip.as_deref())),
            Action::Ask { question } => Some(match a.ask(question) {
                Ok(true) => Ok(()),
                Ok(false) => Err("отменено".to_owned()),
                Err(e) => Err(e),
            }),
            Action::AssistantSetMode { mode, on } => Some(a.set_mode(*mode, *on)),
            Action::AssistantSetTheme { color } => Some(a.set_theme(color)),
            Action::AssistantOpenPage { page } => Some(a.open_page(page)),
            Action::AssistantRepeat => Some(a.repeat()),
            Action::AssistantCancel => Some(a.cancel()),
            Action::Timer { sec, then_command } => Some(
                secs(sec)
                    .ok_or_else(|| "таймер без времени".to_owned())
                    .and_then(|d| a.schedule(d, Job::RunCommand(then_command.clone()))),
            ),
            Action::Reminder { sec, text } => Some(
                secs(sec)
                    .ok_or_else(|| "напоминание без времени".to_owned())
                    .and_then(|d| a.schedule(d, Job::Remind(text.clone()))),
            ),
            _ => None,
        };
        if let Some(r) = assistant {
            return r.map(|()| None);
        }
        let (tx, rx) = mpsc::channel();
        let backend = self.backend.clone();
        let a = action.clone();
        std::thread::spawn(move || {
            let _ = tx.send(backend.perform(&a));
        });
        rx.recv_timeout(self.timeout)
            .unwrap_or_else(|_| Err(format!("timeout {} s", self.timeout.as_secs())))
    }
}

/// A whole-string slot becomes a typed JSON value (number for `{число}`/`{время}`);
/// slots inside longer strings are substituted as text. Then `%VARS%` are expanded.
fn fill(s: &str, slots: &BTreeMap<String, SlotValue>, apps: &dyn AppLocator) -> Value {
    if let Some(v) = slots.get(s.trim()) {
        return match v {
            SlotValue::Number(n) | SlotValue::Duration(n) => serde_json::json!(n),
            SlotValue::Text(t) => Value::String(t.clone()),
        };
    }
    let mut text = s.to_owned();
    for (k, v) in slots {
        let rep = match v {
            SlotValue::Number(n) | SlotValue::Duration(n) => format!("{n}"),
            SlotValue::Text(t) => t.clone(),
        };
        text = text.replace(k, &rep);
    }
    Value::String(expand(&text, apps))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::parse_pack;

    struct Apps;
    impl AppLocator for Apps {
        fn locate(&self, exe: &str) -> Option<String> {
            (exe == "chrome.exe").then(|| r"C:\Chrome\chrome.exe".into())
        }
    }

    struct Failing;
    impl Backend for Failing {
        fn perform(&self, a: &Action) -> Result<Option<String>, String> {
            match a {
                Action::ProcessKill { .. } => Err("no such process".into()),
                Action::LaunchUwp { .. } => {
                    std::thread::sleep(Duration::from_millis(300));
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
    }

    fn cmd(actions: &str, phrase: &str) -> Command {
        let json = format!(
            r#"{{"id":"t","name":"T","commands":[{{"id":"c","name":"C","phrases":["{phrase}"],"actions":[{actions}]}}]}}"#
        );
        parse_pack(&json).expect("pack").0.commands.remove(0)
    }

    fn exec(b: Arc<dyn Backend>) -> Executor {
        let mut e = Executor::new(b, Arc::new(DryRun::default()), Arc::new(Apps));
        e.pauses = false;
        e
    }

    #[test]
    fn fills_slots_and_vars_in_dry_run() {
        let dry = Arc::new(DryRun::default());
        let e = exec(dry.clone());
        let c = cmd(
            r#"{"type":"Launch.File","path":"%CHROME%","args":"https://ya.ru/search?text={текст}"},
               {"type":"Lux.PauseMS","ms":300},
               {"type":"System.VolumeSet","level":"{число}"}"#,
            "найди {текст} {число}",
        );
        let slots = BTreeMap::from([
            ("{текст}".to_owned(), SlotValue::Text("котики".into())),
            ("{число}".to_owned(), SlotValue::Number(50.0)),
        ]);
        let res = e.run(&c, &slots);
        assert!(res.iter().all(|r| r.ok), "{res:?}");
        assert_eq!(res.len(), 3);
        assert_eq!(
            dry.actions(),
            vec![
                Action::LaunchFile {
                    path: r"C:\Chrome\chrome.exe".into(),
                    args: "https://ya.ru/search?text=котики".into(),
                    workdir: None,
                    admin: false
                },
                Action::VolumeSet {
                    level: Num::Value(50.0)
                },
            ]
        );
    }

    #[test]
    fn stops_on_error_and_times_out() {
        let mut e = exec(Arc::new(Failing));
        let c = cmd(
            r#"{"type":"Launch.Url","url":"https://ya.ru"},{"type":"Process.Kill","name":"x.exe"},{"type":"Window.Close"}"#,
            "x",
        );
        let res = e.run(&c, &BTreeMap::new());
        assert_eq!(res.len(), 2);
        assert!(res[0].ok);
        assert_eq!(res[1].action, "Process.Kill");
        assert_eq!(res[1].error.as_deref(), Some("no such process"));

        e.timeout = Duration::from_millis(50);
        let c = cmd(r#"{"type":"Launch.Uwp","aumid":"a!b"}"#, "x");
        let res = e.run(&c, &BTreeMap::new());
        assert!(res[0]
            .error
            .as_deref()
            .is_some_and(|m| m.starts_with("timeout")));
    }

    #[test]
    fn assistant_actions_and_ask_cancel() {
        let dry = Arc::new(DryRun::default());
        let mut e = Executor::new(dry.clone(), dry.clone(), Arc::new(Apps));
        e.pauses = false;
        let c = cmd(
            r#"{"type":"Speak","clip":"ok"},
               {"type":"Reminder","sec":"{время}","text":"проверить духовку"},
               {"type":"Timer","sec":5,"then_command":"music"},
               {"type":"Assistant.Mode","mode":"silent","on":true},
               {"type":"Ask","question":"Точно? [нет]"},
               {"type":"Window.Close"}"#,
            "напомни через {время}",
        );
        let slots = BTreeMap::from([("{время}".to_owned(), SlotValue::Duration(600.0))]);
        let res = e.run(&c, &slots);
        assert_eq!(res.len(), 5, "stops after declined Ask: {res:?}");
        assert_eq!(res[4].error.as_deref(), Some("отменено"));
        assert_eq!(
            dry.actions()[1],
            Action::Reminder {
                sec: Num::Value(600.0),
                text: "проверить духовку".into()
            }
        );
        assert!(!dry.actions().contains(&Action::WindowClose));
    }

    #[test]
    fn unfilled_slot_passes_through() {
        let e = exec(Arc::new(DryRun::default()));
        let c = cmd(
            r#"{"type":"System.VolumeSet","level":"{число}"}"#,
            "громкость на {число}",
        );
        let res = e.run(&c, &BTreeMap::new());
        assert!(
            res[0].ok,
            "slot string stays a Slot value; backend decides: {res:?}"
        );
    }
}
