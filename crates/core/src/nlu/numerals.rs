//! Russian numerals and durations (SPEC §4.2): «пятьдесят» → 50, «сто двадцать пять» → 125,
//! «два часа» → 7200 s, «полчаса», «через минуту», «полтора часа», «час тридцать минут».

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Unit,
    Ten,
    Hundred,
}

fn word_value(w: &str) -> Option<(f64, Class)> {
    use Class::*;
    let v = match w {
        "ноль" | "нуль" => (0.0, Unit),
        "один" | "одна" | "одно" | "одну" | "одного" | "одной" => {
            (1.0, Unit)
        }
        "два" | "две" | "двух" => (2.0, Unit),
        "три" | "трех" => (3.0, Unit),
        "четыре" | "четырех" => (4.0, Unit),
        "пять" | "пяти" => (5.0, Unit),
        "шесть" | "шести" => (6.0, Unit),
        "семь" | "семи" => (7.0, Unit),
        "восемь" | "восьми" => (8.0, Unit),
        "девять" | "девяти" => (9.0, Unit),
        "десять" | "десяти" => (10.0, Ten),
        "одиннадцать" => (11.0, Ten),
        "двенадцать" => (12.0, Ten),
        "тринадцать" => (13.0, Ten),
        "четырнадцать" => (14.0, Ten),
        "пятнадцать" | "пятнадцати" => (15.0, Ten),
        "шестнадцать" => (16.0, Ten),
        "семнадцать" => (17.0, Ten),
        "восемнадцать" => (18.0, Ten),
        "девятнадцать" => (19.0, Ten),
        "двадцать" | "двадцати" => (20.0, Ten),
        "тридцать" | "тридцати" => (30.0, Ten),
        "сорок" | "сорока" => (40.0, Ten),
        "пятьдесят" | "пятидесяти" => (50.0, Ten),
        "шестьдесят" => (60.0, Ten),
        "семьдесят" => (70.0, Ten),
        "восемьдесят" => (80.0, Ten),
        "девяносто" => (90.0, Ten),
        "сто" | "ста" => (100.0, Hundred),
        "двести" => (200.0, Hundred),
        "триста" => (300.0, Hundred),
        "четыреста" => (400.0, Hundred),
        "пятьсот" => (500.0, Hundred),
        "шестьсот" => (600.0, Hundred),
        "семьсот" => (700.0, Hundred),
        "восемьсот" => (800.0, Hundred),
        "девятьсот" => (900.0, Hundred),
        "полтора" | "полторы" => (1.5, Unit),
        _ => return None,
    };
    Some(v)
}

fn is_thousand(w: &str) -> bool {
    matches!(w, "тысяча" | "тысячи" | "тысяч" | "тысячу")
}

/// Parse a number starting at `words[0]`; returns (value, words consumed).
pub fn parse_number(words: &[&str]) -> Option<(f64, usize)> {
    if let Some(first) = words.first() {
        if let Ok(v) = first.replace(',', ".").parse::<f64>() {
            return Some((v, 1));
        }
    }
    let (mut total, mut current, mut used) = (0.0, 0.0, 0);
    let mut last: Option<Class> = None;
    for w in words {
        if let Some((v, class)) = word_value(w) {
            // «двадцать пять» ok, «пять двадцать» = two numbers; teens take the units slot
            let fits = match last {
                None => true,
                Some(l) => class < l && !(l == Class::Ten && current % 10.0 != 0.0),
            };
            if !fits {
                break;
            }
            current += v;
            last = Some(class);
        } else if is_thousand(w) {
            total += if current == 0.0 {
                1000.0
            } else {
                current * 1000.0
            };
            current = 0.0;
            last = None;
        } else {
            break;
        }
        used += 1;
    }
    (used > 0).then_some((total + current, used))
}

/// Replace every number word sequence with digits: «громкость на пятьдесят» → «громкость на 50».
/// Input must be normalized (lowercase, no punctuation).
pub fn replace_numbers(text: &str) -> String {
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let mut out: Vec<String> = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        match parse_number(&words[i..]) {
            Some((v, n)) => {
                out.push(fmt_num(v));
                i += n;
            }
            None => {
                out.push(words[i].to_owned());
                i += 1;
            }
        }
    }
    out.join(" ")
}

fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn unit_seconds(w: &str) -> Option<f64> {
    match w {
        "секунда" | "секунду" | "секунды" | "секунд" | "сек" => {
            Some(1.0)
        }
        "минута" | "минуту" | "минуты" | "минут" | "мин" => Some(60.0),
        "час" | "часа" | "часов" | "часик" => Some(3600.0),
        "сутки" | "суток" => Some(86_400.0),
        _ => None,
    }
}

fn half_unit(w: &str) -> Option<f64> {
    match w {
        "полчаса" => Some(1800.0),
        "полминуты" => Some(30.0),
        "полсекунды" => Some(0.5),
        _ => None,
    }
}

