# Jarvis — offline voice assistant for Windows 10/11 (Rust + Tauri 2 + Svelte)
Spec: SPEC.md (read only the § you need) · Tasks: PLAN.md · Memory: STATE.md (≤30 lines, overwrite)
ALWAYS follow skill `jarvis-dev` (.claude/skills/jarvis-dev/SKILL.md) and coding style skill `ponytail` (.claude/skills/ponytail/SKILL.md, level full).
UI work (M4, M5, editor, settings, overlay): also read `apple-design` and `emil-design-eng`, and for motion `animate` / `animation-vocabulary` / `review-animations` (.claude/skills/, by Emil Kowalski, MIT). Target: Apple / Microsoft / Telegram / Discord polish. Visual reference = video 26 frames in `docs/design/video26/` (Luxify: dark navy glass, blue accent, orb).
Language: code/comments/commits EN; UI/voice RU.
Target PC: 8 GB RAM (mostly used), RTX 3060 6 GB. Core works WITHOUT LLM (LLM = optional M13).
Priorities: works offline > low RAM/CPU > matches reference videos 1:1 > extra features.
Binaries (models/WAV/GGUF) go to GitHub Release assets, not git history. Private personal project: reusing code from SPEC §13 refs is OK.

## UI design system
Every UI change follows `docs/design/SYSTEM.md` (tokens in `frontend/src/styles/tokens.css`). Review screenshots with Chromium `--font-render-hinting=none`, otherwise Linux headless spaces Cyrillic unevenly.
