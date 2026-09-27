//! Chains (SPEC §4.3): «открой браузер и включи музыку» → two commands, run in order.
//! Split only when every part matches a chainable command; otherwise the whole
//! utterance is one command (so «найди хлеб и масло» stays a search).

use super::matcher::{Match, Matcher};
use super::normalize_utterance;
use crate::commands::Command;

const SEPARATORS: &[&str] = &[
    "и",
    "потом",
    "затем",
    "а еще",
    "после этого",
    "и потом",
    "и затем",
];

/// Split a normalized utterance at chain words (longest separators first).
pub fn split(norm: &str) -> Vec<String> {
    let mut seps: Vec<&str> = SEPARATORS.to_vec();
    seps.sort_by_key(|s| std::cmp::Reverse(s.len()));
    let mut parts = vec![format!(" {norm} ")];
    for sep in seps {
        let pat = format!(" {sep} ");
        parts = parts
            .into_iter()
            .flat_map(|p| {
                p.split(&pat)
                    .map(|x| format!(" {} ", x.trim()))
                    .collect::<Vec<_>>()
            })
            .collect();
    }
    parts
        .into_iter()
        .map(|p| p.trim().to_owned())
        .filter(|p| !p.is_empty())
        .collect()
}

/// Commands to run for an utterance: a chain if all parts match chainable commands,
/// else the single best match; empty = not understood.
pub fn plan(
    utterance: &str,
    matcher: &Matcher,
    commands: &[Command],
    threshold: f64,
    rank: &dyn Fn(usize) -> Option<f64>,
) -> Vec<Match> {
    let norm = normalize_utterance(utterance);
    let parts = split(&norm);
    let whole = matcher.best_with(&norm, threshold, rank);
    if parts.len() > 1 {
        let chain: Option<Vec<Match>> = parts
            .iter()
            .map(|p| {
                matcher
                    .best_with(p, threshold, rank)
                    .filter(|m| commands[m.index].chainable)
            })
            .collect();
        if let Some(chain) = chain {
            // a near-perfect single match beats splitting («сохрани и закрой» as one command)
            if whole.as_ref().is_none_or(|w| w.score < 0.99) {
                return chain;
            }
        }
    }
    whole.into_iter().collect()
}

/// Voice answer to a confirmation: `Some(true)` yes, `Some(false)` no, `None` unrelated.
pub fn yes_no(utterance: &str) -> Option<bool> {
    let norm = normalize_utterance(utterance);
    const NO: &[&str] = &[
        "нет",
        "отмена",
        "отмени",
        "отменить",
        "не надо",
        "стоп",
        "не",
    ];
    const YES: &[&str] = &[
        "да",
        "подтверждаю",
        "подтвердить",
        "выполняй",
        "конечно",
        "ага",
        "угу",
        "давай",
        "так точно",
    ];
    let has = |list: &[&str]| {
        list.iter()
            .any(|w| format!(" {norm} ").contains(&format!(" {w} ")))
    };
    if has(NO) {
        Some(false)
    } else if has(YES) {
        Some(true)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::parse_pack;

    fn cmds() -> Vec<Command> {
        let json = r#"{"id":"t","name":"T","commands":[
          {"id":"browser","name":"Открыть браузер","phrases":["браузер"],"optional":["открой"],
           "actions":[{"type":"Launch.Url","url":"https://ya.ru"}]},
          {"id":"music","name":"Включить музыку","phrases":["включи музыку"],"actions":[{"type":"Media.PlayPause"}]},
          {"id":"off","name":"Выключить","phrases":["выключи компьютер"],"chainable":false,
           "actions":[{"type":"System.Shutdown"}]},
          {"id":"search","name":"Поиск","phrases":["найди {текст}"],
           "actions":[{"type":"Launch.Url","url":"https://ya.ru/search?text={текст}"}]}
        ]}"#;
        parse_pack(json).expect("pack").0.commands
    }

    fn ids(u: &str) -> Vec<String> {
        let c = cmds();
        let m = Matcher::new(&c);
        plan(u, &m, &c, 0.7, &|_| Some(0.0))
            .into_iter()
            .map(|x| c[x.index].id.clone())
            .collect()
    }

    #[test]
    fn splits_on_chain_words() {
        assert_eq!(
            split("открой браузер и включи музыку"),
            vec!["открой браузер", "включи музыку"]
        );
        assert_eq!(split("а потом б затем в"), vec!["а", "б", "в"]);
        assert_eq!(split("а и потом б"), vec!["а", "б"]);
        assert_eq!(split("одна команда"), vec!["одна команда"]);
    }

    #[test]
    fn plans_chain_single_and_none() {
        assert_eq!(
            ids("Открой браузер и включи музыку"),
            vec!["browser", "music"]
        );
        assert_eq!(
            ids("открой браузер потом включи музыку"),
            vec!["browser", "music"]
        );
        assert_eq!(ids("найди хлеб и масло"), vec!["search"]);
        assert_eq!(
            ids("открой браузер и выключи компьютер"),
            Vec::<String>::new(),
            "non-chainable part → whole utterance, no match"
        );
        assert_eq!(ids("спой песню"), Vec::<String>::new());
    }

    #[test]
    fn yes_no_answers() {
        for (u, want) in [
            ("да", Some(true)),
            ("Да, подтверждаю", Some(true)),
            ("выполняй", Some(true)),
            ("нет", Some(false)),
            ("отмена", Some(false)),
            ("не надо", Some(false)),
            ("какая погода", None),
        ] {
            assert_eq!(yes_no(u), want, "{u}");
        }
    }
}
