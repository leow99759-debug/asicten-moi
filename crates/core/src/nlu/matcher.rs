//! Phrase matcher (SPEC §4.2): exact → fuzzy tokens → synonyms, required + optional words,
//! slots `{число}` `{время}` `{текст}` `{приложение}`. Embeddings are an optional later tier.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use ts_rs::TS;

use super::normalize_utterance;
use super::numerals::parse_duration;
use crate::commands::Command;

/// Token similarity needed to count as the same word («ключи» ≈ «включи»).
const TOKEN_SIM: f64 = 0.8;
/// Score lost per unexpected extra word in the utterance.
const EXTRA_PENALTY: f64 = 0.2;

/// Colloquial → canonical word, applied to phrases and utterances alike.
const SYNONYMS: &[(&str, &str)] = &[
    ("вруби", "включи"),
    ("врубай", "включи"),
    ("вырубай", "выключи"),
    ("выруби", "выключи"),
    ("отключи", "выключи"),
    ("запусти", "открой"),
    ("запускай", "открой"),
    ("открывай", "открой"),
    ("комп", "компьютер"),
    ("компьютера", "компьютер"),
    ("пк", "компьютер"),
    ("ноут", "компьютер"),
    ("ноутбук", "компьютер"),
    ("звук", "громкость"),
    ("музон", "музыку"),
    ("музыка", "музыку"),
    ("музычку", "музыку"),
    ("окошко", "окно"),
    ("телегу", "телеграм"),
    ("телеграмм", "телеграм"),
    ("ютуб", "youtube"),
];

/// Words that may appear anywhere without penalty (units after slot numbers etc.).
const SOFT: &[&str] = &["процентов", "процента", "процент", "на", "мне", "же", "а"];

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(export)]
pub enum SlotValue {
    Number(f64),
    /// Seconds.
    Duration(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    /// Index into the command list given to [`Matcher::new`].
    pub index: usize,
    pub score: f64,
    pub slots: BTreeMap<String, SlotValue>,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Word(String),
    Slot(String),
}

struct Entry {
    index: usize,
    phrase: Vec<Tok>,
    optional: Vec<String>,
}

pub struct Matcher {
    entries: Vec<Entry>,
    /// Slot-free canonical phrase → entry ids, for the exact fast path.
    exact: HashMap<String, Vec<usize>>,
}

fn canon(w: &str) -> String {
    SYNONYMS
        .iter()
        .find(|(k, _)| *k == w)
        .map_or_else(|| w.to_owned(), |(_, v)| (*v).to_owned())
}

fn tokens(norm: &str) -> Vec<String> {
    norm.split(' ')
        .filter(|w| !w.is_empty())
        .map(canon)
        .collect()
}

fn phrase_tokens(phrase: &str) -> Vec<Tok> {
    // protect slots from normalization: «громкость на {число}»
    let mut out = Vec::new();
    for part in phrase.split_inclusive('}') {
        let (text, slot) = match part.find('{') {
            Some(i) => (&part[..i], Some(&part[i..])),
            None => (part, None),
        };
        out.extend(
            tokens(&normalize_utterance(text))
                .into_iter()
                .map(Tok::Word),
        );
        if let Some(s) = slot {
            out.push(Tok::Slot(s.trim().to_owned()));
        }
    }
    out
}

fn sim(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }
    if a.chars().count() < 3 || b.chars().count() < 3 {
        return 0.0;
    }
    strsim::normalized_levenshtein(a, b).max(strsim::jaro_winkler(a, b) - 0.05)
}

fn is_duration_word(w: &str) -> bool {
    w.parse::<f64>().is_ok() || parse_duration(w).is_some() || matches!(w, "через" | "на")
}

