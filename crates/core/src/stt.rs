//! Command STT (SPEC §1, §2.1): streaming zipformer small-ru (Vosk team) via sherpa-onnx.
//! Loaded lazily on wake word; audio arriving while the model loads is buffered so no word
//! is lost; kept warm for `stt_keep_warm_sec`, then unloaded (0 MB when idle).

use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use sherpa_onnx::{OnlineRecognizer, OnlineRecognizerConfig, OnlineStream};

use crate::audio::SAMPLE_RATE;
use crate::{Error, Result};

/// Folder name inside `assets/` (tools/fetch-assets.ps1).
pub const MODEL_DIR: &str = "sherpa-onnx-streaming-zipformer-small-ru-vosk-int8-2025-08-16";

/// A loaded recognizer with one utterance stream.
pub struct Stt {
    rec: OnlineRecognizer,
    stream: OnlineStream,
    scratch: Vec<f32>,
    partial: String,
}

impl Stt {
    pub fn load(dir: &Path) -> Result<Self> {
        let file = |name: &str| Some(dir.join(name).to_string_lossy().into_owned());
        let mut c = OnlineRecognizerConfig::default();
        c.model_config.transducer.encoder = file("encoder.int8.onnx");
        c.model_config.transducer.decoder = file("decoder.onnx");
        c.model_config.transducer.joiner = file("joiner.int8.onnx");
        c.model_config.tokens = file("tokens.txt");
        c.model_config.num_threads = 1;
        c.decoding_method = Some("greedy_search".into());
        c.enable_endpoint = true;
        c.rule1_min_trailing_silence = 2.4; // nothing said
        c.rule2_min_trailing_silence = 0.6; // end of phrase
        c.rule3_min_utterance_length = 20.0;
        let rec = OnlineRecognizer::create(&c)
            .ok_or_else(|| Error::Model(format!("STT init failed: {}", dir.display())))?;
        let stream = rec.create_stream();
        Ok(Self {
            rec,
            stream,
            scratch: Vec::new(),
            partial: String::new(),
        })
    }

    /// Feed audio; returns the partial transcript when it changed.
    pub fn accept(&mut self, samples: &[i16]) -> Option<String> {
        self.scratch.clear();
        self.scratch
            .extend(samples.iter().map(|&s| f32::from(s) / 32768.0));
        self.stream
            .accept_waveform(SAMPLE_RATE as i32, &self.scratch);
        while self.rec.is_ready(&self.stream) {
            self.rec.decode(&self.stream);
        }
        let text = self.text();
        (text != self.partial).then(|| {
            self.partial.clone_from(&text);
            text
        })
    }

    /// Trailing silence after speech (or long nothing) detected.
    pub fn is_endpoint(&self) -> bool {
        self.rec.is_endpoint(&self.stream)
    }

    /// Close the utterance, return the final text and get ready for the next one.
    pub fn finish(&mut self) -> String {
        // pad so the last word leaves the model's right context
        self.accept(&[0; SAMPLE_RATE as usize / 2]);
        let text = self.text();
        self.reset();
        text
    }

    /// Drop the current utterance without a result.
    /// Fresh stream: `OnlineRecognizer::reset` keeps encoder context, which garbles the next start.
    pub fn reset(&mut self) {
        self.stream = self.rec.create_stream();
        self.partial.clear();
    }

    fn text(&self) -> String {
        self.rec
            .get_result(&self.stream)
            .map(|r| r.text.trim().to_lowercase())
            .unwrap_or_default()
    }
}

/// Lazy wrapper: load on demand in the background, buffer meanwhile, unload after idle.
pub struct LazyStt {
    dir: PathBuf,
    keep_warm: Duration,
    always_warm: bool,
    loaded: Option<Stt>,
    loading: Option<JoinHandle<Result<Stt>>>,
    buffer: Vec<i16>,
    last_used: Instant,
}

impl LazyStt {
    pub fn new(dir: PathBuf, keep_warm: Duration) -> Self {
        Self {
            dir,
            keep_warm,
            always_warm: false,
            loaded: None,
            loading: None,
            buffer: Vec::new(),
            last_used: Instant::now(),
        }
    }

