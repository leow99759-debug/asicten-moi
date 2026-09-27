//! Utterance → plan (single/chain) → confirm if needed (§4.6) → execute → outcome.
//! Runs on its own worker thread in the app, so `Assistant::ask` can block while the
//! voice loop keeps listening for «да/нет».

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Serialize;
use ts_rs::TS;

use crate::commands::{Command, Reply};
use crate::db::Status;
use crate::executor::{Assistant, Executor, StepResult};
use crate::nlu::chain::plan;
use crate::nlu::matcher::{Matcher, SlotValue};

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CommandOutcome {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub steps: Vec<StepResult>,
    pub reply: Reply,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Outcome {
    pub phrase: String,
    /// Empty = not understood.
    pub commands: Vec<CommandOutcome>,
}

impl Outcome {
    /// History view (§3.4): ids joined with `+`, first non-Done status (Done if all ok);
    /// not understood = `(None, Error)`.
    pub fn summary(&self) -> (Option<String>, Status) {
        if self.commands.is_empty() {
            return (None, Status::Error);
        }
        let ids: Vec<&str> = self.commands.iter().map(|c| c.id.as_str()).collect();
        let status = self
            .commands
            .iter()
            .map(|c| c.status)
            .find(|s| *s != Status::Done)
            .unwrap_or(Status::Done);
        (Some(ids.join("+")), status)
    }
}

/// One thing to say: a clip from the first available category, else `text` via TTS.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    pub clips: Vec<String>,
    pub text: Option<String>,
}

impl Line {
    fn new(clips: &[&str], text: impl Into<String>) -> Self {
        Self {
            clips: clips.iter().map(|c| (*c).to_owned()).collect(),
            text: Some(text.into()),
        }
    }
}

impl Outcome {
    /// What Jarvis says afterwards: step outputs (time, battery…), then one closing line
    /// for the last command — its reply when done, else the status phrase (§4.4, §6.1).
    pub fn voice_lines(&self) -> Vec<Line> {
        let Some(last) = self.commands.last() else {
            return vec![Line::new(&["not_found"], "Простите, сэр, не понял команду")];
        };
        let mut out: Vec<Line> = self
            .commands
            .iter()
            .flat_map(|c| c.steps.iter().filter_map(|s| s.output.clone()))
            .map(|t| Line {
                clips: vec![],
                text: Some(t),
            })
            .collect();
        let error = last
            .steps
            .iter()
            .find_map(|s| s.error.clone())
            .unwrap_or_default();
        let closing = match last.status {
            Status::Done if last.reply.clips.is_empty() && last.reply.text.is_none() => None,
            Status::Done => Some(Line {
                clips: last.reply.clips.clone(),
                text: last.reply.text.clone(),
            }),
            Status::Cancelled => Some(Line::new(&["cancel"], "Отменено, сэр")),
            Status::NoInternet => Some(Line::new(&["no_internet"], crate::NO_INTERNET)),
            Status::Error => Some(Line::new(
                &["error"],
                format!("Сэр, не удалось выполнить: {error}"),
            )),
        };
        out.extend(closing);
        out
    }
}

pub struct Brain {
    commands: Vec<Command>,
    matcher: Matcher,
    executor: Executor,
    assistant: Arc<dyn Assistant>,
    pub threshold: f64,
}

impl Brain {
    pub fn new(commands: Vec<Command>, executor: Executor, assistant: Arc<dyn Assistant>) -> Self {
        let matcher = Matcher::new(&commands);
        Self {
            commands,
            matcher,
            executor,
            assistant,
            threshold: 0.75,
        }
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn handle(&self, utterance: &str) -> Outcome {
        let mut out = Outcome {
            phrase: utterance.to_owned(),
            commands: Vec::new(),
        };
        let fg = self.executor.foreground_exe();
        let rank = |i: usize| self.commands[i].context_rank(fg.as_deref());
        for m in plan(
            utterance,
            &self.matcher,
            &self.commands,
            self.threshold,
            &rank,
        ) {
            let o = self.execute(&self.commands[m.index], &m.slots);
            let stop = o.status != Status::Done;
            out.commands.push(o);
            if stop {
                break;
            }
        }
        out
    }

    /// Run a command by id (timers, UI «▶ Test», history repeat).
    pub fn run_by_id(&self, id: &str) -> Option<CommandOutcome> {
        let cmd = self.commands.iter().find(|c| c.id == id)?;
        Some(self.execute(cmd, &BTreeMap::new()))
    }

    fn execute(&self, cmd: &Command, slots: &BTreeMap<String, SlotValue>) -> CommandOutcome {
        let mut o = CommandOutcome {
            id: cmd.id.clone(),
            name: cmd.name.clone(),
            status: Status::Done,
            steps: Vec::new(),
            reply: cmd.reply.clone(),
        };
        if cmd.needs_confirm() {
            let q = format!("Сэр, выполнить «{}»?", cmd.name.to_lowercase());
            if !self.assistant.ask(&q).unwrap_or(false) {
                o.status = Status::Cancelled;
                return o;
            }
        }
        o.steps = self.executor.run(cmd, slots);
        if let Some(err) = o.steps.iter().find_map(|s| s.error.as_deref()) {
            o.status = if err == crate::NO_INTERNET {
                Status::NoInternet
            } else {
                Status::Error
            };
        }
        o
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{parse_pack, Action, AppLocator, Num};
    use crate::executor::DryRun;

    struct NoApps;
    impl AppLocator for NoApps {
        fn locate(&self, _: &str) -> Option<String> {
            None
        }
    }

    fn brain(dry: Arc<DryRun>) -> Brain {
        let json = r#"{"id":"t","name":"T","commands":[
          {"id":"browser","name":"Открыть браузер","phrases":["браузер"],"optional":["открой"],
           "actions":[{"type":"Launch.Url","url":"https://ya.ru"}],"reply":{"clips":["done"]}},
          {"id":"music","name":"Включить музыку","phrases":["включи музыку"],"actions":[{"type":"Media.PlayPause"}]},
          {"id":"off_in","name":"Выключить компьютер","phrases":["выключи компьютер через {время}"],
           "actions":[{"type":"System.Shutdown","delay_sec":"{время}"}]},
          {"id":"howru","name":"Как дела","phrases":["как дела"],"reply":{"clips":["status"]}}
        ]}"#;
        let cmds = parse_pack(json).expect("pack").0.commands;
        let mut ex = Executor::new(dry.clone(), dry.clone(), Arc::new(NoApps));
        ex.pauses = false;
        Brain::new(cmds, ex, dry)
    }