/// Duration in seconds from normalized text with digits (run [`replace_numbers`] first):
/// «2 часа» → 7200, «через минуту» → 60, «полчаса» → 1800, «1 час 30 минут» → 5400.
/// Sums every «[number] unit» group found; `None` if there is none.
pub fn parse_duration(text: &str) -> Option<f64> {
    let words: Vec<&str> = text.split(' ').filter(|w| !w.is_empty()).collect();
    let mut total = 0.0;
    let mut found = false;
    let mut i = 0;
    while i < words.len() {
        if let Some(s) = half_unit(words[i]) {
            total += s;
            found = true;
            i += 1;
            continue;
        }
        let num = words[i].replace(',', ".").parse::<f64>().ok();
        match (num, words.get(i + 1).and_then(|w| unit_seconds(w))) {
            (Some(n), Some(u)) => {
                total += n * u;
                found = true;
                i += 2;
            }
            _ => {
                if let Some(u) = unit_seconds(words[i]) {
                    // bare unit: «через час», «через минуту»
                    total += u;
                    found = true;
                } else if num.is_some() && found {
                    // «час 30» → 30 minutes after hours, «1 минута 30» → seconds
                    let prev = words[i - 1];
                    let tail = match unit_seconds(prev) {
                        Some(3600.0) => 60.0,
                        Some(60.0) => 1.0,
                        _ => 0.0,
                    };
                    total += num.unwrap_or(0.0) * tail;
                }
                i += 1;
            }
        }
    }
    found.then_some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num(s: &str) -> Option<f64> {
        let w: Vec<&str> = s.split(' ').collect();
        parse_number(&w).map(|(v, _)| v)
    }

    #[test]
    fn numbers() {
        let cases: &[(&str, f64)] = &[
            ("ноль", 0.0),
            ("один", 1.0),
            ("одну", 1.0),
            ("две", 2.0),
            ("пять", 5.0),
            ("десять", 10.0),
            ("одиннадцать", 11.0),
            ("девятнадцать", 19.0),
            ("двадцать", 20.0),
            ("двадцать пять", 25.0),
            ("сорок два", 42.0),
            ("пятьдесят", 50.0),
            ("девяносто девять", 99.0),
            ("сто", 100.0),
            ("сто двадцать пять", 125.0),
            ("двести", 200.0),
            ("триста пятнадцать", 315.0),
            ("девятьсот девяносто девять", 999.0),
            ("тысяча", 1000.0),
            ("две тысячи двадцать шесть", 2026.0),
            ("пять тысяч", 5000.0),
            ("полтора", 1.5),
            ("50", 50.0),
            ("2,5", 2.5),
        ];
        for (s, v) in cases {
            assert_eq!(num(s), Some(*v), "{s}");
        }
        assert_eq!(num("громкость"), None);
    }

    #[test]
    fn number_boundaries() {
        assert_eq!(parse_number(&["пять", "двадцать"]), Some((5.0, 1)));
        assert_eq!(
            parse_number(&["двадцать", "пять", "минут"]),
            Some((25.0, 2))
        );
        assert_eq!(parse_number(&["пятнадцать", "пять"]), Some((15.0, 1)));
        assert_eq!(parse_number(&["двадцать", "двадцать"]), Some((20.0, 1)));
    }

    #[test]
    fn replaces_in_text() {
        let cases = [
            ("громкость на пятьдесят", "громкость на 50"),
            (
                "выключи компьютер через два часа",
                "выключи компьютер через 2 часа",
            ),
            (
                "поставь таймер на двадцать пять минут",
                "поставь таймер на 25 минут",
            ),
            ("открой браузер", "открой браузер"),
            ("яркость сто", "яркость 100"),
            (
                "подожди две секунды и включи музыку",
                "подожди 2 секунды и включи музыку",
            ),
            ("через полтора часа", "через 1.5 часа"),
            ("громкость на 30", "громкость на 30"),
        ];
        for (a, b) in cases {
            assert_eq!(replace_numbers(a), b, "{a}");
        }
    }

    #[test]
    fn durations() {
        let cases: &[(&str, f64)] = &[
            ("2 часа", 7200.0),
            ("через 2 часа", 7200.0),
            ("через час", 3600.0),
            ("через минуту", 60.0),
            ("30 секунд", 30.0),
            ("полчаса", 1800.0),
            ("через полминуты", 30.0),
            ("1.5 часа", 5400.0),
            ("1 час 30 минут", 5400.0),
            ("час 30 минут", 5400.0),
            ("2 часа 15 минут", 8100.0),
            ("5 минут", 300.0),
            ("10 мин", 600.0),
            ("1 минута 30", 90.0),
            ("час 30", 5400.0),
            ("сутки", 86_400.0),
        ];
        for (s, v) in cases {
            assert_eq!(parse_duration(s), Some(*v), "{s}");
        }
        assert_eq!(parse_duration("открой браузер"), None);
        assert_eq!(parse_duration("громкость на 50"), None);
    }

    #[test]
    fn words_to_duration_end_to_end() {
        for (s, v) in [
            ("выключи компьютер через два часа", 7200.0),
            ("через двадцать пять минут", 1500.0),
            ("через полтора часа", 5400.0),
            ("таймер на сорок пять секунд", 45.0),
        ] {
            assert_eq!(parse_duration(&replace_numbers(s)), Some(v), "{s}");
        }
    }
}
