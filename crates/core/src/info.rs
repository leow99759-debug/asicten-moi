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

#[cfg(test)]
mod tests {
    use super::*;

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
