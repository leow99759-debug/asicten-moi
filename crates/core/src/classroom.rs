//! Google Classroom homework («какие задания на завтра»): OAuth URLs/bodies, API JSON
//! parsing and the spoken answer. Pure, tested; HTTP lives in jarvis-win.

use std::collections::HashSet;

use serde_json::Value;

use crate::info::plural;

pub const SCOPES: &str = "https://www.googleapis.com/auth/classroom.courses.readonly https://www.googleapis.com/auth/classroom.coursework.me.readonly";
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const API: &str = "https://classroom.googleapis.com/v1";
pub const NOT_CONNECTED: &str =
    "Сэр, Google Classroom не подключён. Подключите его в настройках, на вкладке ИИ.";
pub const REVOKED: &str =
    "Сэр, доступ к Google Classroom отозван или истёк. Подключите его заново в настройках.";
/// Tasks named per day, the rest are counted.
const PER_DAY: usize = 6;
const DAY: i64 = 1440;

pub fn pct(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn unpct(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = s
            .get(i + 1..i + 3)
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (b[i], hex) {
            (b'%', Some(v)) => {
                out.push(v);
                i += 2;
            }
            (b'+', _) => out.push(b' '),
            (c, _) => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Browser consent page; `redirect` = loopback `http://127.0.0.1:<port>`, PKCE plain.
pub fn auth_url(client_id: &str, redirect: &str, state: &str, verifier: &str) -> String {
    format!(
        "{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent&state={state}&code_challenge={verifier}&code_challenge_method=plain",
        pct(client_id),
        pct(redirect),
        pct(SCOPES)
    )
}

pub fn code_body(id: &str, secret: &str, code: &str, verifier: &str, redirect: &str) -> String {
    format!(
        "grant_type=authorization_code&client_id={}&client_secret={}&code={}&code_verifier={verifier}&redirect_uri={}",
        pct(id),
        pct(secret),
        pct(code),
        pct(redirect)
    )
}

pub fn refresh_body(id: &str, secret: &str, refresh: &str) -> String {
    format!(
        "grant_type=refresh_token&client_id={}&client_secret={}&refresh_token={}",
        pct(id),
        pct(secret),
        pct(refresh)
    )
}

/// First line of the loopback request (`GET /?code=…&state=… HTTP/1.1`) → code.
/// `Ok(None)` = not the redirect (favicon etc.).
pub fn redirect_code(request_line: &str, state: &str) -> Result<Option<String>, String> {
    let path = request_line.split_whitespace().nth(1).unwrap_or("");
    let Some(query) = path.strip_prefix("/?") else {
        return Ok(None);
    };
    let get = |k: &str| {
        query
            .split('&')
            .find_map(|p| p.strip_prefix(k)?.strip_prefix('='))
            .map(unpct)
    };
    if let Some(e) = get("error") {
        return Err(format!("Google отказал в доступе ({e})"));
    }
    if get("state").as_deref() != Some(state) {
        return Err("неверный ответ авторизации".into());
    }
    get("code")
        .map(Some)
        .ok_or_else(|| "Google не вернул код".into())
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub access: String,
    pub refresh: Option<String>,
}

/// Token endpoint reply. `Err(REVOKED)` when the refresh token is dead.
pub fn token(json: &str) -> Result<Token, String> {
    let v: Value = serde_json::from_str(json).map_err(|_| format!("token: {json}"))?;
    if let Some(access) = v["access_token"].as_str() {
        return Ok(Token {
            access: access.to_owned(),
            refresh: v["refresh_token"].as_str().map(str::to_owned),
        });
    }
    if v["error"] == "invalid_grant" {
        return Err(REVOKED.into());
    }
    Err(api_error(&v).unwrap_or_else(|| format!("token: {json}")))
}

/// Google API error text, if the reply is an error.
pub fn api_error(v: &Value) -> Option<String> {
    let e = &v["error"];
    if e.is_null() {
        return None;
    }
    Some(
        e["message"]
            .as_str()
            .or(v["error_description"].as_str())
            .or(e.as_str())
            .unwrap_or("ошибка Google")
            .to_owned(),
    )
}

fn parse(json: &str) -> Result<Value, String> {
    let v: Value =
        serde_json::from_str(json).map_err(|_| "Classroom: неверный ответ".to_owned())?;
    match api_error(&v) {
        Some(e) => Err(format!("Classroom: {e}")),
        None => Ok(v),
    }
}

/// `courses.list` → (id, name).
pub fn courses(json: &str) -> Result<Vec<(String, String)>, String> {
    let v = parse(json)?;
    Ok(v["courses"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| Some((c["id"].as_str()?.to_owned(), c["name"].as_str()?.to_owned())))
        .collect())
}

/// `studentSubmissions.list` → course work ids already turned in.
pub fn done_ids(json: &str) -> Result<HashSet<String>, String> {
    let v = parse(json)?;
    Ok(v["studentSubmissions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|s| matches!(s["state"].as_str(), Some("TURNED_IN" | "RETURNED")))
        .filter_map(|s| s["courseWorkId"].as_str().map(str::to_owned))
        .collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Work {
    pub id: String,
    pub course: String,
    pub title: String,
    /// Due moment, minutes since the Unix epoch, UTC (Classroom stores UTC).
    pub due: i64,
}

/// `courseWork.list` of one course → works that have a due date.
pub fn works(course: &str, json: &str) -> Result<Vec<Work>, String> {
    let v = parse(json)?;
    Ok(v["courseWork"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|w| {
            let d = &w["dueDate"];
            let n = |x: &Value| x.as_i64();
            let day = days_from_civil(n(&d["year"])?, n(&d["month"])?, n(&d["day"])?);
            let t = &w["dueTime"];
            let min = n(&t["hours"]).unwrap_or(0) * 60 + n(&t["minutes"]).unwrap_or(0);
            Some(Work {
                id: w["id"].as_str()?.to_owned(),
                course: course.to_owned(),
                title: w["title"]
                    .as_str()
                    .unwrap_or("без названия")
                    .trim()
                    .to_owned(),
                due: day * DAY + min,
            })
        })
        .collect())
}

/// Days since 1970-01-01 (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `homework` = today…day after tomorrow, `homework:N` = only day N (0 = today).
pub fn range(what: &str) -> Option<(i64, i64)> {
    match what.strip_prefix("homework")? {
        "" => Some((0, 2)),
        n => n.strip_prefix(':')?.parse().ok().map(|n| (n, n)),
    }
}

fn day_name(d: i64) -> String {
    match d {
        0 => "сегодня".into(),
        1 => "завтра".into(),
        2 => "послезавтра".into(),
        n => format!("через {n} {}", plural(n, "день", "дня", "дней")),
    }
}

fn short(s: &str) -> String {
    let s: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    match s.char_indices().nth(80) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s,
    }
}

/// Spoken answer. `now` = UTC minutes since epoch, `offset` = local UTC offset in minutes.
pub fn phrase(
    works: &[Work],
    done: &HashSet<String>,
    now: i64,
    offset: i64,
    (from, to): (i64, i64),
) -> String {
    let today = (now + offset).div_euclid(DAY);
    let mut due: Vec<(i64, i64, &Work)> = Vec::new();
    let mut overdue = 0;
    for w in works.iter().filter(|w| !done.contains(&w.id)) {
        let local = w.due + offset;
        let d = local.div_euclid(DAY) - today;
        if (from..=to).contains(&d) {
            due.push((d, local.rem_euclid(DAY), w));
        } else if (-7..0).contains(&d) && from == 0 {
            overdue += 1;
        }
    }
    due.sort_by_key(|(d, t, w)| (*d, *t, w.course.clone()));
    let mut out = Vec::new();
    if due.is_empty() {
        out.push(if from == to {
            format!("На {} заданий нет, сэр.", day_name(from))
        } else {
            "На ближайшие три дня заданий нет, сэр.".to_owned()
        });
    }
    for d in from..=to {
        let day: Vec<_> = due.iter().filter(|(x, ..)| *x == d).collect();
        if day.is_empty() {
            continue;
        }
        let n = day.len() as i64;
        let mut items: Vec<String> = day
            .iter()
            .take(PER_DAY)
            .map(|(_, t, w)| {
                let at = if *t >= 23 * 60 + 59 || *t == 0 {
                    String::new()
                } else {
                    format!(" до {}:{:02}", t / 60, t % 60)
                };
                format!("{} — «{}»{at}", w.course.trim(), short(&w.title))
            })
            .collect();
        if day.len() > PER_DAY {
            items.push(format!("и ещё {}", day.len() - PER_DAY));
        }
        out.push(format!(
            "На {} {n} {}: {}.",
            day_name(d),
            plural(n, "задание", "задания", "заданий"),
            items.join("; ")
        ));
    }
    if overdue > 0 {
        out.push(format!(
            "И ещё {overdue} {} за последнюю неделю.",
            plural(
                overdue,
                "просроченное задание",
                "просроченных задания",
                "просроченных заданий"
            )
        ));
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: &str = r#"{"courseWork":[
        {"id":"1","title":"Упражнение 5","dueDate":{"year":2026,"month":9,"day":29},"dueTime":{"hours":20,"minutes":59}},
        {"id":"2","title":"Реферат","dueDate":{"year":2026,"month":9,"day":30},"dueTime":{"hours":15}},
        {"id":"3","title":"Старое","dueDate":{"year":2026,"month":9,"day":25},"dueTime":{"hours":20,"minutes":59}},
        {"id":"4","title":"Без срока"},
        {"id":"5","title":"Сдано","dueDate":{"year":2026,"month":9,"day":29},"dueTime":{"hours":10}}
    ]}"#;

    fn now() -> i64 {
        // 2026-09-28 13:00 UTC = 16:00 in Kyiv/Jerusalem (UTC+3)
        days_from_civil(2026, 9, 28) * DAY + 13 * 60
    }

    #[test]
    fn civil_days() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1), 11_017);
        assert_eq!(days_from_civil(2026, 9, 28), 20_724);
    }

    #[test]
    fn homework_phrase() {
        let w = works("Алгебра", WORK).expect("ok");
        assert_eq!(w.len(), 4);
        let done: HashSet<String> = ["5".to_owned()].into();
        let s = phrase(&w, &done, now(), 180, (0, 2));
        assert_eq!(
            s,
            "На завтра 1 задание: Алгебра — «Упражнение 5». \
             На послезавтра 1 задание: Алгебра — «Реферат» до 18:00. \
             И ещё 1 просроченное задание за последнюю неделю."
        );
        assert_eq!(
            phrase(&w, &done, now(), 180, (0, 0)),
            "На сегодня заданий нет, сэр. И ещё 1 просроченное задание за последнюю неделю."
        );
        assert_eq!(
            phrase(&[], &done, now(), 180, (0, 2)),
            "На ближайшие три дня заданий нет, сэр."
        );
        assert!(phrase(&w, &HashSet::new(), now(), 180, (1, 1)).starts_with("На завтра 2 задания"));
    }

    #[test]
    fn parsing() {
        assert_eq!(range("homework"), Some((0, 2)));
        assert_eq!(range("homework:1"), Some((1, 1)));
        assert_eq!(range("news"), None);
        let c = courses(r#"{"courses":[{"id":"7","name":"История"}]}"#).expect("ok");
        assert_eq!(c, vec![("7".into(), "История".into())]);
        let d = done_ids(
            r#"{"studentSubmissions":[{"courseWorkId":"1","state":"TURNED_IN"},{"courseWorkId":"2","state":"CREATED"}]}"#,
        )
        .expect("ok");
        assert!(d.contains("1") && !d.contains("2"));
        assert!(courses(r#"{"error":{"code":403,"message":"denied"}}"#)
            .expect_err("err")
            .contains("denied"));
        assert_eq!(
            token(r#"{"access_token":"a","refresh_token":"r"}"#).expect("ok"),
            Token {
                access: "a".into(),
                refresh: Some("r".into())
            }
        );
        assert_eq!(
            token(r#"{"error":"invalid_grant"}"#).expect_err("err"),
            REVOKED
        );
    }

    #[test]
    fn oauth() {
        let line = "GET /?state=s1&code=4%2F0Ab-x&scope=a+b HTTP/1.1";
        assert_eq!(
            redirect_code(line, "s1").expect("ok"),
            Some("4/0Ab-x".into())
        );
        assert!(redirect_code(line, "other").is_err());
        assert_eq!(
            redirect_code("GET /favicon.ico HTTP/1.1", "s1").expect("ok"),
            None
        );
        assert!(redirect_code("GET /?error=access_denied&state=s1 HTTP/1.1", "s1").is_err());
        let u = auth_url("id.apps", "http://127.0.0.1:5000", "s1", "v");
        assert!(u.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A5000"));
        assert!(u.contains("access_type=offline"));
        assert!(refresh_body("i", "s/", "r").contains("client_secret=s%2F"));
    }
}
