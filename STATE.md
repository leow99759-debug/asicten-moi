last: T063 recorder: crates/win/src/recorder.rs pure Recorder (Keys.Type from ToUnicodeEx chars incl. Cyrillic + backspace edit, chords → Keys.Press Ctrl+S, double click, wheel bursts, pauses ≥200ms rounded 50, trailing pause dropped) + WH_MOUSE_LL/WH_KEYBOARD_LL thread (skips injected input and Jarvis's own windows); IPC recorder_start/stop; card «● Запись 00:02 ■ Стоп». T062 CI green
next: T064 import/export .jarvispack §5.6 (then M6 T070). Untested on real Windows: recorder hooks, ▶ Тест, probe, say_reply
blocked: wake sensitivity 50 needs real-voice tuning
decisions: dev agent on Linux (no mic/GUI) → windows-latest CI is truth; Windows-only code behind cfg(windows); local win check: clippy --target x86_64-pc-windows-msvc with llvm-rc/lib.exe stubs
decisions: STT = sherpa-onnx streaming zipformer small-ru int8 (Vosk team) instead of Vosk: +90 MB RAM loaded vs +190 MB, 3x faster decode, no libvosk DLLs, better free speech. No grammar mode → no-prefix mode relies on strict NLU match
decisions: wake fallback = user-recorded custom .rpw (rustpotter) in settings, not Vosk grammar (zipformer doesn't know «джарвис»)
decisions: sherpa-onnx official crate (static) for VAD/STT/TTS; rustpotter = Priler fork (candle locked 0.9.1)
decisions: one IPC channel `core` + discriminated union (ts-rs); bindings regen on cargo test, CI checks diff
decisions: assets in Release assets-v1 → assets/<name>/ (gitignored); tests needing models skip if missing; small 22 kHz Piper fixture WAVs committed in tests/fixtures
decisions: embeddings tier (MiniLM) postponed: fuzzy+synonyms cover §11; add only if real usage shows misses
perf: idle_ram=? idle_cpu=? | stt load 1.8 s on slow 1-core sandbox, +90 MB RSS
decisions: Fish Audio s2.1-pro-free (free, RU) + voice «ДЖАРВИС» 4c3eaacc… → build-time WAV phrase generation (T044/T121) and Piper fine-tune dataset (T120); runtime stays offline
decisions: playback via cpal directly (already a dep, rodio 0.22 needs MSRV 1.87 + 2nd cpal); linear resample is fine for speech
decisions: Fish Audio lines never mix into categories that have originals (user rule: originals win); Priler howdy/remaster skipped (synthetic, speaker sim 0.55–0.79 vs og 0.8–0.94)
