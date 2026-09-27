//! WASAPI capture via cpal on a dedicated thread (cpal streams are not `Send` everywhere).

use std::sync::mpsc;
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use super::Converter;
use crate::{Error, Result};

fn audio_err(e: impl std::fmt::Display) -> Error {
    Error::Audio(e.to_string())
}

/// Names of available input devices (for the settings picker).
pub fn input_devices() -> Result<Vec<String>> {
    let devices = cpal::default_host().input_devices().map_err(audio_err)?;
    Ok(devices
        .filter_map(|d| d.description().ok().map(|x| x.name().to_owned()))
        .collect())
}

fn pick_device(name: Option<&str>) -> Result<cpal::Device> {
    let host = cpal::default_host();
    if let Some(name) = name {
        let found = host
            .input_devices()
            .map_err(audio_err)?
            .find(|d| d.description().is_ok_and(|x| x.name() == name));
        match found {
            Some(d) => return Ok(d),
            None => tracing::warn!(name, "mic not found, using default"),
        }
    }
    host.default_input_device()
        .ok_or_else(|| Error::Audio("no input device".into()))
}

/// Running capture. Dropping it stops the stream and joins the thread.
pub struct Capture {
    stop: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Capture {
    /// Start capturing `device` (None = default). `on_chunk` gets 16 kHz mono i16 on the audio thread.
    pub fn start<F>(device: Option<&str>, on_chunk: F) -> Result<Self>
    where
        F: FnMut(&[i16]) + Send + 'static,
    {
        let device = pick_device(device)?;
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<()>>();
        let thread = std::thread::Builder::new()
            .name("jarvis-audio".into())
            .spawn(move || match build_stream(&device, on_chunk) {
                Ok(stream) => {
                    let _ = ready_tx.send(Ok(()));
                    let _ = stop_rx.recv(); // park until stop / drop
                    drop(stream);
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            })?;
        ready_rx
            .recv()
            .map_err(|_| Error::Audio("audio thread died".into()))??;
        Ok(Self {
            stop: Some(stop_tx),
            thread: Some(thread),
        })
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn build_stream<F>(device: &cpal::Device, on_chunk: F) -> Result<cpal::Stream>
where
    F: FnMut(&[i16]) + Send + 'static,
{
    let cfg = device.default_input_config().map_err(audio_err)?;
    tracing::info!(?cfg, "mic config");
    let stream = match cfg.sample_format() {
        SampleFormat::F32 => typed::<f32, F>(device, cfg.into(), on_chunk),
        SampleFormat::I16 => typed::<i16, F>(device, cfg.into(), on_chunk),
        SampleFormat::I32 => typed::<i32, F>(device, cfg.into(), on_chunk),
        SampleFormat::U16 => typed::<u16, F>(device, cfg.into(), on_chunk),
        other => Err(Error::Audio(format!("unsupported sample format {other}"))),
    }?;
    stream.play().map_err(audio_err)?;
    Ok(stream)
}

fn typed<T, F>(
    device: &cpal::Device,
    cfg: cpal::StreamConfig,
    mut on_chunk: F,
) -> Result<cpal::Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
    F: FnMut(&[i16]) + Send + 'static,
{
    let mut conv = Converter::new(cfg.sample_rate, cfg.channels);
    let mut floats = Vec::new();
    let mut out = Vec::new();
    device
        .build_input_stream(
            cfg,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                floats.clear();
                floats.extend(data.iter().map(|&s| s.to_sample::<f32>()));
                out.clear();
                conv.push(&floats, &mut out);
                if !out.is_empty() {
                    on_chunk(&out);
                }
            },
            |err| tracing::warn!(%err, "mic stream error"),
            None,
        )
        .map_err(audio_err)
}
