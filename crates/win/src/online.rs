//! Online services over the built-in curl.exe (Windows 10 1803+): news digest (SPEC §8)
//! with an optional Gemini summary, ElevenLabs / Fish Audio speech. Keys go in temp header files, not argv.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use jarvis_core::config::{CloudVoice, Online};
use jarvis_core::news;

use crate::backend::run_hidden;

/// Feeds and prices: SPEC network timeout. The LLM/TTS reply needs longer (≤15 s).
const FETCH_SEC: &str = "4";
const SLOW_SEC: &str = "15";
/// Tried after the configured model, in case Google retires it.
const MODEL_ALIAS: &str = "gemini-flash-lite-latest";
/// Both voices return raw 16-bit mono PCM at this rate.
pub const CLOUD_RATE: u32 = 24_000;

fn get(url: &str) -> Option<String> {
    run_hidden("curl", &["-sfL", "-m", FETCH_SEC, "-A", "Mozilla/5.0", url])
        .map_err(|e| tracing::warn!(%url, "fetch: {e}"))
        .ok()
}

/// Temp file removed on drop.
pub(crate) struct Temp(pub(crate) PathBuf);

impl Temp {
    pub(crate) fn new(ext: &str, data: &[u8]) -> Result<Self, String> {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "jarvis-{}-{}.{ext}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&p, data).map_err(|e| e.to_string())?;
        Ok(Self(p))
    }

    pub(crate) fn at(&self) -> String {
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

/// Text in the online voice: mono samples at [`CLOUD_RATE`].
pub fn cloud_tts(voice: &CloudVoice, text: &str) -> Result<Vec<f32>, String> {
    let out = Temp::new("pcm", b"")?;
    match voice {
        CloudVoice::Eleven { key, voice } => post(
            &format!(
                "https://api.elevenlabs.io/v1/text-to-speech/{voice}?output_format=pcm_{CLOUD_RATE}"
            ),
            &format!("xi-api-key: {key}"),
            &serde_json::json!({"text": text, "model_id": "eleven_multilingual_v2"}).to_string(),
            Some(&out),
        ),
        CloudVoice::Fish { key, voice } => post(
            "https://api.fish.audio/v1/tts",
            &format!("Authorization: Bearer {key}\nmodel: s2.1-pro-free"),
            &serde_json::json!({
                "text": text, "reference_id": voice, "format": "pcm", "sample_rate": CLOUD_RATE,
            })
            .to_string(),
            Some(&out),
        ),
    }
    .map_err(|e| explain(&e))?;
    let pcm = std::fs::read(&out.0).map_err(|e| e.to_string())?;
    if pcm.len() < 2 {
        return Err("голос: пустой ответ".into());
    }
    Ok(pcm
        .chunks_exact(2)
        .map(|b| f32::from(i16::from_le_bytes([b[0], b[1]])) / 32768.0)
        .collect())
}

/// curl's error for a voice request → what the user should fix.
fn explain(e: &str) -> String {
    let why = if e.contains("error: 401") || e.contains("error: 403") {
        "ключ не подходит — проверьте его в Настройки → ИИ"
    } else if e.contains("error: 402") {
        "на аккаунте закончились кредиты (баланс API)"
    } else if e.contains("error: 429") {
        "слишком много запросов, лимит"
    } else if e.contains("(28)") {
        "сервер не ответил за 15 с"
    } else if e.contains("(6)") || e.contains("(7)") {
        "нет интернета"
    } else {
        return format!("голос онлайн: {e}");
    };
    format!("голос онлайн: {why} [{e}]")
}

#[cfg(test)]
mod tests {
    use super::explain;

    #[test]
    fn voice_errors_are_readable() {
        assert!(explain("curl: (22) The requested URL returned error: 402").contains("кредиты"));
        assert!(explain("curl: (22) The requested URL returned error: 401").contains("ключ"));
        assert!(explain("curl: (6) Could not resolve host").contains("интернета"));
    }
}
