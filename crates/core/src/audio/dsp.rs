//! Pure DSP helpers, platform-independent and unit-tested.

use std::collections::VecDeque;

/// Pipeline sample rate for VAD / wake word / STT.
pub const SAMPLE_RATE: u32 = 16_000;

/// Streaming converter: interleaved f32 at any rate/channel count → 16 kHz mono i16.
/// Downmix = channel average; resample = box filter (average of the input samples that
/// fall into each output period), which doubles as a cheap anti-alias low-pass.
// ponytail: box filter, swap for a windowed-sinc (rubato) if STT accuracy suffers.
pub struct Converter {
    channels: usize,
    ratio: f64,
    phase: f64,
    acc: f32,
    n: u32,
    last: f32,
}

impl Converter {
    pub fn new(in_rate: u32, channels: u16) -> Self {
        Self {
            channels: usize::from(channels.max(1)),
            ratio: f64::from(in_rate) / f64::from(SAMPLE_RATE),
            phase: 0.0,
            acc: 0.0,
            n: 0,
            last: 0.0,
        }
    }

    /// Convert one device buffer, appending to `out`.
    pub fn push(&mut self, interleaved: &[f32], out: &mut Vec<i16>) {
        for frame in interleaved.chunks_exact(self.channels) {
            self.acc += frame.iter().sum::<f32>() / self.channels as f32;
            self.n += 1;
            self.phase += 1.0;
            while self.phase >= self.ratio {
                if self.n > 0 {
                    self.last = self.acc / self.n as f32;
                    self.acc = 0.0;
                    self.n = 0;
                }
                out.push(to_i16(self.last));
                self.phase -= self.ratio;
            }
        }
    }
}

fn to_i16(x: f32) -> i16 {
    (x.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

/// Loudness 0.0–1.0 for the UI: RMS in dBFS mapped linearly from -60 dB (0) to 0 dB (1).
pub fn level(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| f64::from(s).powi(2)).sum();
    let rms = (sum / samples.len() as f64).sqrt() / f64::from(i16::MAX);
    if rms <= 0.0 {
        return 0.0;
    }
    ((20.0 * rms.log10() + 60.0) / 60.0).clamp(0.0, 1.0) as f32
}

/// Fixed-size history of the most recent samples (pre-roll before wake word / STT load).
pub struct Ring {
    buf: VecDeque<i16>,
    cap: usize,
}

impl Ring {
    pub fn with_seconds(secs: f32) -> Self {
        let cap = (secs.max(0.0) * SAMPLE_RATE as f32) as usize;
        Self {
            buf: VecDeque::with_capacity(cap),
            cap,
        }
    }

    /// Append, dropping the oldest samples beyond capacity.
    pub fn push(&mut self, samples: &[i16]) {
        let tail = &samples[samples.len().saturating_sub(self.cap)..];
        let overflow = (self.buf.len() + tail.len()).saturating_sub(self.cap);
        self.buf.drain(..overflow);
        self.buf.extend(tail);
    }

    /// Take everything buffered, oldest first.
    pub fn take(&mut self) -> Vec<i16> {
        self.buf.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(rate: u32, channels: u16, secs: f32, hz: f32) -> Vec<f32> {
        let frames = (rate as f32 * secs) as usize;
        (0..frames)
            .flat_map(|i| {
                let v = (i as f32 / rate as f32 * hz * std::f32::consts::TAU).sin() * 0.5;
                std::iter::repeat_n(v, channels as usize)
            })
            .collect()
    }

    #[test]
    fn converts_common_rates_to_16k_mono() {
        for (rate, ch) in [
            (48_000, 2),
            (44_100, 2),
            (16_000, 1),
            (8_000, 1),
            (96_000, 4),
        ] {
            let mut c = Converter::new(rate, ch);
            let mut out = Vec::new();
            // feed in odd-sized chunks to exercise streaming state
            for chunk in sine(rate, ch, 1.0, 440.0).chunks(ch as usize * 333) {
                c.push(chunk, &mut out);
            }
            let diff = (out.len() as i64 - 16_000).abs();
            assert!(diff <= 1, "{rate}/{ch}: {} samples", out.len());
            let peak = out.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
            assert!((14_000..=17_000).contains(&peak), "{rate}: peak {peak}");
        }
    }

    #[test]
    fn downmix_cancels_opposite_channels() {
        let mut c = Converter::new(16_000, 2);
        let mut out = Vec::new();
        c.push(&[0.5, -0.5, 0.25, -0.25], &mut out);
        assert_eq!(out, vec![0, 0]);
    }

    #[test]
    fn level_maps_db() {
        assert_eq!(level(&[]), 0.0);
        assert_eq!(level(&[0; 100]), 0.0);
        assert!((level(&[i16::MAX; 100]) - 1.0).abs() < 1e-3);
        let quiet = level(&[33; 100]); // ≈ -60 dBFS
        assert!(quiet < 0.02, "{quiet}");
    }

    #[test]
    fn ring_keeps_latest() {
        let mut r = Ring::with_seconds(0.0005); // 8 samples
        r.push(&[1, 2, 3, 4, 5]);
        r.push(&[6, 7, 8, 9, 10]);
        assert_eq!(r.len(), 8);
        assert_eq!(r.take(), vec![3, 4, 5, 6, 7, 8, 9, 10]);
        assert!(r.is_empty());
        r.push(&(0..20).collect::<Vec<_>>());
        assert_eq!(r.take(), (12..20).collect::<Vec<_>>());
    }
}