    #[test]
    fn chain_confirm_and_dialog() {
        let dry = Arc::new(DryRun::default());
        let b = brain(dry.clone());

        let o = b.handle("Открой браузер и включи музыку");
        let ids: Vec<_> = o
            .commands
            .iter()
            .map(|c| (c.id.as_str(), c.status))
            .collect();
        assert_eq!(
            ids,
            vec![("browser", Status::Done), ("music", Status::Done)]
        );

        let o = b.handle("выключи компьютер через два часа");
        assert_eq!(o.commands[0].status, Status::Done);
        assert!(dry.actions().contains(&Action::Ask {
            question: "Сэр, выполнить «выключить компьютер»?".into()
        }));
        assert!(dry.actions().contains(&Action::Shutdown {
            delay_sec: Num::Value(7200.0)
        }));

        let o = b.handle("Джарвис, как дела?");
        assert_eq!(o.commands[0].reply.clips, vec!["status"]);
        assert!(o.commands[0].steps.is_empty());

        assert!(b.handle("спой песню").commands.is_empty());
    }

    #[test]
    fn voice_lines_per_status() {
        let dry = Arc::new(DryRun::default());
        let b = brain(dry);
        let lines = |u: &str| b.handle(u).voice_lines();
        assert_eq!(lines("спой песню")[0].clips, vec!["not_found"]);
        assert_eq!(lines("открой браузер")[0].clips, vec!["done"]);
        assert!(lines("включи музыку").is_empty());
        let chain = lines("открой браузер и как дела");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].clips, vec!["status"]);
    }

    #[test]
    fn context_rules_pick_by_foreground() {
        let json = r#"{"id":"t","name":"T","commands":[
          {"id":"theme","name":"Тема Джарвиса","phrases":["измени цвет темы на {текст}"],
           "actions":[{"type":"Assistant.SetTheme","color":"{текст}"}]},
          {"id":"ppt","name":"Тема презентации","phrases":["измени цвет темы на {текст}"],
           "when":{"foreground":"WINWORD.EXE|POWERPNT.EXE"},
           "actions":[{"type":"PowerPoint.SetVariantColor","color":"{текст}"}]},
          {"id":"slide","name":"Следующий слайд","phrases":["следующий слайд"],
           "when":{"foreground":"POWERPNT.EXE"},"actions":[{"type":"Keys.Press","keys":"Right"}]}
        ]}"#;
        let dry = Arc::new(DryRun::default());
        let mut ex = Executor::new(dry.clone(), dry.clone(), Arc::new(NoApps));
        ex.pauses = false;
        let b = Brain::new(parse_pack(json).expect("pack").0.commands, ex, dry.clone());
        let id = |u: &str| b.handle(u).commands.first().map(|c| c.id.clone());

        assert_eq!(
            id("измени цвет темы на фиолетовый").as_deref(),
            Some("theme")
        );
        assert_eq!(id("следующий слайд"), None);

        *dry.foreground.lock().expect("lock") = Some("powerpnt.exe".into());
        assert_eq!(id("измени цвет темы на фиолетовый").as_deref(), Some("ppt"));
        assert_eq!(id("следующий слайд").as_deref(), Some("slide"));
        assert!(dry.actions().contains(&Action::PptSetVariantColor {
            color: "фиолетовый".into()
        }));
    }

    #[test]
    fn repo_packs_modes_and_context() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        let cmds: Vec<Command> = crate::commands::load_dir(&dir)
            .0
            .into_iter()
            .flat_map(|p| p.commands)
            .collect();
        let dry = Arc::new(DryRun::default());
        let mut ex = Executor::new(dry.clone(), dry.clone(), Arc::new(NoApps));
        ex.pauses = false;
        let b = Brain::new(cmds, ex, dry.clone());
        let ids =
            |u: &str| -> Vec<String> { b.handle(u).commands.into_iter().map(|c| c.id).collect() };

        for (u, id) in [
            ("Джарвис, запусти игровой режим", "modes.game_on"),
            ("включи режим фильма", "modes.movie_on"),
            ("ночной режим", "modes.night_on"),
            ("рабочий режим", "modes.work_on"),
            ("выключи игровой режим", "modes.game_off"),
            ("дальше", "basic.next"),
        ] {
            assert_eq!(ids(u), vec![id.to_owned()], "{u}");
        }
        *dry.foreground.lock().expect("lock") = Some("POWERPNT.EXE".into());
        assert_eq!(ids("дальше"), vec!["office.ppt_next".to_owned()]);
        assert_eq!(
            ids("измени цвет темы на синий"),
            vec!["office.ppt_theme".to_owned()]
        );
    }
}
