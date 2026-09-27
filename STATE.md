last: T023 core::executor (Backend trait, DryRun, slot fill → typed JSON, %VAR% expand, 10 s timeout/action, stop on first error, pauses in core) + win::backend WinBackend Launch.File/Url/Uwp/Find (ShellExecuteW, fuzzy Start Menu .lnk), Process.Kill (taskkill)
next: T024
blocked: T002 needs-user (workflow edits: user pastes tools/ci/ci.yml into .github/workflows) | wake sensitivity 50 needs real-voice tuning (synthetic «Джарвис, включи музыку» passes only at 70)
decisions: dev agent on Linux (no mic/GUI) → windows-latest CI is truth; Windows-only code behind cfg(windows); local win check: clippy --target x86_64-pc-windows-msvc with llvm-rc/lib.exe stubs
decisions: STT = sherpa-onnx streaming zipformer small-ru int8 (Vosk team) instead of Vosk: +90 MB RAM loaded vs +190 MB, 3x faster decode, no libvosk DLLs, better free speech. No grammar mode → no-prefix mode relies on strict NLU match
decisions: wake fallback = user-recorded custom .rpw (rustpotter) in settings, not Vosk grammar (zipformer doesn't know «джарвис»)
decisions: sherpa-onnx official crate (static) for VAD/STT/TTS; rustpotter = Priler fork (candle locked 0.9.1)
decisions: one IPC channel `core` + discriminated union (ts-rs); bindings regen on cargo test, CI checks diff
decisions: assets in Release assets-v1 → assets/<name>/ (gitignored); tests needing models skip if missing; small 22 kHz Piper fixture WAVs committed in tests/fixtures
decisions: embeddings tier (MiniLM) postponed: fuzzy+synonyms cover §11; add only if real usage shows misses
perf: idle_ram=? idle_cpu=? | stt load 1.8 s on slow 1-core sandbox, +90 MB RSS
