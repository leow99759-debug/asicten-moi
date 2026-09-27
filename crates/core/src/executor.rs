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

use crate::commands::{expand, Action, AppLocator, Command, Num};
use crate::nlu::matcher::SlotValue;

/// Performs one side effect. `Ok(Some(text))` = something to say (time, clipboard…).
/// Pauses are handled by the executor itself.
pub trait Backend: Send + Sync {
    fn perform(&self, action: &Action) -> Result<Option<String>, String>;
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
    pub fn new(backend: Arc<dyn Backend>, apps: Arc<dyn AppLocator + Send + Sync>) -> Self {
        Self {
            backend,
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
        let mut e = Executor::new(b, Arc::new(Apps));
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
