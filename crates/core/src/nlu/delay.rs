//! «через 5 минут смени язык» / «смени язык через 5 минут»: any command, run later.

use super::numerals::parse_duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Delay {
    pub sec: f64,
    /// Utterance without «через …» (normalized).
    pub rest: String,
    /// The time words, e.g. «5 минут».
    pub span: String,
}

fn time_word(w: &str) -> bool {
    w.parse::<f64>().is_ok() || parse_duration(w).is_some()
}

/// Normalized utterance → delay + remaining command; `None` without «через <время>».
pub fn split_delay(norm: &str) -> Option<Delay> {
    let w: Vec<&str> = norm.split(' ').filter(|w| !w.is_empty()).collect();
    let at = w.iter().position(|x| *x == "через")?;
    let end = at + 1 + w[at + 1..].iter().take_while(|x| time_word(x)).count();
    let span = w[at + 1..end].join(" ");
    let sec = parse_duration(&span).filter(|s| *s > 0.0)?;
    let rest: Vec<&str> = w[..at]
        .iter()
        .chain(&w[end..])
        .copied()
        .skip_while(|x| matches!(*x, "сделай" | "выполни"))
        .collect();
    (!rest.is_empty()).then(|| Delay {
        sec,
        rest: rest.join(" "),
        span,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_both_orders() {
        let d = split_delay("через 5 минут сделай громкость 20").expect("delay");
        assert_eq!(
            (d.sec, d.rest.as_str(), d.span.as_str()),
            (300.0, "громкость 20", "5 минут")
        );
        let d = split_delay("смени язык через 5 секунд").expect("delay");
        assert_eq!((d.sec, d.rest.as_str()), (5.0, "смени язык"));
        let d = split_delay("через час 30 минут выключи звук").expect("delay");
        assert_eq!(d.sec, 5400.0);
        assert!(split_delay("через 5 минут").is_none());
        assert!(split_delay("пройди через дверь").is_none());
        assert!(split_delay("громкость 20").is_none());
    }
}
