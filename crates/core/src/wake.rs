//! Wake word «Джарвис» via Rustpotter + Priler's `jarvis-default.rpw` (SPEC §1, §2.2).
//! Detector settings adapted from github.com/Priler/jarvis (jarvis-core/src/config.rs).
//! Vosk grammar fallback lives with the STT (T013).

use std::path::Path;

use rustpotter::{Rustpotter, RustpotterConfig, SampleFormat, ScoreMode};

use crate::audio::SAMPLE_RATE;
use crate::{Error, Result};

pub struct WakeWord {
    rp: Rustpotter,
    config: RustpotterConfig,
    frame: usize,
    pending: Vec<i16>,
}

/// Slider 0–100 → detection threshold: 0 = strict (0.75), 50 = 0.5, 100 = loose (0.25).
pub fn threshold(sensitivity: u8) -> f32 {
    0.75 - f32::from(sensitivity.min(100)) / 200.0
}

impl WakeWord {
    pub fn new(rpw: &Path, sensitivity: u8) -> Result<Self> {
        let mut config = RustpotterConfig::default();
        config.fmt.sample_rate = SAMPLE_RATE as usize;
        config.fmt.sample_format = SampleFormat::I16;
        config.detector.threshold = threshold(sensitivity);
        config.detector.avg_threshold = 0.0;
        config.detector.min_scores = 15;
        config.detector.score_ref = 0.22;
        config.detector.band_size = 5;
        config.detector.score_mode = ScoreMode::Max;
        config.filters.gain_normalizer.enabled = false;
        config.filters.band_pass.enabled = true;
        config.filters.band_pass.low_cutoff = 80.0;
        config.filters.band_pass.high_cutoff = 400.0;
        let mut rp = Rustpotter::new(&config).map_err(Error::Model)?;
        rp.add_wakeword_from_file("jarvis", &rpw.to_string_lossy())
            .map_err(Error::Model)?;
        let frame = rp.get_samples_per_frame();
        Ok(Self {
            rp,
            config,
            frame,
            pending: Vec::with_capacity(frame * 2),
        })
    }

    pub fn set_sensitivity(&mut self, sensitivity: u8) {
        self.config.detector.threshold = threshold(sensitivity);
        self.rp.update_detector_config(&self.config.detector);
    }

    /// Feed 16 kHz mono samples; returns the score when «Джарвис» was detected.
    pub fn push(&mut self, samples: &[i16]) -> Option<f32> {
        self.pending.extend_from_slice(samples);
        let mut hit = None;
        let mut used = 0;
        while self.pending.len() - used >= self.frame {
            if let Some(d) = self
                .rp
                .process_samples(&self.pending[used..used + self.frame])
            {
                hit = Some(d.score);
            }
            used += self.frame;
        }
        self.pending.drain(..used);
        hit
    }

    pub fn reset(&mut self) {
        self.rp.reset();
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{asset, fixture, read_wav_16k};

    fn detect(ww: &mut WakeWord, name: &str) -> Option<f32> {
        ww.reset();
        let mut audio = vec![0i16; 8_000];
        audio.extend(read_wav_16k(&fixture(&format!("wake/{name}.wav"))));
        audio.extend(vec![0i16; 16_000]);
        audio.chunks(1600).filter_map(|c| ww.push(c)).last()
    }

    #[test]
    fn threshold_mapping() {
        assert_eq!(threshold(0), 0.75);
        assert_eq!(threshold(50), 0.5);
        assert_eq!(threshold(100), 0.25);
        assert_eq!(threshold(255), 0.25);
    }

    #[test]
    fn detects_jarvis_and_ignores_other_speech() {
        let Some(rpw) = asset("rustpotter-jarvis/jarvis-default.rpw") else {
            eprintln!("assets missing; skipping");
            return;
        };
        // Synthetic Piper voice scores lower than a real one; real-voice tuning = user check.
        let mut ww = WakeWord::new(&rpw, 50).expect("wake");
        assert!(detect(&mut ww, "jarvis").is_some(), "«Джарвис» missed");
        assert_eq!(detect(&mut ww, "neg_hello"), None);
        assert_eq!(detect(&mut ww, "neg_browser"), None);
        ww.set_sensitivity(70);
        assert!(
            detect(&mut ww, "jarvis_music").is_some(),
            "«Джарвис, включи музыку» missed"
        );
        assert_eq!(detect(&mut ww, "neg_browser"), None);
        ww.set_sensitivity(0);
        assert_eq!(
            detect(&mut ww, "jarvis"),
            None,
            "strict threshold should reject"
        );
    }
}
