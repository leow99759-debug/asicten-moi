# PLAN — one line per task. `[ ]` todo · `[x]` done · `[!]` blocked (reason in STATE.md). Refs → SPEC.md §.
Rule: take the FIRST `[ ]` in order. M13 is optional and runs only after v1.0.0. Each task = code + tests + green build + 1 commit.

## M0 Bootstrap
- [x] T001 Cargo workspace (core, win, ai, remote, app) + Tauri 2 + Svelte5/TS/Vite frontend skeleton; `cargo tauri dev` opens empty frameless window §1
- [!] T002 GitHub Actions (windows-latest): fmt, clippy -D warnings, cargo test, npm ci/build/vitest; cache; badge §14
- [x] T003 Config (serde JSON in %APPDATA%/Jarvis) + SQLite (history, settings) + tracing logs with rotation §1
- [x] T004 IPC events core→UI (state, transcript, history, level) + typed TS bindings §1
- [x] T005 `tools/fetch-assets.ps1`: download Vosk small-ru, Priler rustpotter .rpw + voice packs, FitoDomik/Jarvis-Sound, Piper ru voice, sha256; upload to GitHub Release `assets-v1`; CI pulls from that release §6 §13

## M1 Audio + wake + STT
- [x] T010 cpal WASAPI capture 16k mono, ring buffer, device select, level meter §2.1
- [x] T011 Silero VAD (sherpa-onnx) segmenting §1
- [x] T012 Wake word Rustpotter "Джарвис" + sensitivity; fallback Vosk grammar §1 §2.2
- [x] T013 Vosk STT lazy: load on wake with audio buffering, keep-warm timer, unload; grammar mode for no-prefix; partials → listening bar §1 §2.1
- [x] T014 Single-utterance "Джарвис, включи музыку", follow-up window, barge-in §2.1
- [x] T015 Modes: prefix / silent / mic_off + voice switch phrases + hotkeys (PTT, show, mute) §2.2 §2.3
- [ ] T016 Test fixtures: generate WAVs via Piper for all §11 phrases; pipeline test wav→intent §11

## M2 Command engine
- [ ] T020 Command JSON schema + loader + validation + path vars (%CHROME% etc. via registry/StartMenu/UWP) §4.1
- [ ] T021 Russian normalizer + numerals/time parser (unit tests ≥40 cases) §4.2
- [ ] T022 Matcher cascade exact→fuzzy→synonyms (embeddings optional, lazy), required+optional phrases, slots; bench ≤30ms/1000 cmds §4.2
- [ ] T023 Executor framework (timeouts, results, dry-run mock for tests) + Launch.*, Process.Kill §4.4
- [ ] T024 Window.* + Keys.* + Mouse.* (SendInput) §4.4
- [ ] T025 Audio/Media (Core Audio volume/mute/device, media keys) §4.4
- [ ] T026 System.* (power w/ delay, cancel, power plan, brightness, screenshot, ms-settings, recycle bin, wifi/bt, DND, clipboard, info) §4.4
- [ ] T027 Lux.Pause*, Sound.PlayWav, Speak, Ask, Run.Command, Timer, Reminder, Assistant.* §4.4 §10.3
- [ ] T028 Chain splitting ("и/потом/затем") + confirm flow (UI dialog + voice да/нет + 30s autocancel) §4.3 §4.6
- [ ] T029 Context rules (`when.foreground`) + PC modes (game/work/movie/night) §4.5 §4.7
- [ ] T030 History persistence + statuses §3.4

## M3 Voice output
- [ ] T040 Voice pack format + player (rodio), categories, no-repeat random, volume §6.1
- [ ] T041 Piper/VITS TTS via sherpa-onnx, lazy load/unload, sentence streaming, speed, FX (EQ+reverb) §6.2
- [ ] T042 SAPI fallback + engine selector backend §6.3
- [ ] T043 Built-in dialog replies without LLM, ≥40 intents (как дела, ты тут, спасибо, время, дата, шутка, приветствие) §7.5
- [ ] T044 Voice asset hunt: find more RU-dub Jarvis clips online, verify text via STT, normalize, trim, categorize into pack; map every reply category §6.1

