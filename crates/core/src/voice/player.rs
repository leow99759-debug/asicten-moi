//! Speech output on the default device via cpal. The stream exists only while something
//! plays (opened on demand, closed after 1 s of silence) so idle CPU stays at zero.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::{read_wav, resample};
use crate::{Error, Result};

const IDLE_CLOSE: Duration = Duration::from_secs(1);

#[derive(Default)]
struct Shared {
    queue: Mutex<VecDeque<f32>>,
    /// f32 bits.
    volume: AtomicU32,
    /// f32 bits, RMS 0..1 of the last buffer (orb animation).
    level: AtomicU32,
}

enum Cmd {
    Wake,
}

pub struct Player {
    tx: Sender<Cmd>,
    shared: Arc<Shared>,
    rate: u32,
}

fn audio_err(e: impl std::fmt::Display) -> Error {
    Error::Audio(e.to_string())
}

impl Player {
    pub fn new() -> Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or_else(|| Error::Audio("no output device".into()))?;
        let rate = device
            .default_output_config()
            .map_err(audio_err)?
            .sample_rate();
        let shared = Arc::new(Shared::default());
        shared.volume.store(1.0_f32.to_bits(), Ordering::Relaxed);
        let (tx, rx) = mpsc::channel::<Cmd>();
        let sh = shared.clone();
        std::thread::Builder::new()
            .name("jarvis-speaker".into())
            .spawn(move || {
                while rx.recv().is_ok() {
                    let stream = match open(&sh) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::warn!("speaker: {e}");
                            continue;
                        }
                    };
                    let mut idle_since: Option<Instant> = None;
                    loop {
                        match rx.recv_timeout(Duration::from_millis(200)) {
                            Ok(_) => idle_since = None,
                            Err(RecvTimeoutError::Disconnected) => return,
                            Err(RecvTimeoutError::Timeout) => {}
                        }
                        let empty = sh.queue.lock().map(|q| q.is_empty()).unwrap_or(true);
                        if !empty {
                            idle_since = None;
                        } else if idle_since.get_or_insert_with(Instant::now).elapsed() > IDLE_CLOSE
                        {
                            break;
                        }
                    }
                    drop(stream);
                    sh.level.store(0, Ordering::Relaxed);
                }
            })?;
        Ok(Self { tx, shared, rate })
    }

    /// Queue mono samples (played after whatever is already queued).
    pub fn play_samples(&self, samples: &[f32], rate: u32) {
        let s = resample(samples, rate, self.rate);
        if let Ok(mut q) = self.shared.queue.lock() {
            q.extend(s);
        }
        let _ = self.tx.send(Cmd::Wake);
    }

    pub fn play_wav(&self, path: &std::path::Path) -> Result<()> {
        let (s, rate) = read_wav(path)?;
        self.play_samples(&s, rate);
        Ok(())
    }

    /// Drop everything queued (barge-in, «отмена»).
    pub fn stop(&self) {
        if let Ok(mut q) = self.shared.queue.lock() {
            q.clear();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.shared
            .queue
            .lock()
            .map(|q| !q.is_empty())
            .unwrap_or(false)
    }

    /// 0.0–1.0.
    pub fn set_volume(&self, v: f32) {
        self.shared
            .volume
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn level(&self) -> f32 {
        f32::from_bits(self.shared.level.load(Ordering::Relaxed))
    }
}

fn open(sh: &Arc<Shared>) -> Result<cpal::Stream> {
    let device = cpal::default_host()
        .default_output_device()
        .ok_or_else(|| Error::Audio("no output device".into()))?;
    let cfg = device.default_output_config().map_err(audio_err)?;
    let stream = match cfg.sample_format() {
        SampleFormat::F32 => typed::<f32>(&device, cfg.into(), sh.clone()),
        SampleFormat::I16 => typed::<i16>(&device, cfg.into(), sh.clone()),
        SampleFormat::I32 => typed::<i32>(&device, cfg.into(), sh.clone()),
        SampleFormat::U16 => typed::<u16>(&device, cfg.into(), sh.clone()),
        other => Err(Error::Audio(format!("unsupported sample format {other}"))),
    }?;
    stream.play().map_err(audio_err)?;
    Ok(stream)
}

fn typed<T>(device: &cpal::Device, cfg: cpal::StreamConfig, sh: Arc<Shared>) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32>,
{
    let ch = usize::from(cfg.channels.max(1));
    device
        .build_output_stream(
            cfg,
            move |out: &mut [T], _: &cpal::OutputCallbackInfo| {
                let vol = f32::from_bits(sh.volume.load(Ordering::Relaxed));
                let mut sum = 0.0_f32;
                let mut q = sh.queue.lock().unwrap_or_else(|e| e.into_inner());
                for frame in out.chunks_mut(ch) {
                    let s = q.pop_front().unwrap_or(0.0) * vol;
                    sum += s * s;
                    frame.fill(T::from_sample(s));
                }
                let frames = (out.len() / ch).max(1) as f32;
                sh.level
                    .store((sum / frames).sqrt().min(1.0).to_bits(), Ordering::Relaxed);
            },
            |err| tracing::warn!(%err, "speaker stream error"),
            None,
        )
        .map_err(audio_err)
}
