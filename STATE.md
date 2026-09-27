last: T005 tools/fetch-assets.ps1 (Release assets-v1 → assets/, sha256 in tools/assets.sha256; -Publish rebuilds from upstream); release uploaded; CI step added to tools/ci/ci.yml
next: T010
blocked: T002 needs-user: GitHub App lacks `workflows` permission; workflow ready at tools/ci/ci.yml → move to .github/workflows/ci.yml
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
decisions: run tauri CLI from repo root (tauri dir = crates/app, app dir = frontend); CI builds frontend before cargo (generate_context embeds dist)
decisions: release profile opt-level=s + LTO + panic=abort to keep binary/RAM small
decisions: one IPC channel + discriminated union instead of 4 channels (fully typed via ts-rs, less glue)
decisions: assets extracted to assets/<name>/ (gitignored): vosk-model-small-ru-0.22, silero_vad.onnx, vits-piper-ru_RU-denis-medium, rustpotter-jarvis, voices-priler, jarvis-sound
perf: idle_ram=? idle_cpu=? (measure after T130)
