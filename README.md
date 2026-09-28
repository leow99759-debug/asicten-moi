# Jarvis — offline voice assistant for Windows 10/11

[![CI](https://github.com/leow99759-debug/asicten-moi/actions/workflows/ci.yml/badge.svg)](https://github.com/leow99759-debug/asicten-moi/actions/workflows/ci.yml)

Private personal project. Rust core + Tauri 2 + Svelte 5 UI. Works offline, idle RAM ≤ 60 MB.

- Spec: `SPEC.md` (RU) · Tasks: `PLAN.md` · Progress memory: `STATE.md`
- Dev workflow: `.claude/skills/jarvis-dev/SKILL.md`, loop prompt: `LOOP_PROMPT.md`

## Build (Windows)
Requirements: Rust (msvc), Node.js LTS, VS Build Tools 2022 (C++; 17.14+ / MSVC 14.44+ recommended), WebView2.
On MSVC ≤14.43 the sherpa-onnx prebuilt libs miss STL helpers (`LNK2001 __std_find_first_of_trivial_pos_1`); `crates/core/build.rs` detects the toolset and adds them (`msvc_compat.rs`).
```
cd frontend && npm ci && cd ..
cargo install tauri-cli --version "^2" --locked   # once
cargo tauri dev        # dev (run from repo root)
cargo tauri build      # installer
```
Models/voices are not in git: `pwsh tools/fetch-assets.ps1` (Release `assets-v1`).

CI: GitHub Actions on `windows-latest` (fmt, clippy, tests, frontend build).
