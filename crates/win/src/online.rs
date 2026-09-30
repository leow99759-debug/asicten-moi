//! Online services over the built-in curl.exe (Windows 10 1803+): news digest (SPEC §8)
//! with an optional Gemini summary, ElevenLabs / Fish Audio speech. Keys go in temp header files, not argv.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use jarvis_core::config::{CloudVoice, Online};
use jarvis_core::llm::{self, AiSlot};
use jarvis_core::news;

use crate::backend::run_hidden;

/// Feeds and prices: SPEC network timeout. The LLM/TTS reply needs longer.
const FETCH_SEC: &str = "4";
const SLOW_SEC: &str = "25";
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

/// «Что нового сегодня»: all feeds in parallel, then the first AI that answers, else headlines.
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
    match chat_chain(&cfg.ai_chain(), "", &prompt) {
        Ok(text) => Ok(text),
        Err(e) => {
            if !e.is_empty() {
                tracing::warn!("digest ai: {e}");
            }
            Ok(news::plain(&sections, &prices))
        }
    }
}

/// First model in the chain that answers; `Err("")` = nothing configured.
fn chat_chain(chain: &[AiSlot], system: &str, user: &str) -> Result<String, String> {
    let mut last = String::new();
    for slot in chain {
        match chat(slot, system, user) {
            Ok(t) => return Ok(t),
            Err(e) => {
                tracing::warn!(provider = %slot.provider, "ai: {e}");
                last = e;
            }
        }
    }
    Err(last)
}

/// One slot: its models in turn (a retired default falls through to the next).
pub fn chat(slot: &AiSlot, system: &str, user: &str) -> Result<String, String> {
    let mut last = "модель не указана".to_owned();
    for model in slot.models() {
        let Some(r) = slot.request(&model, system, user) else {
            return Err(format!("неизвестная нейросеть «{}»", slot.provider));
        };
        let reply = post(&r.url, &r.headers, &r.body, None).map_err(|e| {
            if e.contains("(6)") || e.contains("(7)") {
                jarvis_core::NO_INTERNET.to_owned()
            } else {
                e
            }
        })?;
        match llm::reply_text(r.wire, &reply) {
            Ok(t) => return Ok(t),
            Err(e) => last = format!("{model}: {e}"),
        }
    }
    Err(last)
}

/// Last questions and answers, so «а почему?» has context. Forgotten after 10 minutes.
type Dialog = (Vec<(String, String)>, Option<Instant>);
static DIALOG: Mutex<Dialog> = Mutex::new((Vec::new(), None));

/// «Джарвис, <любой вопрос>»: the first model that answers, in the Jarvis manner.
pub fn ask(cfg: &Online, question: &str) -> Result<String, String> {
    let chain = cfg.ai_chain();
    if chain.is_empty() {
        return Err("для ответов на вопросы добавьте нейросеть в Настройки → ИИ".into());
    }
    let mut d = DIALOG.lock().unwrap_or_else(|e| e.into_inner());
    if d.1.is_some_and(|t| t.elapsed() > Duration::from_secs(600)) {
        d.0.clear();
    }
    let mut user = String::new();
    if !d.0.is_empty() {
        user.push_str("Предыдущий разговор:\n");
        for (q, a) in &d.0 {
            user.push_str(&format!("Сэр: {q}\nДжарвис: {a}\n"));
        }
        user.push_str("\nНовый вопрос: ");
    }
    user.push_str(question);
    drop(d);
    let answer = chat_chain(&chain, llm::ASK_SYSTEM, &user)?;
    let mut d = DIALOG.lock().unwrap_or_else(|e| e.into_inner());
    d.0.push((question.to_owned(), answer.clone()));
    if d.0.len() > 4 {
        d.0.remove(0);
    }
    d.1 = Some(Instant::now());
    Ok(answer)
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
        "сервер не ответил вовремя"
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
