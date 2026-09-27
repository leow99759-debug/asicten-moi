//! Voice output level 2 (SPEC §6.2): Piper/VITS via sherpa-onnx for any text.
//! Loaded on first use, unloaded after idle; long text is synthesized sentence by sentence
//! so playback starts after the first one. Optional «movie» FX: EQ + short room reverb.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig};

use crate::{Error, Result};

/// Fallback voice until the Jarvis model is trained (T120), folder inside `assets/`.
pub const DEFAULT_MODEL_DIR: &str = "vits-piper-ru_RU-denis-medium";

struct Loaded {
    tts: OfflineTts,
    used: Instant,
}

pub struct Tts {
    dir: PathBuf,
    loaded: Mutex<Option<Loaded>>,
}

fn find_onnx(dir: &Path) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "onnx"))
}

impl Tts {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            loaded: Mutex::new(None),
        }
    }

    fn load(&self) -> Result<OfflineTts> {
        let s = |p: PathBuf| Some(p.to_string_lossy().into_owned());
        let model = find_onnx(&self.dir)
            .ok_or_else(|| Error::Model(format!("no .onnx in {}", self.dir.display())))?;
        let mut c = OfflineTtsConfig::default();
        c.model.vits.model = s(model);
        c.model.vits.tokens = s(self.dir.join("tokens.txt"));
        let espeak = self.dir.join("espeak-ng-data");
        if espeak.exists() {
            c.model.vits.data_dir = s(espeak);
        }
        c.model.num_threads = 1;
        c.max_num_sentences = 1;
        OfflineTts::create(&c)
            .ok_or_else(|| Error::Model(format!("TTS init failed: {}", self.dir.display())))
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.lock().map(|l| l.is_some()).unwrap_or(false)
    }

    /// Synthesize one chunk of text → (mono samples, sample rate). `speed` 1.0 = normal.
    pub fn synth(&self, text: &str, speed: f32) -> Result<(Vec<f32>, u32)> {
        let mut guard = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            let t = Instant::now();
            *guard = Some(Loaded {
                tts: self.load()?,
                used: Instant::now(),
            });
            tracing::info!(ms = t.elapsed().as_millis() as u64, "tts loaded");
        }
        let l = guard.as_mut().ok_or_else(|| Error::Model("tts".into()))?;
        l.used = Instant::now();
        let cfg = GenerationConfig {
            speed: speed.clamp(0.5, 2.0),
            ..GenerationConfig::default()
        };
        let audio = l
            .tts
            .generate_with_config(text, &cfg, None::<fn(&[f32], f32) -> bool>)
            .ok_or_else(|| Error::Model("tts generate failed".into()))?;
        Ok((audio.samples().to_vec(), audio.sample_rate().max(1) as u32))
    }

    /// Free the model when unused for `idle` (0 MB at rest, SPEC §12).
    pub fn unload_if_idle(&self, idle: Duration) {
        let mut guard = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        if guard.as_ref().is_some_and(|l| l.used.elapsed() >= idle) {
            *guard = None;
            tracing::info!("tts unloaded");
        }
    }
}

/// Split text into sentences for streaming synthesis (keeps the punctuation).
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        cur.push(ch);
        if matches!(ch, '.' | '!' | '?' | '…' | ';' | '\n') {
            let s = cur.trim();
            if s.chars().any(char::is_alphanumeric) {
                out.push(s.to_owned());
            }
            cur.clear();
        }
    }
    let s = cur.trim();
    if s.chars().any(char::is_alphanumeric) {
        out.push(s.to_owned());
    }
    out
}

/// Biquad filter (RBJ cookbook), direct form I.
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    x: [f32; 2],
    y: [f32; 2],
}

