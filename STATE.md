last: T029 context rules: Backend::foreground_exe, Command::context_rank (when.foreground, «A.EXE|B.EXE», +0.1 bonus, else filtered), Matcher::best_with(rank); packs/modes.json (game/work/movie/night/normal, §4.7), packs/office.json (PowerPoint/Word context); night = brightness+volume+DND+opens ms-settings:nightlight (no public Night Light API)
next: T030 history persistence + statuses → tag m2
blocked: T002 needs-user (workflow edits: user pastes tools/ci/ci.yml into .github/workflows) | wake sensitivity 50 needs real-voice tuning (synthetic «Джарвис, включи музыку» passes only at 70)
decisions: dev agent on Linux (no mic/GUI) → windows-latest CI is truth; Windows-only code behind cfg(windows); local win check: clippy --target x86_64-pc-windows-msvc with llvm-rc/lib.exe stubs
decisions: STT = sherpa-onnx streaming zipformer small-ru int8 (Vosk team) instead of Vosk: +90 MB RAM loaded vs +190 MB, 3x faster decode, no libvosk DLLs, better free speech. No grammar mode → no-prefix mode relies on strict NLU match
decisions: wake fallback = user-recorded custom .rpw (rustpotter) in settings, not Vosk grammar (zipformer doesn't know «джарвис»)
decisions: sherpa-onnx official crate (static) for VAD/STT/TTS; rustpotter = Priler fork (candle locked 0.9.1)
decisions: one IPC channel `core` + discriminated union (ts-rs); bindings regen on cargo test, CI checks diff
decisions: assets in Release assets-v1 → assets/<name>/ (gitignored); tests needing models skip if missing; small 22 kHz Piper fixture WAVs committed in tests/fixtures
decisions: embeddings tier (MiniLM) postponed: fuzzy+synonyms cover §11; add only if real usage shows misses
perf: idle_ram=? idle_cpu=? | stt load 1.8 s on slow 1-core sandbox, +90 MB RSS
decisions: Fish Audio s2.1-pro-free (free, RU) + voice «ДЖАРВИС» 4c3eaacc… → build-time WAV phrase generation (T044/T121) and Piper fine-tune dataset (T120); runtime stays offline
