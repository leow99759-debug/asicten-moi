You are developing the Jarvis project in this repo. Follow skill `jarvis-dev` exactly.
Do ONE iteration: STATE.md → first `[ ]` task in PLAN.md → its SPEC § only → implement → fmt/clippy/test/build green → tick task → overwrite STATE.md → commit `T0xx: ...` → push.
If CI on main is red, fixing it comes first.
If a task fails 3 times, mark `[!]` with reason and take the next one.
When EVERY task in M0–M12 of PLAN.md is `[x]` (only `[!] needs-user` may remain, listed in STATE.md) AND all SPEC §11 scenarios pass AND §12 perf is met AND v1.0.0 release with Full installer exists, output exactly: <promise>JARVIS_V1_DONE</promise> (M13 optional AI is started separately by the user)
Otherwise output one line: `T0xx done` / `T0xx blocked: reason`.
