//! Test helpers: locate fetched assets (tools/fetch-assets.ps1) and load WAVs as 16 kHz mono.

use std::path::{Path, PathBuf};

use crate::audio::Converter;

/// `assets/<rel>` in the repo root, if fetched.
pub fn asset(rel: &str) -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets")
        .join(rel);
    p.exists().then_some(p)
}

pub fn read_wav_16k(path: &Path) -> Vec<i16> {
    let mut reader = hound::WavReader::open(path).expect("open wav");
    let spec = reader.spec();
    let floats: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.expect("sample") as f32 / scale)
                .collect()
        }
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.expect("sample"))
            .collect(),
    };
    let mut out = Vec::new();
    Converter::new(spec.sample_rate, spec.channels).push(&floats, &mut out);
    out
}
