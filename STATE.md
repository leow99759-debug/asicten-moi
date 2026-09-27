last: T012 core::wake Rustpotter (Priler fork, candle locked 0.9.1) + jarvis-default.rpw, Priler detector cfg, sensitivity 0–100→thr 0.75..0.25; fixtures tests/fixtures/wake/*.wav (Piper denis)
next: T013 (also add Vosk grammar wake fallback there)
blocked: T002 needs-user (workflow file edits go through user) | wake sensitivity default 50 needs real-voice tuning (synthetic «Джарвис, включи музыку» only passes at 70)
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
decisions: run tauri CLI from repo root (tauri dir = crates/app, app dir = frontend); CI builds frontend before cargo (generate_context embeds dist)
decisions: release profile opt-level=s + LTO + panic=abort to keep binary/RAM small
decisions: one IPC channel + discriminated union instead of 4 channels (fully typed via ts-rs, less glue)
decisions: assets extracted to assets/<name>/ (gitignored): vosk-model-small-ru-0.22, silero_vad.onnx, vits-piper-ru_RU-denis-medium, rustpotter-jarvis, voices-priler, jarvis-sound
decisions: sherpa-onnx official Rust crate (static link) for VAD/TTS; tests needing models use test_util::asset() and skip when assets/ missing
perf: idle_ram=? idle_cpu=? (measure after T130)
