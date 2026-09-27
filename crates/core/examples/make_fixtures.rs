//! Regenerate SPEC §11 WAV fixtures with the base Piper RU voice (16 kHz mono).
//! `cargo run -p jarvis-core --example make_fixtures --release` (needs assets/, see tools/fetch-assets.ps1)

use std::path::Path;

use jarvis_core::audio::{Converter, SAMPLE_RATE};
use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsModelConfig,
    OfflineTtsVitsModelConfig,
};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let voice = root.join("assets/vits-piper-ru_RU-denis-medium");
    let p = |f: &str| Some(voice.join(f).to_string_lossy().into_owned());
    let tts = OfflineTts::create(&OfflineTtsConfig {
        model: OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: p("ru_RU-denis-medium.onnx"),
                tokens: p("tokens.txt"),
                data_dir: p("espeak-ng-data"),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: 2,
            ..Default::default()
        },
        max_num_sentences: 1,
        ..Default::default()
    })
    .expect("Piper voice missing: run tools/fetch-assets.ps1");

    let dir = root.join("tests/fixtures/s11");
    let list = std::fs::read_to_string(dir.join("phrases.tsv")).expect("phrases.tsv");
    for line in list.lines().filter(|l| !l.trim().is_empty()) {
        let (id, text) = line.split_once('\t').expect("id<TAB>phrase");
        let cfg = GenerationConfig {
            speed: 1.0,
            silence_scale: 0.2,
            ..Default::default()
        };
        let audio = tts
            .generate_with_config::<fn(&[f32], f32) -> bool>(text, &cfg, None)
            .expect("tts");
        let mut pcm = Vec::new();
        Converter::new(audio.sample_rate() as u32, 1).push(audio.samples(), &mut pcm);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(dir.join(format!("{id}.wav")), spec).expect("wav");
        for s in pcm {
            w.write_sample(s).expect("write");
        }
        w.finalize().expect("finalize");
        println!("{id} {text}");
    }
}
