//! Google Classroom over curl.exe: one-time OAuth (loopback redirect, SPEC-style desktop
//! flow) and «какие задания на завтра». Secrets go in temp files, not argv.

use std::collections::HashSet;
use std::hash::BuildHasher;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

use jarvis_core::classroom as gc;
use jarvis_core::config::Online;

use crate::backend::run_hidden;
use crate::online::Temp;

const FETCH_SEC: &str = "4";
/// Consent in the browser (login, 2FA) may take a while.
const CONNECT_WAIT: Duration = Duration::from_secs(180);

fn random_hex(words: usize) -> String {
    (0..words)
        .map(|i| format!("{:016x}", std::hash::RandomState::new().hash_one(i)))
        .collect()
}

/// Transport failure (curl exit ≠ 0) = no network; Google errors come back as JSON.
fn curl(args: &[&str]) -> Result<String, String> {
    run_hidden("curl", args).map_err(|e| {
        tracing::warn!("classroom curl: {e}");
        jarvis_core::NO_INTERNET.to_owned()
    })
}

fn token_request(body: &str) -> Result<gc::Token, String> {
    let b = Temp::new("form", body.as_bytes())?;
    let ba = b.at();
    let json = curl(&["-sS", "-m", "15", "--data-binary", &ba, gc::TOKEN_URL])?;
    gc::token(&json)
}

fn api_get(access: &str, path: &str) -> Result<String, String> {
    let h = Temp::new("h", format!("Authorization: Bearer {access}\n").as_bytes())?;
    let ha = h.at();
    curl(&[
        "-sS",
        "-m",
        FETCH_SEC,
        "-H",
        &ha,
        &format!("{}/{path}", gc::API),
    ])
}

/// Browser consent → refresh token. Blocks until the redirect arrives or 3 minutes pass.
pub fn connect(id: &str, secret: &str) -> Result<String, String> {
    let (id, secret) = (id.trim(), secret.trim());
    if id.is_empty() || secret.is_empty() {
        return Err("введите Client ID и Client secret".into());
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let redirect = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().map_err(|e| e.to_string())?.port()
    );
    let (state, verifier) = (random_hex(1), random_hex(4));
    crate::backend::shell_open(
        &gc::auth_url(id, &redirect, &state, &verifier),
        "",
        None,
        false,
    )?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + CONNECT_WAIT;
    let code = loop {
        let (mut stream, _) = match listener.accept() {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() > deadline {
                    return Err("время ожидания входа в Google истекло".into());
                }
                std::thread::sleep(Duration::from_millis(200));
                continue;
            }
            Err(e) => return Err(e.to_string()),
        };
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut line = String::new();
        let _ = BufReader::new(&stream).read_line(&mut line);
        let got = gc::redirect_code(&line, &state);
        let page = match &got {
            Ok(Some(_)) => "Джарвис подключён к Google Classroom. Вкладку можно закрыть.",
            Ok(None) => "",
            Err(e) => e.as_str(),
        };
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<!doctype html><meta charset=utf-8><body style=\"font:20px system-ui;background:#0b1220;color:#e6edf7;display:grid;place-items:center;height:90vh\">{page}</body>"
        );
        match got {
            Ok(Some(code)) => break code,
            Ok(None) => continue,
            Err(e) => return Err(e),
        }
    };
    let t = token_request(&gc::code_body(id, secret, &code, &verifier, &redirect))?;
    t.refresh
        .ok_or_else(|| "Google не выдал постоянный доступ, попробуйте ещё раз".into())
}

/// Local UTC offset in minutes (Classroom due dates are UTC).
fn utc_offset() -> i64 {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::SYSTEMTIME;
        use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemTime};
        let m = |t: SYSTEMTIME| {
            gc::days_from_civil(t.wYear.into(), t.wMonth.into(), t.wDay.into()) * 1440
                + i64::from(t.wHour) * 60
                + i64::from(t.wMinute)
        };
        // SAFETY: neither call has a failure mode.
        let (l, u) = unsafe { (GetLocalTime(), GetSystemTime()) };
        // round to 15 min: the two reads may straddle a minute
        ((m(l) - m(u)) as f64 / 15.0).round() as i64 * 15
    }
    #[cfg(not(windows))]
    {
        0
    }
}

/// «Какие задания на завтра»: `what` = `homework` or `homework:N`.
pub fn homework(cfg: &Online, what: &str) -> Result<String, String> {
    let days = gc::range(what).ok_or("неизвестный запрос")?;
    let refresh = cfg.classroom_token.trim();
    if refresh.is_empty() {
        return Ok(gc::NOT_CONNECTED.into());
    }
    let access = match token_request(&gc::refresh_body(
        cfg.classroom_id.trim(),
        cfg.classroom_secret.trim(),
        refresh,
    )) {
        Ok(t) => t.access,
        Err(e) if e == gc::REVOKED => return Ok(e),
        Err(e) => return Err(e),
    };
    let courses = gc::courses(&api_get(
        &access,
        "courses?studentId=me&courseStates=ACTIVE&pageSize=50",
    )?)?;
    let (mut works, mut done) = (Vec::new(), HashSet::new());
    let per_course: Vec<_> = std::thread::scope(|s| {
        let jobs: Vec<_> = courses
            .iter()
            .map(|(id, name)| {
                let access = &access;
                s.spawn(move || -> Result<_, String> {
                    let w = gc::works(
                        name,
                        &api_get(access, &format!("courses/{id}/courseWork?pageSize=50"))?,
                    )?;
                    let d = gc::done_ids(&api_get(
                        access,
                        &format!(
                            "courses/{id}/courseWork/-/studentSubmissions?userId=me&pageSize=100"
                        ),
                    )?)?;
                    Ok((w, d))
                })
            })
            .collect();
        jobs.into_iter()
            .map(|j| j.join().unwrap_or_else(|_| Err("поток упал".into())))
            .collect()
    });
    for r in per_course {
        let (w, d) = r?;
        works.extend(w);
        done.extend(d);
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64
        / 60;
    Ok(gc::phrase(&works, &done, now, utc_offset(), days))
}