impl Matcher {
    pub fn new(commands: &[Command]) -> Self {
        let mut entries = Vec::new();
        let mut exact: HashMap<String, Vec<usize>> = HashMap::new();
        for (index, c) in commands.iter().enumerate() {
            let optional: Vec<String> = c
                .optional
                .iter()
                .flat_map(|o| tokens(&normalize_utterance(o)))
                .collect();
            for p in &c.phrases {
                let phrase = phrase_tokens(p);
                if phrase.is_empty() {
                    continue;
                }
                if phrase.iter().all(|t| matches!(t, Tok::Word(_))) {
                    let key = phrase
                        .iter()
                        .filter_map(|t| match t {
                            Tok::Word(w) => Some(w.as_str()),
                            Tok::Slot(_) => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    exact.entry(key).or_default().push(entries.len());
                }
                entries.push(Entry {
                    index,
                    phrase,
                    optional: optional.clone(),
                });
            }
        }
        Self { entries, exact }
    }

    /// Best command for an utterance with score ≥ `threshold`.
    pub fn best(&self, utterance: &str, threshold: f64) -> Option<Match> {
        let u = tokens(&normalize_utterance(utterance));
        if u.is_empty() {
            return None;
        }
        // exact: utterance minus optional/soft words equals a phrase
        if let Some(e) = self.exact_candidate(&u) {
            return Some(Match {
                index: e.index,
                score: 1.0,
                slots: BTreeMap::new(),
            });
        }
        let mut best: Option<Match> = None;
        for e in &self.entries {
            if let Some((score, slots)) = score_entry(e, &u) {
                let better = best.as_ref().is_none_or(|b| score > b.score);
                if score >= threshold && better {
                    best = Some(Match {
                        index: e.index,
                        score,
                        slots,
                    });
                }
            }
        }
        best
    }

    fn exact_candidate(&self, u: &[String]) -> Option<&Entry> {
        if let Some(&i) = self.exact.get(&u.join(" ")).and_then(|v| v.first()) {
            return Some(&self.entries[i]);
        }
        self.entries.iter().find(|e| {
            let phrase: Vec<&str> = e
                .phrase
                .iter()
                .filter_map(|t| match t {
                    Tok::Word(w) => Some(w.as_str()),
                    Tok::Slot(_) => None,
                })
                .collect();
            let rest: Vec<&str> = u
                .iter()
                .map(String::as_str)
                .filter(|w| !e.optional.iter().any(|o| o == w))
                .collect();
            !e.optional.is_empty() && phrase.len() == e.phrase.len() && rest == phrase
        })
    }
}

/// Align phrase tokens to utterance tokens; extra utterance words cost unless optional/soft.
fn score_entry(e: &Entry, u: &[String]) -> Option<(f64, BTreeMap<String, SlotValue>)> {
    let mut slots = BTreeMap::new();
    let (sum, extras) = align(e, &e.phrase, u, &mut slots)?;
    let words = e.phrase.len() as f64;
    let base = sum / words;
    // longer phrases win ties («включи музыку» over «музыку»)
    let score = (base - EXTRA_PENALTY * extras as f64 + 0.001 * words).min(1.0);
    Some((score, slots))
}

/// Returns (similarity sum over phrase tokens, number of penalized extra words).
fn align(
    e: &Entry,
    p: &[Tok],
    u: &[String],
    slots: &mut BTreeMap<String, SlotValue>,
) -> Option<(f64, usize)> {
    let free = |w: &str| e.optional.iter().any(|o| o == w) || SOFT.contains(&w);
    let Some(first) = p.first() else {
        let extras = u.iter().filter(|w| !free(w)).count();
        return Some((0.0, extras));
    };
    let mut best: Option<(f64, usize, BTreeMap<String, SlotValue>)> = None;
    let mut consider =
        |r: Option<(f64, usize)>, gain: f64, skipped: usize, s: BTreeMap<String, SlotValue>| {
            if let Some((sum, extras)) = r {
                let cand = (sum + gain, extras + skipped);
                let key = |c: &(f64, usize)| c.0 - EXTRA_PENALTY * c.1 as f64;
                if best.as_ref().is_none_or(|b| key(&cand) > key(&(b.0, b.1))) {
                    best = Some((cand.0, cand.1, s));
                }
            }
        };
    let mut skipped = 0;
    for start in 0..u.len() {
        match first {
            Tok::Word(w) => {
                let s = sim(w, &u[start]);
                if s >= TOKEN_SIM {
                    let mut sub = slots.clone();
                    let r = align(e, &p[1..], &u[start + 1..], &mut sub);
                    consider(r, s, skipped, sub);
                }
            }
            Tok::Slot(name) => {
                for end in start + 1..=u.len() {
                    let span = &u[start..end];
                    let Some(v) = slot_value(name, span) else {
                        continue;
                    };
                    let mut sub = slots.clone();
                    sub.insert(name.clone(), v);
                    let r = align(e, &p[1..], &u[end..], &mut sub);
                    consider(r, 1.0, skipped, sub);
                }
            }
        }
        if !free(&u[start]) {
            skipped += 1;
        }
    }
    let (sum, extras, s) = best?;
    *slots = s;
    Some((sum, extras))
}

fn slot_value(name: &str, span: &[String]) -> Option<SlotValue> {
    match name {
        "{число}" => (span.len() == 1)
            .then(|| span[0].parse::<f64>().ok())
            .flatten()
            .map(SlotValue::Number),
        "{время}" => {
            if !span.iter().all(|w| is_duration_word(w)) {
                return None;
            }
            parse_duration(&span.join(" ")).map(SlotValue::Duration)
        }
        _ => Some(SlotValue::Text(span.join(" "))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::parse_pack;

    fn cmds() -> Vec<Command> {
        let json = r#"{"id":"t","name":"T","commands":[
          {"id":"browser","name":"Браузер","phrases":["браузер","хром"],
           "optional":["открой","запусти","включи","пожалуйста","ну-ка","давай"],
           "actions":[{"type":"Launch.Url","url":"https://ya.ru"}]},
          {"id":"music","name":"Музыка","phrases":["включи музыку"],"actions":[{"type":"Media.PlayPause"}]},
          {"id":"close","name":"Закрыть окно","phrases":["закрой окно"],"actions":[{"type":"Window.Close"}]},
          {"id":"minall","name":"Свернуть всё","phrases":["сверни все окна"],"actions":[{"type":"Window.MinimizeAll"}]},
          {"id":"vol","name":"Громкость","phrases":["громкость на {число}"],
           "actions":[{"type":"System.VolumeSet","level":"{число}"}]},
          {"id":"off_in","name":"Выключить через","phrases":["выключи компьютер через {время}"],
           "actions":[{"type":"System.Shutdown","delay_sec":"{время}"}]},
          {"id":"off","name":"Выключить","phrases":["выключи компьютер"],"actions":[{"type":"System.Shutdown"}]},
          {"id":"tg","name":"Telegram","phrases":["открой телеграм"],"actions":[{"type":"Launch.Find","name":"Telegram"}]},
          {"id":"search","name":"Поиск","phrases":["найди {текст}"],"actions":[{"type":"Launch.Url","url":"https://ya.ru/search?text={текст}"}]},
          {"id":"thanks","name":"Спасибо","phrases":["спасибо"],"reply":{"clips":["thanks"]}},
          {"id":"howru","name":"Как дела","phrases":["как дела"],"reply":{"clips":["status"]}}
        ]}"#;
        parse_pack(json).expect("pack").0.commands
    }

    fn id(m: &Matcher, c: &[Command], text: &str) -> Option<String> {
        m.best(text, 0.7).map(|x| c[x.index].id.clone())
    }

    #[test]
    fn required_and_optional_words() {
        let c = cmds();
        let m = Matcher::new(&c);
        for t in [
            "браузер",
            "открой браузер",
            "открой браузер пожалуйста",
            "ну-ка запусти хром",
        ] {
            assert_eq!(id(&m, &c, t).as_deref(), Some("browser"), "{t}");
        }
    }

    #[test]
    fn fuzzy_synonyms_and_stt_errors() {
        let c = cmds();
        let m = Matcher::new(&c);
        for (t, want) in [
            ("ключи музыку", "music"),
            ("вруби музон", "music"),
            ("закрой окошко", "close"),
            ("свернее все окна", "minall"),
            ("открой телег", "tg"),
            ("открой телегу", "tg"),
            ("джавис как дело", "howru"),
            ("спасибо джарвис", "thanks"),
            ("выруби комп", "off"),
        ] {
            assert_eq!(id(&m, &c, t).as_deref(), Some(want), "{t}");
        }
        assert_eq!(id(&m, &c, "какая сегодня луна"), None);
        assert_eq!(id(&m, &c, "расскажи анекдот про котов"), None);
    }

    #[test]
    fn slots() {
        let c = cmds();
        let m = Matcher::new(&c);
        let r = m.best("громкость на пятьдесят", 0.7).expect("vol");
        assert_eq!(c[r.index].id, "vol");
        assert_eq!(r.slots["{число}"], SlotValue::Number(50.0));
        let r = m.best("громкость на 30 процентов", 0.7).expect("vol%");
        assert_eq!(r.slots["{число}"], SlotValue::Number(30.0));
        let r = m
            .best("выключи компьютер через два часа", 0.7)
            .expect("off_in");
        assert_eq!(c[r.index].id, "off_in");
        assert_eq!(r.slots["{время}"], SlotValue::Duration(7200.0));
        let r = m
            .best("выключи компьютер через полчаса", 0.7)
            .expect("off_in");
        assert_eq!(r.slots["{время}"], SlotValue::Duration(1800.0));
        assert_eq!(id(&m, &c, "выключи компьютер").as_deref(), Some("off"));
        let r = m.best("найди рецепт борща", 0.7).expect("search");
        assert_eq!(r.slots["{текст}"], SlotValue::Text("рецепт борща".into()));
    }

    #[test]
    fn bench_1000_commands_under_30ms() {
        let mut c = cmds();
        let words = [
            "окно",
            "файл",
            "папку",
            "игру",
            "сайт",
            "видео",
            "чат",
            "почту",
            "карту",
            "заметку",
        ];
        let verbs = [
            "открой",
            "закрой",
            "покажи",
            "сохрани",
            "удали",
            "найди",
            "запиши",
            "сверни",
            "обнови",
            "скопируй",
        ];
        let mut n = 0;
        'outer: for v in verbs {
            for w in words {
                for k in 0..10 {
                    let mut cmd = c[2].clone();
                    cmd.id = format!("gen{n}");
                    cmd.phrases = vec![format!("{v} {w} номер {k}"), format!("{w} {v} {k}")];
                    c.push(cmd);
                    n += 1;
                    if n == 1000 {
                        break 'outer;
                    }
                }
            }
        }
        let m = Matcher::new(&c);
        let t = std::time::Instant::now();
        let runs = 20;
        for _ in 0..runs {
            assert_eq!(id(&m, &c, "громкость на пятьдесят").as_deref(), Some("vol"));
        }
        let per = t.elapsed() / runs;
        let limit = if cfg!(debug_assertions) { 300 } else { 30 };
        assert!(per.as_millis() <= limit, "{per:?} per match");
    }
}
