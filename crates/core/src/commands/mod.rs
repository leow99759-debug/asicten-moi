//! Command model, packs, loading and validation (SPEC §4.1).

mod action;
mod pathvars;

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use action::{Action, AssistantMode, Num, PowerPlan, Side};
pub use pathvars::{expand, AppLocator, APP_VARS};

/// Slot placeholders allowed in phrases (§4.1).
pub const SLOTS: [&str; 4] = ["{число}", "{время}", "{текст}", "{приложение}"];

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Reply {
    /// Voice pack categories (§6.1), random pick: `ok`, `done`, `loading`…
    #[serde(default)]
    pub clips: Vec<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct When {
    /// Foreground process exe, e.g. `POWERPNT.EXE` (§4.5).
    #[serde(default)]
    pub foreground: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Command {
    pub id: String,
    #[serde(default)]
    pub folder: Vec<String>,
    pub name: String,
    /// At least one must be said.
    pub phrases: Vec<String>,
    /// May or may not be said.
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(default)]
    pub actions: Vec<Action>,
    #[serde(default)]
    pub reply: Reply,
    /// Ask before running (§4.6); dangerous actions force it anyway.
    #[serde(default)]
    pub confirm: bool,
    /// May take part in «… и …» chains (§4.3).
    #[serde(default = "yes")]
    pub chainable: bool,
    #[serde(default)]
    pub slots: BTreeMap<String, String>,
    #[serde(default)]
    pub when: Option<When>,
}

impl Command {
    pub fn needs_confirm(&self) -> bool {
        self.confirm || self.actions.iter().any(Action::needs_confirm)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Pack {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub version: String,
    pub commands: Vec<Command>,
}

/// Problems in one command; the command is skipped, the rest of the pack loads.
fn validate(cmd: &Command) -> Vec<String> {
    let mut errs = Vec::new();
    if cmd.id.trim().is_empty() {
        errs.push("empty id".into());
    }
    if cmd.phrases.iter().all(|p| p.trim().is_empty()) {
        errs.push("no phrases".into());
    }
    if cmd.actions.is_empty() && cmd.reply.clips.is_empty() && cmd.reply.text.is_none() {
        errs.push("no actions and no reply".into());
    }
    let mut used = HashSet::new();
    for p in &cmd.phrases {
        for slot in placeholders(p) {
            if !SLOTS.contains(&slot.as_str()) {
                errs.push(format!("unknown slot {slot} in «{p}»"));
            }
            used.insert(slot);
        }
    }
    for a in &cmd.actions {
        for s in a.params().iter().flat_map(|p| placeholders(p)) {
            if !used.contains(&s) {
                errs.push(format!("action uses {s} that no phrase provides"));
            }
        }
    }
    errs
}

/// `{...}` tokens in a string.
fn placeholders(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(start) = rest.find('{') {
        let Some(len) = rest[start..].find('}') else {
            break;
        };
        out.push(rest[start..=start + len].to_owned());
        rest = &rest[start + len + 1..];
    }
    out
}

/// Parse a pack, dropping invalid commands (reported in the second value).
pub fn parse_pack(json: &str) -> crate::Result<(Pack, Vec<String>)> {
    let mut pack: Pack = serde_json::from_str(json)?;
    let mut report = Vec::new();
    let mut ids = HashSet::new();
    pack.commands.retain(|c| {
        let mut errs = validate(c);
        if !ids.insert(c.id.clone()) {
            errs.push("duplicate id".into());
        }
        for e in &errs {
            report.push(format!("{}: {e}", c.id));
        }
        errs.is_empty()
    });
    Ok((pack, report))
}

/// Load every `*.json` pack in a folder (sorted); broken files are reported, not fatal.
pub fn load_dir(dir: &Path) -> (Vec<Pack>, Vec<String>) {
    let mut packs = Vec::new();
    let mut report = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    for f in files {
        let name = f.display().to_string();
        match std::fs::read_to_string(&f)
            .map_err(crate::Error::from)
            .and_then(|t| parse_pack(&t))
        {
            Ok((pack, errs)) => {
                report.extend(errs.into_iter().map(|e| format!("{name}: {e}")));
                packs.push(pack);
            }
            Err(e) => report.push(format!("{name}: {e}")),
        }
    }
    for r in &report {
        tracing::warn!("pack: {r}");
    }
    (packs, report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_EXAMPLE: &str = r#"{
      "id": "chrome.open",
      "folder": ["Управление Chrome"],
      "name": "Открыть браузер",
      "phrases": ["браузер", "хром"],
      "optional": ["открой", "запусти", "включи", "пожалуйста", "ну-ка", "давай"],
      "actions": [
        {"type": "Launch.File", "path": "%CHROME%", "args": ""},
        {"type": "Lux.PauseMS", "ms": 300}
      ],
      "reply": {"clips": ["ok"], "text": "Да, сэр"},
      "confirm": false,
      "chainable": true,
      "slots": {}
    }"#;

    fn pack(cmds: &str) -> String {
        format!(r#"{{"id":"t","name":"T","commands":[{cmds}]}}"#)
    }

    #[test]
    fn parses_spec_example_and_roundtrips() {
        let (p, errs) = parse_pack(&pack(SPEC_EXAMPLE)).expect("parse");
        assert!(errs.is_empty(), "{errs:?}");
        let c = &p.commands[0];
        assert_eq!(
            c.actions[0],
            Action::LaunchFile {
                path: "%CHROME%".into(),
                args: String::new(),
                workdir: None,
                admin: false
            }
        );
        assert_eq!(
            c.actions[1],
            Action::PauseMs {
                ms: Num::Value(300.0)
            }
        );
        let back: Command =
            serde_json::from_str(&serde_json::to_string(c).expect("ser")).expect("de");
        assert_eq!(&back, c);
    }

    #[test]
    fn slots_and_confirm() {
        let json = pack(
            r#"{"id":"vol","name":"Громкость","phrases":["громкость на {число}"],
                "actions":[{"type":"System.VolumeSet","level":"{число}"}]},
               {"id":"off","name":"Выключить","phrases":["выключи компьютер через {время}"],
                "actions":[{"type":"System.Shutdown","delay_sec":"{время}"}]}"#,
        );
        let (p, errs) = parse_pack(&json).expect("parse");
        assert!(errs.is_empty(), "{errs:?}");
        assert!(!p.commands[0].needs_confirm());
        assert!(p.commands[1].needs_confirm());
    }

    #[test]
    fn invalid_commands_are_dropped_with_report() {
        let json = pack(
            r#"{"id":"a","name":"A","phrases":[],"actions":[{"type":"Window.Close"}]},
               {"id":"b","name":"B","phrases":["x {цвет}"],"actions":[{"type":"Window.Close"}]},
               {"id":"c","name":"C","phrases":["x"],"actions":[{"type":"Keys.Type","text":"{текст}"}]},
               {"id":"d","name":"D","phrases":["x"]},
               {"id":"e","name":"E","phrases":["ок"],"reply":{"clips":["ok"]}},
               {"id":"e","name":"E2","phrases":["ок"],"reply":{"clips":["ok"]}}"#,
        );
        let (p, errs) = parse_pack(&json).expect("parse");
        assert_eq!(p.commands.len(), 1);
        assert_eq!(p.commands[0].name, "E");
        assert_eq!(errs.len(), 5, "{errs:?}");
    }

    #[test]
    fn unknown_action_type_is_a_parse_error() {
        let json = pack(r#"{"id":"a","name":"A","phrases":["x"],"actions":[{"type":"Nope"}]}"#);
        assert!(parse_pack(&json).is_err());
    }

    #[test]
    fn load_dir_reports_broken_files() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(dir.path().join("a.json"), pack(SPEC_EXAMPLE)).expect("w");
        std::fs::write(dir.path().join("b.json"), "{broken").expect("w");
        std::fs::write(dir.path().join("readme.txt"), "x").expect("w");
        let (packs, report) = load_dir(dir.path());
        assert_eq!(packs.len(), 1);
        assert_eq!(report.len(), 1);
    }
}
