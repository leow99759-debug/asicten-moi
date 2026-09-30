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

/// SPEC §0.1: spoken when a command needs the network.
pub const NO_INTERNET_PHRASE: &str = "Сэр, нет подключения к интернету. Эта функция требует сети.";

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
            Status::NoInternet => Some(Line::new(&["no_internet"], NO_INTERNET_PHRASE)),
            Status::Error => Some(Line::new(
                &["error"],
                format!("Сэр, не удалось выполнить: {error}"),
            )),
        };
        out.extend(closing);
        out
    }
}

/// Match score needed to run a command.
pub const THRESHOLD: f64 = 0.75;

/// Editor live preview row (§5.3): the command a sample phrase reaches.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Probe {
    pub text: String,
    /// First command the brain would run; `None` = not understood.
    pub id: Option<String>,
    pub name: Option<String>,
}

/// Which command each text reaches if `target` (unsaved edit) replaced its saved version.
/// Uses `target`'s own `when` context, so context commands preview in their app.
pub fn probe(commands: &[Command], target: &Command, texts: &[String]) -> Vec<Probe> {
    let mut set: Vec<Command> = commands
        .iter()
        .filter(|c| c.enabled || c.id == target.id)
        .cloned()
        .collect();
    match set.iter_mut().find(|c| c.id == target.id) {
        Some(c) => *c = target.clone(),
        None => set.push(target.clone()),
    }
    let matcher = Matcher::new(&set);
    let fg = target
        .when
        .as_ref()
        .and_then(|w| w.foreground.as_deref())
        .and_then(|f| f.split('|').next())
        .map(str::trim);
    let rank = |i: usize| set[i].context_rank(fg);
    texts
        .iter()
        .map(|text| {
            let hit = plan(text, &matcher, &set, THRESHOLD, &rank)
                .first()
                .map(|m| &set[m.index]);
            Probe {
                text: text.clone(),
                id: hit.map(|c| c.id.clone()),
                name: hit.map(|c| c.name.clone()),
            }
        })
        .collect()
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
            threshold: THRESHOLD,
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

    /// Editor «▶ Тест» (§5.3): run an unsaved command; slots come from a sample phrase.
    pub fn test(&self, cmd: &Command, sample: &str) -> CommandOutcome {
        let slots = Matcher::new(std::slice::from_ref(cmd))
            .best(sample, 0.0)
            .map(|m| m.slots)
            .unwrap_or_default();
        self.execute(cmd, &slots)
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

        assert!(b.handle("квантовая абракадабра").commands.is_empty());
    }

    #[test]
    fn probe_and_test_unsaved_command() {
        let dry = Arc::new(DryRun::default());
        let b = brain(dry.clone());
        let mut timer = b.commands()[2].clone();
        timer.phrases.push("выруби комп через {время}".into());
        let texts = [
            "открой браузер",
            "выруби комп через 10 минут",
            "абракадабра",
        ]
        .map(String::from);
        let got: Vec<_> = probe(b.commands(), &timer, &texts)
            .into_iter()
            .map(|p| p.id)
            .collect();
        assert_eq!(
            got,
            [Some("browser"), Some("off_in"), None].map(|s| s.map(String::from))
        );
        // a new command whose phrase an existing one already owns shows the clash
        let json =
            r#"{"id":"b2","name":"Браузер 2","phrases":["браузер"],"reply":{"clips":["ok"]}}"#;
        let b2: Command = serde_json::from_str(json).expect("cmd");
        let p = probe(b.commands(), &b2, &["открой браузер".into()]);
        assert_eq!(p[0].name.as_deref(), Some("Открыть браузер"));

        let o = b.test(&timer, "выключи компьютер через 5 минут");
        assert_eq!(o.status, Status::Done);
        assert!(dry.actions().contains(&Action::Shutdown {
            delay_sec: Num::Value(300.0)
        }));
    }

    #[test]
    fn voice_lines_per_status() {
        let dry = Arc::new(DryRun::default());
        let b = brain(dry);
        let lines = |u: &str| b.handle(u).voice_lines();
        assert_eq!(lines("квантовая абракадабра")[0].clips, vec!["not_found"]);
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
        // every plain phrase of every context-free command reaches that command
        let mut clashes = Vec::new();
        for c in b.commands().iter().filter(|c| c.when.is_none()) {
            for p in c.phrases.iter().filter(|p| !p.contains('{')) {
                let got = ids(p);
                if got != vec![c.id.clone()] {
                    clashes.push(format!("«{p}» → {got:?}, want {}", c.id));
                }
            }
        }
        assert!(clashes.is_empty(), "{clashes:#?}");

        *dry.foreground.lock().expect("lock") = Some("POWERPNT.EXE".into());
        assert_eq!(ids("дальше"), vec!["office.ppt_next".to_owned()]);
        assert_eq!(
            ids("измени цвет темы на синий"),
            vec!["office.ppt_theme".to_owned()]
        );
    }

    #[test]
    fn addon_phrases_reach_their_command_in_context() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        let cmds: Vec<Command> = crate::commands::addons(&dir, &[])
            .into_iter()
            .flat_map(|a| a.pack.commands)
            .collect();
        let dry = Arc::new(DryRun::default());
        let mut ex = Executor::new(dry.clone(), dry.clone(), Arc::new(NoApps));
        ex.pauses = false;
        let b = Brain::new(cmds, ex, dry.clone());
        let mut clashes = Vec::new();
        for c in b.commands() {
            let fg = c.when.as_ref().and_then(|w| w.foreground.as_deref());
            *dry.foreground.lock().expect("lock") =
                fg.and_then(|f| f.split('|').next()).map(str::to_owned);
            for p in c.phrases.iter().filter(|p| !p.contains('{')) {
                let got: Vec<String> = b.handle(p).commands.into_iter().map(|o| o.id).collect();
                if got != vec![c.id.clone()] {
                    clashes.push(format!("«{p}» ({fg:?}) → {got:?}, want {}", c.id));
                }
            }
        }
        assert!(clashes.is_empty(), "{clashes:#?}");

        for (fg, u, id) in [
            (None, "найди на ютубе смешных котов", "youtube.search"),
            (
                Some("chrome.exe"),
                "найди на ютубе смешных котов",
                "youtube.search",
            ),
            (
                Some("chrome.exe"),
                "загугли курс биткоина",
                "chrome.web_search",
            ),
            (
                Some("opera.exe"),
                "загугли курс биткоина",
                "operagx.web_search",
            ),
            (None, "найди в хроме рецепт борща", "chrome.search"),
            (None, "яркость на 40", "windows.bright_set"),
            (None, "окно на монитор 2", "windows.win_monitor_n"),
            (Some("chrome.exe"), "назад на 10 секунд", "youtube.back10"),
            (Some("chrome.exe"), "назад", "chrome.back"),
            (Some("explorer.exe"), "назад", "explorer.back"),
            (None, "дальше", "basic.next"),
            (None, "курс доллара", "basic.usd"),
            (None, "джарвис что нового сегодня", "basic.news"),
            (None, "какие новости", "basic.news"),
            (None, "что нового", "basic.news"),
            (
                None,
                "джарвис какие задания на завтра",
                "basic.homework_tomorrow",
            ),
            (None, "что задали на сегодня", "basic.homework_today"),
            (None, "какая домашка на послезавтра", "basic.homework_after"),
            (None, "какие у меня задания", "basic.homework"),
            (None, "что по домашке", "basic.homework"),
            (None, "курс долла", "basic.usd"),
            (None, "джарвис курс евро", "basic.eur"),
            (None, "открой проводник", "explorer.open"),
            (None, "открой ютуб", "youtube.open"),
            (None, "это открой браузер", "basic.browser"),
            (None, "открой вс код", "basic.launch"),
            (None, "открой дискорд", "basic.launch"),
            (None, "запусти обс", "basic.launch"),
            (None, "открой протон впн", "basic.launch"),
            (None, "открой гитхаб", "basic.launch"),
            (None, "напоминание поставь", "basic.remind_help"),
            (None, "поставь напоминание", "basic.remind_help"),
            (
                None,
                "напомни через 10 минут проверить духовку",
                "basic.remind",
            ),
            (
                None,
                "поставь напоминание через пять минут позвонить маме",
                "basic.remind",
            ),
            (None, "поставь таймер на 5 минут", "basic.timer"),
            (None, "найди видео мистер бист", "youtube.search"),
            (None, "включи песню группа крови", "basic.song"),
            (None, "включи музыку", "basic.music"),
            (None, "джарвис выключи себя", "basic.quit"),
            (None, "закрой телеграм", "windows.close_app"),
            (None, "джарвис закрой дискорд", "windows.close_app"),
            (None, "закрой окно", "basic.close_window"),
            (None, "закрой все окна", "basic.close_all"),
            (None, "закрой хром", "chrome.close"),
            (Some("chrome.exe"), "закрой вкладку", "chrome.tab_close"),
            (None, "сделай ярче", "windows.bright_up"),
            (None, "темнее на 20", "windows.bright_down_n"),
            (None, "выключи экран", "windows.monitor_off"),
            (None, "включи темную тему", "windows.dark_on"),
            (None, "какой у меня ip", "windows.ip"),
            (None, "сколько места на диске", "windows.disk"),
            (Some("discord.exe"), "прими звонок", "discord.answer"),
            (Some("spotify.exe"), "лайкни трек", "spotify.like"),
            (None, "открой библиотеку стим", "steam.library"),
            (None, "открой телеграм", "basic.launch"),
            (None, "включи тихий режим", "modes.silent_on"),
            (None, "выключи компьютер", "basic.shutdown"),
            (None, "сделай тише на 20", "basic.volume_down_n"),
            (None, "тише на двадцать процентов", "basic.volume_down_n"),
            (None, "джарвис громче на 10", "basic.volume_up_n"),
            (None, "сделай тише", "basic.volume_down"),
            (None, "закрой все окна", "basic.close_all"),
            (None, "джарвис сверни все вкладки", "basic.minimize_all"),
            (None, "скриншот экрана", "basic.screenshot"),
            (None, "скриншот области", "windows.snip"),
            (None, "покажи скриншоты", "basic.screenshots"),
        ] {
            *dry.foreground.lock().expect("lock") = fg.map(str::to_owned);
            let got: Vec<String> = b.handle(u).commands.into_iter().map(|o| o.id).collect();
            assert_eq!(got, vec![id.to_owned()], "{u} ({fg:?})");
        }
        let before = dry.actions().len();
        b.handle("тише на 20 процентов");
        assert_eq!(
            dry.actions()[before..],
            [Action::VolumeDown {
                step: Num::Value(20.0)
            }]
        );
    }
}
