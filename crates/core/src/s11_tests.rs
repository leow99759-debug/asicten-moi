//! SPEC §11 acceptance fixtures: WAV (Piper) → listener (PTT) → STT text ≈ expected phrase.
//! Synthetic Piper voice is harder than a real one (e.g. «закрой окно» → «закоротно»),
//! so thresholds are loose. Intent/executor checks are added on top once the command engine exists (M2, T131).

use std::time::{Duration, Instant};

use crate::audio::SAMPLE_RATE;
use crate::listener::{samples_to_duration, ListenCfg, Listener, Output};
use crate::stt::{LazyStt, MODEL_DIR};
use crate::test_util::{asset, fixture, read_wav_16k};
use crate::text::{normalize, strip_wake};
use crate::vad::Vad;
use crate::wake::WakeWord;

fn norm(s: &str) -> String {
    strip_wake(&normalize(s))
}

#[test]
fn s11_phrases_are_recognized() {
    let (Some(rpw), Some(vad), Some(stt)) = (
        asset("rustpotter-jarvis/jarvis-default.rpw"),
        asset("silero_vad.onnx"),
        asset(MODEL_DIR),
    ) else {
        eprintln!("assets missing; skipping");
        return;
    };
    let mut l = Listener::new(
        WakeWord::new(&rpw, 50).expect("wake"),
        Vad::new(&vad, 10.0).expect("vad"),
        LazyStt::new(stt, Duration::from_secs(600)),
        ListenCfg {
            listen_timeout: Duration::from_secs(10),
            followup: Duration::from_secs(5),
        },
    );
    let list = std::fs::read_to_string(fixture("s11/phrases.tsv")).expect("phrases");
    let t0 = Instant::now();
    let mut fed = 0usize;
    let mut scores = Vec::new();
    for line in list.lines().filter(|l| !l.trim().is_empty()) {
        let (id, want) = line.split_once('\t').expect("tsv");
        let mut audio = read_wav_16k(&fixture(&format!("s11/{id}.wav")));
        audio.extend(vec![0i16; SAMPLE_RATE as usize * 2]);
        l.activate(t0 + samples_to_duration(fed));
        let mut got = None;
        for c in audio.chunks(1600) {
            fed += c.len();
            for o in l.push(c, t0 + samples_to_duration(fed)).expect("push") {
                if let Output::Final(t) = o {
                    got = Some(t);
                }
            }
        }
        let mut got = got.unwrap_or_default();
        // In the real loop Rustpotter detects the name; STT's spelling of it is irrelevant.
        if want.starts_with("Джарвис") && got.starts_with("дж") {
            got = got
                .split_once(' ')
                .map(|(_, rest)| rest.to_owned())
                .unwrap_or_default();
        }
        let score = strsim::normalized_levenshtein(&norm(want), &norm(&got));
        eprintln!("§11 #{id} {score:.2} want «{want}» got «{got}»");
        scores.push((id.to_owned(), score));
    }
    let bad: Vec<_> = scores.iter().filter(|(_, s)| *s < 0.6).collect();
    let avg = scores.iter().map(|(_, s)| s).sum::<f64>() / scores.len() as f64;
    assert!(avg >= 0.8, "average similarity {avg:.2}");
    assert!(bad.is_empty(), "poorly recognized: {bad:?}");
}
