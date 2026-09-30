//! Russian phrasing for System.Info answers (time, date, battery, load). Pure, tested.

/// Russian plural: 1 час, 2 часа, 5 часов.
pub fn plural<'a>(n: i64, one: &'a str, few: &'a str, many: &'a str) -> &'a str {
    let (n10, n100) = (n.abs() % 10, n.abs() % 100);
    if n10 == 1 && n100 != 11 {
        one
    } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
        few
    } else {
        many
    }
}

pub fn time_phrase(hour: u32, minute: u32) -> String {
    let (h, m) = (i64::from(hour), i64::from(minute));
    let hours = format!("{h} {}", plural(h, "час", "часа", "часов"));
    if m == 0 {
        format!("Сейчас ровно {hours}, сэр")
    } else {
        format!(
            "Сейчас {hours} {m} {}, сэр",
            plural(m, "минута", "минуты", "минут")
        )
    }
}

const MONTHS: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];
/// 0 = Sunday (Win32 SYSTEMTIME.wDayOfWeek).
const WEEKDAYS: [&str; 7] = [
    "воскресенье",
    "понедельник",
    "вторник",
    "среда",
    "четверг",
    "пятница",
    "суббота",
];

/// «Доброе утро/день/вечер» by local hour (greet category, §6.1).
pub fn greeting_phrase(hour: u32) -> &'static str {
    match hour {
        5..=11 => "Доброе утро, сэр",
        12..=17 => "Добрый день, сэр",
        18..=22 => "Добрый вечер, сэр",
        _ => "Доброй ночи, сэр",
    }
}

/// Voice category for the time-of-day greeting (`greet_morning` … `greet_night`).
pub fn greet_category(hour: u32) -> &'static str {
    match hour {
        5..=11 => "greet_morning",
        12..=17 => "greet_day",
        18..=22 => "greet_evening",
        _ => "greet_night",
    }
}

pub fn date_phrase(day: u32, month: u32, weekday: u32) -> String {
    let m = MONTHS
        .get(month.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    let w = WEEKDAYS.get(weekday as usize).copied().unwrap_or("");
    format!("Сегодня {w}, {day} {m}, сэр")
}

pub fn battery_phrase(percent: Option<u8>, charging: bool) -> String {
    match percent {
        None => "Сэр, батарея не обнаружена, питание от сети".into(),
        Some(p) => {
            let p = i64::from(p);
            let tail = if charging {
                ", идёт зарядка"
            } else {
                ""
            };
            format!(
                "Заряд батареи {p} {}{tail}",
                plural(p, "процент", "процента", "процентов")
            )
        }
    }
}

pub fn load_phrase(cpu: u8, ram: u8) -> String {
    let (c, r) = (i64::from(cpu), i64::from(ram));
    format!(
        "Загрузка процессора {c} {}, памяти {r} {}",
        plural(c, "процент", "процента", "процентов"),
        plural(r, "процент", "процента", "процентов")
    )
}

/// NBU daily rates JSON (`bank.gov.ua/NBUStatService/v1/statdirectory/exchange?json`,
/// hryvnias per 1 unit) → «Доллар — 41 гривна 25 копеек, сэр».
pub fn rate_phrase(json: &str, code: &str) -> Option<String> {
    let code = code.to_uppercase();
    let list: Vec<serde_json::Value> = serde_json::from_str(json).ok()?;
    let rate = list
        .iter()
        .find(|v| v["cc"].as_str() == Some(code.as_str()))?["rate"]
        .as_f64()?;
    let kop_total = (rate * 100.0).round() as i64;
    let (uah, kop) = (kop_total / 100, kop_total % 100);
    let name = match code.as_str() {
        "USD" => "Доллар",
        "EUR" => "Евро",
        "CNY" => "Юань",
        "PLN" => "Злотый",
        other => other,
    };
    Some(format!(
        "{name} — {uah} {} {kop} {}, сэр",
        plural(uah, "гривна", "гривны", "гривен"),
        plural(kop, "копейка", "копейки", "копеек")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_from_nbu_json() {
        let json = r#"[{"r030":840,"txt":"Долар США","rate":41.2534,"cc":"USD"},
            {"r030":985,"txt":"Злотий","rate":11.05,"cc":"PLN"}]"#;
        assert_eq!(
            rate_phrase(json, "usd").as_deref(),
            Some("Доллар — 41 гривна 25 копеек, сэр")
        );
        assert_eq!(
            rate_phrase(json, "PLN").as_deref(),
            Some("Злотый — 11 гривен 5 копеек, сэр")
        );
        assert_eq!(rate_phrase(json, "EUR"), None);
    }

    #[test]
    fn plurals() {
        for (n, w) in [
            (1, "час"),
            (2, "часа"),
            (5, "часов"),
            (11, "часов"),
            (21, "час"),
            (22, "часа"),
            (14, "часов"),
            (0, "часов"),
        ] {
            assert_eq!(plural(n, "час", "часа", "часов"), w, "{n}");
        }
    }

    #[test]
    fn greetings_by_hour() {
        assert_eq!(greeting_phrase(7), "Доброе утро, сэр");
        assert_eq!(greet_category(7), "greet_morning");
        assert_eq!(greet_category(23), "greet_night");
        assert_eq!(greeting_phrase(13), "Добрый день, сэр");
        assert_eq!(greeting_phrase(20), "Добрый вечер, сэр");
        assert_eq!(greeting_phrase(2), "Доброй ночи, сэр");
    }

    #[test]
    fn phrases() {
        assert_eq!(time_phrase(14, 5), "Сейчас 14 часов 5 минут, сэр");
        assert_eq!(time_phrase(21, 0), "Сейчас ровно 21 час, сэр");
        assert_eq!(time_phrase(3, 22), "Сейчас 3 часа 22 минуты, сэр");
        assert_eq!(
            date_phrase(27, 9, 0),
            "Сегодня воскресенье, 27 сентября, сэр"
        );
        assert_eq!(
            battery_phrase(Some(81), true),
            "Заряд батареи 81 процент, идёт зарядка"
        );
        assert_eq!(
            battery_phrase(None, false),
            "Сэр, батарея не обнаружена, питание от сети"
        );
        assert_eq!(
            load_phrase(12, 73),
            "Загрузка процессора 12 процентов, памяти 73 процента"
        );
    }
}
