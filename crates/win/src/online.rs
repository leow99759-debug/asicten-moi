//! Online services over the built-in curl.exe (Windows 10 1803+): news digest (SPEC §8)
//! with an optional Gemini summary, Fish Audio speech. Keys go in temp header files, not argv.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use jarvis_core::config::Online;
use jarvis_core::news;

use crate::backend::run_hidden;

/// Feeds and prices: SPEC network timeout. The LLM/TTS reply needs longer (≤15 s).
const FETCH_SEC: &str = "4";
const SLOW_SEC: &str = "15";
/// Tried after the configured model, in case Google retires it.
const MODEL_ALIAS: &str = "gemini-flash-lite-latest";
pub const FISH_RATE: u32 = 24_000;

fn get(url: &str) -> Option<String> {
    run_hidden("curl", &["-sfL", "-m", FETCH_SEC, "-A", "Mozilla/5.0", url])
        .map_err(|e| tracing::warn!(%url, "fetch: {e}"))
        .ok()
}

/// Temp file removed on drop.
struct Temp(PathBuf);

impl Temp {
    fn new(ext: &str, data: &[u8]) -> Result<Self, String> {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "jarvis-{}-{}.{ext}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&p, data).map_err(|e| e.to_string())?;
        Ok(Self(p))
    }

    fn at(&self) -> String {
        format!("@{}", self.0.display())
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// POST JSON; `headers` = extra "Name: value" lines (secrets).
fn post(url: &str, headers: &str, body: &str, out: Option<&Temp>) -> Result<String, String> {
    let h = Temp::new(
        "h",
        format!("Content-Type: application/json\n{headers}\n").as_bytes(),
    )?;
    let b = Temp::new("json", body.as_bytes())?;
    let (ha, ba) = (h.at(), b.at());
    let mut args = vec!["-sS", "-m", SLOW_SEC, "-H", &ha, "--data-binary", &ba];
    let o = out.map(|t| t.0.display().to_string());
    if let Some(o) = &o {
        args.extend(["-f", "-o", o]);
    }
    args.push(url);
    run_hidden("curl", &args)
}

/// «Что нового сегодня»: all feeds in parallel, then Gemini (keys in turn), else headlines.
pub fn digest(cfg: &Online) -> Result<String, String> {
    let (sections, prices) = std::thread::scope(|s| {
        let feeds: Vec<_> = news::TOPICS
            .iter()
            .map(|(topic, q)| {
                s.spawn(move || {
                    let heads = get(&news::feed_url(*q))
                        .map(|x| news::titles(&x, 8))
                        .unwrap_or_default();
                    ((*topic).to_owned(), heads)
                })
            })
            .collect();
        let prices = s.spawn(|| get(news::PRICES_URL).map(|j| news::prices(&j)));
        (
            feeds
                .into_iter()
                .filter_map(|h| h.join().ok())
                .collect::<Vec<_>>(),
            prices.join().ok().flatten().unwrap_or_default(),
        )
    });
    if prices.is_empty() && sections.iter().all(|(_, h)| h.is_empty()) {
        return Err(jarvis_core::NO_INTERNET.to_owned());
    }
    let prompt = news::prompt(&sections, &prices);
    for key in cfg.gemini_keys() {
        for model in [cfg.gemini_model.trim(), MODEL_ALIAS] {
            match gemini(key, model, &prompt) {
                Ok(text) => return Ok(text),
                Err(e) => tracing::warn!(%model, "gemini: {e}"),
            }
        }
    }
    Ok(news::plain(&sections, &prices))
}

fn gemini(key: &str, model: &str, prompt: &str) -> Result<String, String> {
    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");
    let reply = post(
        &url,
        &format!("x-goog-api-key: {key}"),
        &news::gemini_body(prompt),
        None,
    )?;
    news::gemini_text(&reply)
}

/// Text in the Fish Audio Jarvis voice: mono samples at [`FISH_RATE`].
pub fn fish_tts(key: &str, voice: &str, text: &str) -> Result<Vec<f32>, String> {
    let body = serde_json::json!({
        "text": text, "reference_id": voice, "format": "pcm", "sample_rate": FISH_RATE,
    });
    let out = Temp::new("pcm", b"")?;
    post(
        "https://api.fish.audio/v1/tts",
        &format!("Authorization: Bearer {key}\nmodel: s2.1-pro-free"),
        &body.to_string(),
        Some(&out),
    )?;
    let pcm = std::fs::read(&out.0).map_err(|e| e.to_string())?;
    if pcm.len() < 2 {
        return Err("fish: пустой ответ".into());
    }
    Ok(pcm
        .chunks_exact(2)
        .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0)
        .collect())
}
