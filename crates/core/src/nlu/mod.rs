//! NLU (SPEC §4.2): normalization, numerals, matching.

pub mod numerals;

use crate::text::{normalize, strip_wake};

/// Filler words dropped before matching.
const FILLERS: &[&str] = &[
    "пожалуйста",
    "ну",
    "ка",
    "давай",
    "слушай",
    "эй",
    "короче",
    "так",
    "эээ",
    "ээ",
    "мм",
    "пожалуй",
    "плиз",
    "можешь",
    "можно",
];

/// Raw STT/typed text → matcher input: lowercase, ё→е, no punctuation, no wake word,
/// no fillers, numerals as digits. «Джарвис, ну-ка громкость на пятьдесят!» → «громкость на 50».
pub fn normalize_utterance(raw: &str) -> String {
    let base = strip_wake(&normalize(raw));
    let kept: Vec<&str> = base
        .split(' ')
        .filter(|w| !w.is_empty() && !FILLERS.contains(w))
        .collect();
    numerals::replace_numbers(&kept.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utterances() {
        for (a, b) in [
            ("Джарвис, ну-ка громкость на пятьдесят!", "громкость на 50"),
            ("Открой браузер, пожалуйста", "открой браузер"),
            ("Спасибо, Джарвис", "спасибо"),
            (
                "Выключи компьютер через два часа.",
                "выключи компьютер через 2 часа",
            ),
            ("Давай ЁЛКУ", "елку"),
            ("джарвис", ""),
        ] {
            assert_eq!(normalize_utterance(a), b, "{a}");
        }
    }
}
