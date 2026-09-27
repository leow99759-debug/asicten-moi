---
name: jarvis-dev
description: Mandatory workflow for developing the Jarvis Windows voice assistant in this repo. Use for every task/iteration: picking the next PLAN.md task, coding, testing, committing, and saving tokens.
---

# Jarvis dev workflow

## Iteration (one task per iteration, strictly)
1. Read `STATE.md` (≤30 lines). Read `PLAN.md` → first `[ ]` task. Nothing else upfront.
2. Read ONLY the SPEC section(s) the task references: `rg -n "^## §4|^### §4.4" SPEC.md`, then read just that range. Never read the whole SPEC.
3. Look at existing code with `rg`/targeted reads. Never open `target/`, `node_modules/`, lockfiles, assets, generated bindings.
4. Implement the smallest complete version that meets the SPEC section. No stubs/TODOs left in shipped paths.
5. Verify (all must pass locally):
   - `cargo fmt --all` · `cargo clippy --workspace -- -D warnings` · `cargo test --workspace`
   - `cd frontend && npm run check && npm run build && npx vitest run` (if frontend touched)
   - For behavior tasks: add/extend a test (unit, fixture WAV, or executor dry-run). For UI tasks: run `cargo tauri dev` briefly and take a screenshot if possible.
6. Tick `[x]` in PLAN.md, overwrite STATE.md, commit, push (see Git).
7. Stop the iteration. Output only: `T0xx done` or `T0xx blocked: <reason>`.

## Stuck rule
Same task fails 3 attempts → mark `[!]`, write 1-line reason + what you tried in STATE.md, move to next task. Blocked tasks get revisited after the milestone ends.
Needs a human (hardware, license, API key, GPU training): mark `[!] needs-user`, continue with others.

## Token economy (hard rules)
- No reports, summaries, changelogs, design docs, or long comments. Code + tests only. The only prose files are STATE.md (overwrite, ≤30 lines) and the short README.
- Don't re-read files you just wrote. Don't cat big files; use `rg -n` + line ranges.
- Don't paste large outputs; pipe test/build output through `| tail -40`, or grep for `error|warning|FAILED`.
- Batch edits per file. Prefer editing over rewriting.
- Subagents only for isolated lookups (library API, docs). Give them tight questions.
- Chat output per iteration: 1 line.

## STATE.md format (overwrite each time)
```
last: T0xx <one line what changed>
next: T0yy
blocked: T0zz <reason> | none
decisions: <≤8 one-liners, prune old>
perf: idle_ram=?MB idle_cpu=?% (update when measured)
```

## Git / GitHub
- Branch `main` protected by CI. Work on `main` directly for small tasks, or `feat/T0xx` + PR if a task >400 lines; merge only when CI green.
- One commit per task: `T0xx: <imperative, ≤60 chars>`. `git push` after each commit. Check CI with `gh run list -L 1`; if red, fixing it is the next task.
- Never commit binaries to git history (models, WAV, GGUF): keep them in `.gitignore`, publish via `gh release upload assets-v1`, bundle into the Full installer. No secrets in git.
- Private personal project: borrowing code/assets from SPEC §13 refs is allowed; add `// adapted from <repo>` comment.
- Milestone done → `git tag m<N>` and push tag.

## Architecture guardrails
- Rust core owns all logic; frontend is a thin view over IPC events/commands. Types shared via generated TS bindings.
- Target PC: 8 GB RAM with ~7 GB already used by Windows+Chrome; RTX 3060 6 GB. RAM is the #1 constraint.
- Idle budget: ≤60 MB RAM (hard 100), ≤1% CPU. Idle = audio capture + VAD + wake word only. Vosk, neural TTS, embeddings, UI load on demand and unload on timers. Hidden window = destroyed WebView. Orb/animations stop when not visible. Measure, don't guess.
- Core must work with NO LLM. The LLM module (M13) is optional and off by default.
- Every Windows side effect goes through `crates/win` behind a trait, so tests use a mock executor (dry-run). Tests must never shut down, click, or type on the dev machine.
- Dangerous actions → confirm flow (SPEC §4.6). LLM can call only whitelisted tools.
- Offline-first: every network call has a timeout (≤4 s) and a NO_INTERNET path that speaks the no_internet clip.
- UI strings in Russian via i18n keys (`frontend/src/i18n/ru.json`), no hardcoded strings.
- Error handling: `anyhow` in app layer, `thiserror` in libs; no `unwrap()` outside tests.
- Keep crates small; a file >500 lines should be split.

## Useful commands
- Dev: `cargo tauri dev` · Build: `cargo tauri build`
- Assets: `pwsh tools/fetch-assets.ps1`
- Perf: `pwsh tools/bench.ps1` (after T130)
- CI: `gh run list -L 3` · `gh run view --log-failed | tail -60`
