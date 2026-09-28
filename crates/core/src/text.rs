//! Russian text helpers shared by mode phrases, NLU and tests.

/// Lowercase, `ё`→`е`, keep only letters/digits, collapse whitespace.
pub fn normalize(s: &str) -> String {
    s.to_lowercase()
        .replace('ё', "е")
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// STT often mangles the name («джавис», «джарес», «жарвис»).
fn is_wake_word(w: &str) -> bool {
    const NAME: &str = "джарвис";
    let lev = 1.0
        - strsim::levenshtein(w, NAME) as f64 / w.chars().count().max(NAME.chars().count()) as f64;
    lev >= 0.55 && strsim::jaro_winkler(w, NAME) >= 0.8
}

/// Drop the wake word from the start/end of a normalized utterance
/// («джарвис как дела» → «как дела», «спасибо джарвис» → «спасибо»).
pub fn strip_wake(norm: &str) -> String {
    let mut words: Vec<&str> = norm.split(' ').filter(|w| !w.is_empty()).collect();
    if words.first().is_some_and(|w| is_wake_word(w)) {
        words.remove(0);
    }
    if words.last().is_some_and(|w| is_wake_word(w)) {
        words.pop();
    }
    words.join(" ")
}

/// Remove Jarvis's own words that the mic picked up from the speakers («да сэр открой
/// браузер» after he said «Да, сэр» → «открой браузер»). `heard` and `said` are normalized;
/// `said` words are matched in order (fuzzy); at least 60% of them must be found.
pub fn strip_echo(heard: &str, said: &str) -> String {
    let said: Vec<&str> = said.split_whitespace().collect();
    let heard: Vec<&str> = heard.split_whitespace().collect();
    let mut hit = vec![false; heard.len()];
    let mut j = 0;
    for (i, w) in heard.iter().enumerate() {
        if j < said.len() && same_word(w, said[j]) {
            hit[i] = true;
            j += 1;
        }
    }
    if said.is_empty() || j * 5 < said.len() * 3 {
        return heard.join(" ");
    }
    heard
        .iter()
        .zip(hit)
        .filter(|(_, h)| !h)
        .map(|(w, _)| *w)
        .collect::<Vec<_>>()
        .join(" ")
}

fn same_word(a: &str, b: &str) -> bool {
    let (a, b) = (a.replace('э', "е"), b.replace('э', "е"));
    a == b || strsim::jaro_winkler(&a, &b) >= 0.88
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes() {
        assert_eq!(
            normalize("  Джарвис, ЁЛКА!  2 раза "),
            "джарвис елка 2 раза"
        );
    }

    #[test]
    fn strips_mangled_wake_word_at_edges_only() {
        for (inp, out) in [
            ("джарвис как дела", "как дела"),
            ("джавис как дело", "как дело"),
            ("спасибо джарес", "спасибо"),
            ("жарвис открой браузер", "открой браузер"),
            ("включи джаз", "включи джаз"),
            ("открой джарвис браузер", "открой джарвис браузер"),
            ("джарвис", ""),
        ] {
            assert_eq!(strip_wake(inp), out, "{inp}");
        }
    }

    #[test]
    fn strips_own_echo() {
        for (heard, said, out) in [
            ("да сэр", "да сэр", ""),
            ("да сер", "да сэр", ""),
            ("да сэр открой браузер", "да сэр", "открой браузер"),
            ("чего вы пытаетесь", "чего вы пытаетесь добиться сэр", ""),
            ("открой браузер", "да сэр", "открой браузер"),
            ("да", "да сэр", "да"),
            ("открой браузер", "", "открой браузер"),
        ] {
            assert_eq!(strip_echo(heard, said), out, "{heard} / {said}");
        }
    }
}
