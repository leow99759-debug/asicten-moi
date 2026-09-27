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
}