    /// Keep the model resident (no-prefix mode / memory saver off, SPEC §2.2, §12).
    pub fn set_always_warm(&mut self, on: bool) {
        self.always_warm = on;
        if on {
            self.begin();
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded.is_some()
    }

    /// Start of an utterance (wake word): kick off loading if needed.
    pub fn begin(&mut self) {
        self.last_used = Instant::now();
        if self.loaded.is_none() && self.loading.is_none() {
            let dir = self.dir.clone();
            self.loading = Some(std::thread::spawn(move || Stt::load(&dir)));
        }
    }

    /// Feed audio; buffered until the model is ready. Returns partial text when changed.
    pub fn push(&mut self, samples: &[i16]) -> Result<Option<String>> {
        self.last_used = Instant::now();
        if self.loading.as_ref().is_some_and(JoinHandle::is_finished) {
            self.join()?;
        }
        match self.loaded.as_mut() {
            Some(stt) => Ok(stt.accept(samples)),
            None => {
                self.buffer.extend_from_slice(samples);
                Ok(None)
            }
        }
    }

    /// Discard buffered/decoded audio (new utterance starts now).
    pub fn reset(&mut self) {
        self.buffer.clear();
        if let Some(stt) = self.loaded.as_mut() {
            stt.reset();
        }
    }

    pub fn is_endpoint(&self) -> bool {
        self.loaded.as_ref().is_some_and(Stt::is_endpoint)
    }

    /// End of utterance: waits for a pending load, returns the final text.
    pub fn finish(&mut self) -> Result<String> {
        self.last_used = Instant::now();
        if self.loading.is_some() {
            self.join()?;
        }
        let stt = self
            .loaded
            .as_mut()
            .ok_or_else(|| Error::Model("STT not started".into()))?;
        Ok(stt.finish())
    }

    /// Call periodically: unloads after `keep_warm` of inactivity.
    pub fn tick(&mut self, now: Instant) {
        if !self.always_warm
            && self.loaded.is_some()
            && now.duration_since(self.last_used) >= self.keep_warm
        {
            self.loaded = None;
            tracing::info!("STT unloaded (idle)");
        }
    }

    fn join(&mut self) -> Result<()> {
        let Some(handle) = self.loading.take() else {
            return Ok(());
        };
        let mut stt = handle
            .join()
            .map_err(|_| Error::Model("STT loader panicked".into()))??;
        let buffered = std::mem::take(&mut self.buffer);
        stt.accept(&buffered);
        self.loaded = Some(stt);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{asset, fixture, read_wav_16k};

    fn model() -> Option<PathBuf> {
        let m = asset(MODEL_DIR);
        if m.is_none() {
            eprintln!("assets missing; skipping");
        }
        m
    }

    #[test]
    fn recognizes_command_with_partials() {
        let Some(dir) = model() else { return };
        let mut stt = Stt::load(&dir).expect("load");
        let audio = read_wav_16k(&fixture("wake/neg_browser.wav"));
        let partials: Vec<String> = audio.chunks(1600).filter_map(|c| stt.accept(c)).collect();
        assert!(!partials.is_empty(), "no partials");
        assert_eq!(stt.finish(), "открой браузер");
        // stream is reusable for the next utterance
        stt.accept(&read_wav_16k(&fixture("wake/neg_hello.wav")));
        assert!(stt.finish().starts_with("привет как"));
    }

    #[test]
    fn lazy_buffers_audio_while_loading_and_unloads() {
        let Some(dir) = model() else { return };
        let mut lazy = LazyStt::new(dir, Duration::from_secs(120));
        lazy.begin();
        // audio arrives immediately, long before the model is loaded
        for c in read_wav_16k(&fixture("wake/neg_browser.wav")).chunks(1600) {
            lazy.push(c).expect("push");
        }
        assert_eq!(lazy.finish().expect("finish"), "открой браузер");
        assert!(lazy.is_loaded());
        lazy.tick(Instant::now());
        assert!(lazy.is_loaded(), "still warm");
        lazy.tick(Instant::now() + Duration::from_secs(121));
        assert!(!lazy.is_loaded(), "unloaded after keep-warm");
        lazy.set_always_warm(true);
        lazy.finish().expect("reload");
        lazy.tick(Instant::now() + Duration::from_secs(10_000));
        assert!(lazy.is_loaded(), "always warm");
    }
}
