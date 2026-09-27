last: T011 core::vad Silero via official sherpa-onnx crate (static prebuilt libs, auto-download), min_silence 0.4s; tests use assets/ (skip if not fetched)
next: T012
blocked: T002 needs-user: GitHub App lacks `workflows` permission; workflow ready at tools/ci/ci.yml → move to .github/workflows/ci.yml
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
decisions: run tauri CLI from repo root (tauri dir = crates/app, app dir = frontend); CI builds frontend before cargo (generate_context embeds dist)
decisions: release profile opt-level=s + LTO + panic=abort to keep binary/RAM small
decisions: one IPC channel + discriminated union instead of 4 channels (fully typed via ts-rs, less glue)
decisions: assets extracted to assets/<name>/ (gitignored): vosk-model-small-ru-0.22, silero_vad.onnx, vits-piper-ru_RU-denis-medium, rustpotter-jarvis, voices-priler, jarvis-sound
decisions: sherpa-onnx official Rust crate (static link) for VAD/TTS; tests needing models use test_util::asset() and skip when assets/ missing
perf: idle_ram=? idle_cpu=? (measure after T130)