## M4 UI
- [ ] T050 Design tokens (dark glass, accent palette, radius, fonts), frameless shell, sidebar, listening bar §3.1 §3.2
- [ ] T051 Orb (canvas/WebGL, idle breathe, reacts to mic+TTS level, pauses when hidden) §3.1
- [ ] T052 Home dashboard tiles + live counters §3.3
- [ ] T053 Main window: history + control panel toggles + volume + mic button §3.4
- [ ] T054 Tray (states, menu), close-to-tray destroys WebView, autostart, always-on-top, Mica/Acrylic §3.2 §3.5
- [ ] T055 Desktop avatar overlay (transparent click-through, fullscreen-aware) §3.6
- [ ] T056 HUD skin on activation §3.7
- [ ] T057 Settings pages (all tabs, themes, transparency, blur, mic device+level, mode phrases, import/export) §10.4
- [ ] T058 Voice synthesis page (engine cards, speed, preview, waveform) §6.4

## M5 Command editor
- [ ] T060 Tree folders→commands→phrases, toolbar, search, drag&drop, breadcrumbs §5.1
- [ ] T061 Tabs + command card (name, Связывать, Подтверждать, actions list w/ drag, param pickers, app icons) §5.2 §5.3
- [ ] T062 Phrases + optional phrases chips + live match preview + reply picker + ▶ Test §5.3
- [ ] T063 Action recorder (LL hooks, pauses, ignore own window) §5.4
- [ ] T064 Import/export .jarvispack §5.6

## M6 Packs
- [ ] T070 Addons screen (Паки/Команды/Озвучка, filters, categories, install/uninstall, counters) §9
- [ ] T071 Packs batch 1: Windows, Explorer, Media, browsers (5), YouTube §9
- [ ] T072 Packs batch 2: music (Spotify, Яндекс Музыка, VK), messengers (Telegram, Discord, WhatsApp), Zoom §9
- [ ] T073 Packs batch 3: games (Steam, Epic, CS2, Dota2, Minecraft, Fortnite, Valorant) §9
- [ ] T074 Packs batch 4: creative+work (OBS, Ps, Pr, Ae, Ai, CapCut, Figma, Blender, VS Code, Notion, Word, Excel, PowerPoint) §9
- [ ] T075 Pack validator test: schema, no coordinate clicks, no dup phrases across packs; total ≥30 packs/≥500 cmds §9

## M7 Smart features WITHOUT LLM
- [ ] T080 Start Menu/UWP/registry app index + fuzzy Launch.Find + synonyms §4.1 §5.5
- [ ] T081 Offline RU scenario parser for "Создать с помощью AI" (both video examples → exact steps) + phrase suggestions §5.5
- [ ] T082 PowerPoint COM: template presentation by topic, theme/variant color; Word actions; context rules §7.4 §4.5
- [ ] T083 "Jarvis AI" sidebar page placeholder (module off, description, enable button disabled until M13) §7

## M8 Online extras
- [ ] T090 Connectivity check + no_internet flow everywhere §8
- [ ] T091 Weather (Open-Meteo), currency (CBR), news (RSS), web search open §8

## M9 Remote
- [ ] T100 axum server + QR pairing + device keys (LAN only, off by default) §10.1
- [ ] T101 PWA PC Control (media, favorites, text/voice command, status, history) §10.1

## M10 Extras
- [ ] T110 Workspace profiles (apps, monitors, halves, layers, reposition if running, close profile) §10.2
- [ ] T111 Double-clap trigger (own implementation) §2.4

## M11 Jarvis voice
- [ ] T120 tools/voice-train: dataset (Jarvis-Sound + Priler + user clips + synth Piper ru → RVC Jarvis), cleaning, Piper fine-tune on local RTX 3060 6GB, ONNX export, README ≤30 lines §6.2
- [ ] T121 make_clips.py: generate missing pack phrases; integrate trained model as default; upload model to Release assets §6.1 §6.2

## M12 Release
- [ ] T130 bench.ps1 + CI perf gate; fix until §12 met §12
- [ ] T131 E2E checklist for all §11 scenarios (auto via fixtures + manual list) all pass §11
- [ ] T132 NSIS installers Full offline (all assets bundled) + Lite, tag v1.0.0, GitHub Release in private repo §14

## M13 OPTIONAL Jarvis AI module (off by default; only after v1.0.0)
- [ ] T140 Hardware/free-RAM check; model picker; download once (3B–4B Q4, full GPU offload on RTX 3060) §7.1
- [ ] T141 llama-server CUDA sidecar manager (lazy start, health, 60s idle unload, never during fullscreen games) §7.1
- [ ] T142 Tool-calling + persona + safety (confirm passthrough) + NLU fallback when module on §7.3 §4.2
- [ ] T143 AI chat window (dialogs, streaming, voice in/out, result cards) §7.2
- [ ] T144 LLM mode for scenario builder + LLM presentation content §5.5 §7.4
- [ ] T145 GigaAM v2 lazy STT for chat/dictation; tag v1.1.0 §1
