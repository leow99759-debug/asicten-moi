last: T000 spec, workflow, .gitignore, README
next: T001
blocked: none
decisions: dev agent runs on Linux (no mic/GUI) → windows-latest CI is the source of truth for build/tests; Windows-only code behind cfg(windows) + traits so Linux `cargo check` works for pure crates
decisions: .rpw wake models also gitignored (Release assets only)
perf: idle_ram=? idle_cpu=? (measure after T130)