impl Biquad {
    fn highpass(rate: f32, f0: f32, q: f32) -> Self {
        let w = std::f32::consts::TAU * f0 / rate;
        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * q);
        let a0 = 1.0 + alpha;
        Self::norm(
            [(1.0 + cos) / 2.0, -(1.0 + cos), (1.0 + cos) / 2.0],
            [a0, -2.0 * cos, 1.0 - alpha],
        )
    }

    fn peak(rate: f32, f0: f32, q: f32, gain_db: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w = std::f32::consts::TAU * f0 / rate;
        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * q);
        Self::norm(
            [1.0 + alpha * a, -2.0 * cos, 1.0 - alpha * a],
            [1.0 + alpha / a, -2.0 * cos, 1.0 - alpha / a],
        )
    }

    fn norm(b: [f32; 3], a: [f32; 3]) -> Self {
        Self {
            b: [b[0] / a[0], b[1] / a[0], b[2] / a[0]],
            a: [a[1] / a[0], a[2] / a[0]],
            x: [0.0; 2],
            y: [0.0; 2],
        }
    }

    fn run(&mut self, x: f32) -> f32 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

/// «Как в фильме»: cut lows, lift presence, add a short metallic room (§6.2).
pub fn movie_fx(samples: &[f32], rate: u32) -> Vec<f32> {
    let r = rate as f32;
    let mut hp = Biquad::highpass(r, 150.0, 0.707);
    let mut presence = Biquad::peak(r, 3000.0, 1.0, 4.0);
    let dry: Vec<f32> = samples.iter().map(|&x| presence.run(hp.run(x))).collect();
    // two feedback combs (≈23/37 ms) give a small, slightly metallic space
    let tail = (r * 0.25) as usize;
    let mut out = vec![0.0f32; dry.len() + tail];
    out[..dry.len()].copy_from_slice(&dry);
    for (ms, fb) in [(23.0, 0.35f32), (37.0, 0.25)] {
        let d = (r * ms / 1000.0) as usize;
        let mut buf = vec![0.0f32; out.len()];
        for i in 0..out.len() {
            let input = dry.get(i).copied().unwrap_or(0.0);
            let prev = if i >= d { buf[i - d] } else { 0.0 };
            buf[i] = input + prev * fb;
        }
        for (o, b) in out.iter_mut().zip(&buf) {
            *o += b * 0.18;
        }
    }
    let peak = out.iter().fold(0.0f32, |m, x| m.max(x.abs()));
    if peak > 0.98 {
        let g = 0.98 / peak;
        out.iter_mut().for_each(|x| *x *= g);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_sentences() {
        assert_eq!(
            sentences("Доброе утро, сэр. Сейчас 7:30! Погода: ясно…  "),
            vec!["Доброе утро, сэр.", "Сейчас 7:30!", "Погода: ясно…"]
        );
        assert_eq!(sentences("да сэр"), vec!["да сэр"]);
        assert!(sentences(" ... ").is_empty());
    }

    #[test]
    fn fx_is_finite_bounded_and_adds_tail() {
        let rate = 22_050;
        let x: Vec<f32> = (0..rate).map(|i| (i as f32 * 0.05).sin() * 0.9).collect();
        let y = movie_fx(&x, rate as u32);
        assert!(y.len() > x.len());
        assert!(y.iter().all(|s| s.is_finite() && s.abs() <= 0.99));
        assert!(y[x.len()..].iter().any(|s| s.abs() > 1e-4), "reverb tail");
    }

    /// Real model round trip: synth → STT hears the words.
    #[test]
    fn piper_speaks_russian() {
        let (Some(tts_dir), Some(stt_dir)) = (
            crate::test_util::asset(DEFAULT_MODEL_DIR),
            crate::test_util::asset(crate::stt::MODEL_DIR),
        ) else {
            return;
        };
        let tts = Tts::new(tts_dir);
        let (s, rate) = tts
            .synth("Добрый день, сэр. Все системы работают.", 1.0)
            .expect("synth");
        assert!(tts.is_loaded());
        assert!(s.len() as f32 / rate as f32 > 1.0);
        let pcm16: Vec<i16> = crate::voice::resample(&s, rate, crate::audio::SAMPLE_RATE)
            .iter()
            .map(|x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)
            .collect();
        let mut stt = crate::stt::Stt::load(&stt_dir).expect("stt");
        stt.accept(&pcm16);
        stt.accept(&[0i16; 16_000]);
        let heard = stt.finish();
        assert!(
            heard.contains("добрый") && heard.contains("систем"),
            "{heard}"
        );
        tts.unload_if_idle(Duration::ZERO);
        assert!(!tts.is_loaded());
    }
}
