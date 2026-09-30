last: video-32 batch: «тише/громче на N», «сверни все (вкладки)» = Win+M, «закрой все окна» (Window.CloseAll, WM_CLOSE to alt-tab windows), own GDI screenshot → Pictures\Screenshots PNG + clipboard DIB, «покажи скриншоты», top pill overlay (Слушаю… → transcript → ✓ result, ui.pill default on, destroyed after 90 s hidden), overlays excluded from capture, work mode opens Spotify + VS Code, ElevenLabs voice (key+voice id, multilingual_v2 pcm_24000) preferred over Fish
next: dialog variety/first-run calibration lines → URLs in Chrome → T072. Untested on real Windows: 16f8228 + news/Fish + Classroom + video-32 batch
blocked: wake sensitivity: default .rpw weak (synthetic probe: sens 50 → 2/4 pos, 3/46 neg; community .rpw far worse) → real fix = user-recorded .rpw; needs real-voice tuning
decisions: online calls via curl.exe; secrets in temp header files (-H @file), not argv; feeds/prices 4 s, Gemini/Fish 15 s (LLM/TTS need longer than §0 4 s rule)
decisions: UI v3 = Luxify videos 1:1 (user 2026-09-28): 64px icon rail, UPPERCASE titles, glass cards, blue pills, orb from right edge; Dashboard page removed (home = Основное окно); refs docs/design/video30 + reference/chatgpt-main.jpg
decisions: dev agent on Linux (no mic/GUI) → windows-latest CI is truth; Windows-only code behind cfg(windows); local win check: clippy -p jarvis-win/-p jarvis-app --target x86_64-pc-windows-msvc with CC/CXX/AR_x86_64_pc_windows_msvc = touch-output stub scripts + llvm-rc stub on PATH (works incl. app crate)
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
decisions: WinRT (speech/radios) + audio COM → keep_mta() pins process MTA once (CoIncrementMTAUsage), fixes flaky 0xc0000005 in jarvis-win tests
decisions: same phrase in several packs is OK only with different `when` (browsers share «новая вкладка» per exe); T075 dup check = (phrase, foreground)
decisions: MSVC ≤14.43 can't link sherpa-onnx prebuilt (LNK2001 __std_find_first_of_trivial_pos_1/2) → core/build.rs detects toolset via find-msvc-tools, cfg old_msvc_stl exports Rust shims (msvc_compat.rs); never enable on 14.44+ (LNK2005)
decisions: echo: engine polls Speaker::is_playing (+400 ms tail) → Listener::set_speaking(on, said); said words stripped from STT (text::strip_echo), echo/wake-only/empty phrase keeps the window open; barge-in = «стоп» only (rustpotter scores his own clips 0.57–0.62 = real «Джарвис»; deviates from SPEC §2.1) [user bug 2026-09-28]
decisions: «открой браузер» = %CHROME% (user wants Chrome, Windows default is Edge)
