last: T004 IPC: single Tauri channel `core` with tagged CoreEvent (state/transcript/history/level), EventSink trait, ts-rs bindings → frontend/src/lib/bindings (regen by cargo test), typed `on(kind, cb)`
next: T005
blocked: T002 needs-user: GitHub App lacks `workflows` permission; workflow ready at tools/ci/ci.yml → move to .github/workflows/ci.yml
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
decisions: run tauri CLI from repo root (tauri dir = crates/app, app dir = frontend); CI builds frontend before cargo (generate_context embeds dist)
decisions: release profile opt-level=s + LTO + panic=abort to keep binary/RAM small
decisions: one IPC channel + discriminated union instead of 4 channels (fully typed via ts-rs, less glue)
perf: idle_ram=? idle_cpu=? (measure after T130)
