//! Silero VAD via sherpa-onnx (SPEC §1): splits the 16 kHz stream into speech segments.

use std::path::Path;

use sherpa_onnx::{SileroVadModelConfig, VadModelConfig, VoiceActivityDetector};

use crate::audio::SAMPLE_RATE;
use crate::{Error, Result};

/// End of phrase = this much silence (seconds).
const MIN_SILENCE_SEC: f32 = 0.4;
const MIN_SPEECH_SEC: f32 = 0.15;

pub struct Vad {
    inner: VoiceActivityDetector,
    scratch: Vec<f32>,
}

impl Vad {
    /// `model` = `silero_vad.onnx`; segments longer than `max_speech_sec` are cut.
    pub fn new(model: &Path, max_speech_sec: f32) -> Result<Self> {
        let config = VadModelConfig {
            silero_vad: SileroVadModelConfig {
                model: Some(model.to_string_lossy().into_owned()),
                threshold: 0.5,
                min_silence_duration: MIN_SILENCE_SEC,
                min_speech_duration: MIN_SPEECH_SEC,
                window_size: 512,
                max_speech_duration: max_speech_sec,
            },
            sample_rate: SAMPLE_RATE as i32,
            num_threads: 1,
            ..Default::default()
        };
        let inner = VoiceActivityDetector::create(&config, max_speech_sec + 2.0)
            .ok_or_else(|| Error::Model(format!("VAD init failed: {}", model.display())))?;
        Ok(Self {
            inner,
            scratch: Vec::new(),
        })
    }

    /// Feed 16 kHz mono samples; returns finished speech segments (usually none).
    pub fn push(&mut self, samples: &[i16]) -> Vec<Vec<i16>> {
        self.scratch.clear();
        self.scratch
            .extend(samples.iter().map(|&s| f32::from(s) / 32768.0));
        self.inner.accept_waveform(&self.scratch);
        self.drain()
    }

    /// Someone is speaking right now.
    pub fn is_speech(&self) -> bool {
        self.inner.detected()
    }

    /// Close the current segment (e.g. listen timeout) and return what's left.
    pub fn flush(&mut self) -> Vec<Vec<i16>> {
        self.inner.flush();
        let out = self.drain();
        self.inner.reset();
        out
    }

    fn drain(&mut self) -> Vec<Vec<i16>> {
        let mut out = Vec::new();
        while let Some(seg) = self.inner.front() {
            out.push(
                seg.samples()
                    .iter()
                    .map(|&x| (x.clamp(-1.0, 1.0) * 32767.0) as i16)
                    .collect(),
            );
            self.inner.pop();
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{asset, read_wav_16k};

    #[test]
    fn segments_speech_between_silence() {
        let (Some(model), Some(wav)) = (asset("silero_vad.onnx"), asset("jarvis-sound/16.wav"))
        else {
            eprintln!("assets missing, run tools/fetch-assets.ps1; skipping");
            return;
        };
        let speech = read_wav_16k(&wav);
        let silence = vec![0i16; SAMPLE_RATE as usize];
        let mut vad = Vad::new(&model, 10.0).expect("vad");
        let mut segs = Vec::new();
        for chunk in silence
            .iter()
            .chain(&speech)
            .chain(&silence)
            .copied()
            .collect::<Vec<_>>()
            .chunks(1600)
        {
            segs.extend(vad.push(chunk));
        }
        segs.extend(vad.flush());
        assert!(!segs.is_empty(), "no speech found");
        let total: usize = segs.iter().map(Vec::len).sum();
        assert!(
            total > SAMPLE_RATE as usize / 2,
            "speech too short: {total}"
        );
        assert!(
            total <= speech.len() + SAMPLE_RATE as usize,
            "silence leaked: {total}"
        );
    }

    #[test]
    fn silence_has_no_segments() {
        let Some(model) = asset("silero_vad.onnx") else {
            return;
        };
        let mut vad = Vad::new(&model, 10.0).expect("vad");
        let mut segs = vad.push(&vec![0i16; SAMPLE_RATE as usize * 3]);
        segs.extend(vad.flush());
        assert!(segs.is_empty());
        assert!(!vad.is_speech());
    }
}
