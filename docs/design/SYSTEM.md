# Jarvis desktop design system (v2)

Source of truth for every screen. Tokens live in `frontend/src/styles/tokens.css`; components
never use raw hex/px values that aren't listed here. Build process for a screen:
hierarchy → components → spacing → type → colour → states → motion → implement → screenshot
review (`?still&page=…`) → iterate.

Direction: **Windows 11 Fluent × Apple restraint**, density of a desktop app (Telegram,
Discord, Files App), not a web dashboard. No oversized rounded cards, no acid glass, no
all-caps titles, no gradients except the orb and the accent glow.

## Layers
| Layer | Token | Use |
|---|---|---|
| Base | `--bg-base` (Mica shows through when active) | window, title bar, sidebar |
| Content | `--bg-layer`, radius `--r-layer` top-left | the page area (like Windows Settings / Files) |
| Card | `--bg-card` + `--stroke` | grouped lists, tiles |
| Control | `--fill`, `--fill-hover`, `--fill-press` | buttons, inputs, selects, segmented |
| Flyout / dialog | `--bg-flyout` + `--shadow-flyout` + backdrop blur | menus, confirm dialog (blur only here) |

Blur (`backdrop-filter`) only on transient surfaces. Long-living surfaces are opaque/Mica
(Microsoft guidance; also cheap to render).

## Colour (dark)
- Accent: cool Windows blue `#4c8dff` (user can change). Used for selection, primary action,
  on-state of toggles, focus ring. Never for large fills.
- Text: `--text` #f3f5f8 · `--text-2` #a4abb8 · `--text-3` #707886 (hints only).
- Status: `--ok` #3ecf8e · `--warn` #f5b83d · `--err` #f0616d — always with an icon/label.

## Type — Inter Variable (bundled, Cyrillic, `opsz` auto)
| Role | Size/line | Weight | Tracking |
|---|---|---|---|
| Display (page title) | 28/34 | 750 | -0.025em |
| Title (section) | 18/24 | 700 | -0.015em |
| Subtitle (card / row title) | 14/20 | 600 | -0.006em |
| Body | 13.5/20 | 450 | -0.003em |
| Caption | 12/16 | 500 | 0 |
| Group label | 12.5/16 | 600 | 0, `--text-2` |
Numbers: `font-variant-numeric: tabular-nums`. Hierarchy = weight + size + colour together.

## Geometry
- Spacing: 4 · 8 · 12 · 16 · 20 · 24 · 32 · 40 (4-pt grid).
- Radius: `--r-xs` 4 (chips) · `--r-sm` 6 (controls) · `--r-md` 8 (cards, groups) ·
  `--r-lg` 12 (layer, dialogs) · pill for toggles/segmented thumbs.
- Controls 32 px high; list rows 48 px (56 with description); nav items 36 px; tree rows 32 px,
  16 px indent per level, selected = `--fill-hover` + the same 3×16 accent indicator.
- Title bar 40 px, caption buttons 46×40 (Windows 11), sidebar 236 px (64 px when window < 1000 px).

## Components
Sidebar (icon + label, selected = `--fill` pill + 3×16 accent indicator; bottom status panel like
Discord: mini orb, state, mic mute) · TitleBar (drag, centred listening pill, caption buttons) ·
PageHeader (display title + one-line subtitle + actions) · Group (card with header, rows split
by 1 px `--divider`) · Row (title, description, control right) · Toggle (40×20, knob stretches on
press) · Slider (4 px track, 18 px thumb with accent core) · Select · TextField · Segmented ·
Button (primary / standard / subtle) · QuickTile (Windows quick-settings style toggle tile) ·
HistoryList · Orb · ConfirmDialog.

## States
hover = `--fill-hover` · press = `--fill-press` + scale .97 (buttons/tiles only) · selected =
accent indicator · focus = 2 px accent ring, offset 2 · disabled = opacity .4, no pointer.

## Motion (emil-design-eng, apple-design)
- Durations: `--t-fast` 120 ms (hover/press) · `--t-base` 180 ms (state) · `--t-slow` 260 ms
  (page/panel enter). Exit ≈ 0.65× enter.
- Easing: `--ease-out` cubic-bezier(.23,1,.32,1) for enter; `--spring` (CSS `linear()`, no
  overshoot) for toggles, tabs indicator, segmented thumb.
- Only `transform`/`opacity`. Page enter: fade + 6 px rise. Lists: 25 ms stagger, max 8 items.
- Everything instant when «Анимации» off or `prefers-reduced-motion`.
- Nothing animates in a loop except the orb (and it stops when the window is hidden).
