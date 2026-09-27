last: T002 (pending activation) GitHub Actions windows-latest CI (frontend check/build/vitest, fmt, clippy -D warnings, test) + badge
next: T003
blocked: T002 needs-user: GitHub App lacks `workflows` permission; workflow ready at tools/ci/ci.yml → move to .github/workflows/ci.yml
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
decisions: run tauri CLI from repo root (tauri dir = crates/app, app dir = frontend); CI builds frontend before cargo (generate_context embeds dist)
decisions: release profile opt-level=s + LTO + panic=abort to keep binary/RAM small
perf: idle_ram=? idle_cpu=? (measure after T130)
