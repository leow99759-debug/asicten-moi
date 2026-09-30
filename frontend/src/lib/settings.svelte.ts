// Config mirror for the settings pages (§10.4). Edits apply to the UI at once and are
// saved to the core after a short pause, so dragging a slider doesn't spam disk writes.
import type { Config } from "./bindings/Config";
import { getConfig, setConfig } from "./commands";
import { setOnTop } from "./window";

export const DEFAULTS: Config = {
  prefix_mode: true,
  silent_mode: false,
  mic_enabled: true,
  mic_device: null,
  wake_sensitivity: 50,
  listen_timeout_sec: 6,
  followup_sec: 5,
  stt_keep_warm_sec: 120,
  memory_saver: true,
  voice_volume: 80,
  close_to_tray: true,
  autostart: false,
  voice_engine: "jarvis",
  voice_speed: 1.0,
  voice_fx: true,
  hotkeys: { push_to_talk: "Ctrl+Alt+J", toggle_window: "Ctrl+Alt+H", toggle_mic: "Ctrl+Alt+M" },
  ui: { accent: "#4486ff", transparency: 30, blur: 90, animations: true, avatar: true, hud: false, pill: true, on_top: false, avatar_pos: null },
  mode_phrases: {
    prefix_on: "перейди в режим префикса",
    prefix_off: "выключи режим префикса",
    silent_on: "перейди в тихий режим",
    silent_off: "выключи тихий режим",
    mic_off: "хватит слушать",
  },
  introduced: true,
  online: { gemini_keys: ["", ""], gemini_model: "gemini-2.5-flash-lite", fish_key: "", fish_voice: "", eleven_key: "", eleven_voice: "onwK4e9ZLuTAKqWW03F9", ai: [], classroom_id: "", classroom_secret: "", classroom_token: "" },
};

export const cfg = $state<{ value: Config }>({ value: structuredClone(DEFAULTS) });

export function hexToRgb(hex: string): string | null {
  if (!/^#[0-9a-f]{6}$/i.test(hex)) return null;
  const n = parseInt(hex.slice(1), 16);
  return `${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}`;
}

let shownAccent = "";

/** Push look & feel prefs into CSS variables / window flags. `fade` = cross-fade a theme change. */
export function applyUi(c: Config = cfg.value, fade = false): void {
  const changed = shownAccent !== "" && shownAccent !== c.ui.accent;
  shownAccent = c.ui.accent;
  const still = !c.ui.animations || matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (fade && changed && !still && document.startViewTransition) document.startViewTransition(() => paint(c));
  else paint(c);
}

function paint(c: Config): void {
  const root = document.documentElement;
  const rgb = hexToRgb(c.ui.accent);
  if (rgb) {
    root.style.setProperty("--accent", c.ui.accent);
    root.style.setProperty("--accent-rgb", rgb);
    const [r, g, b] = rgb.split(",").map(Number);
    const light = 0.299 * r + 0.587 * g + 0.114 * b > 186;
    root.style.setProperty("--on-accent", light ? "#111" : "#fff");
  }
  // transparency 0–100 → glass alpha 0.85–0.35; blur 0–100 → 0–40 px
  const alpha = 0.85 - (c.ui.transparency / 100) * 0.5;
  root.style.setProperty("--surface", `rgba(30, 33, 40, ${alpha.toFixed(2)})`);
  root.style.setProperty("--surface-2", `rgba(38, 42, 51, ${(alpha - 0.03).toFixed(2)})`);
  root.style.setProperty("--blur", `${Math.round((c.ui.blur / 100) * 40)}px`);
  root.dataset.motion = c.ui.animations ? "on" : "off";
}

let timer: ReturnType<typeof setTimeout> | undefined;
let lastOnTop: boolean | undefined;

/** Call after mutating `cfg.value`. */
export function saved(): void {
  applyUi();
  if (lastOnTop !== cfg.value.ui.on_top) {
    lastOnTop = cfg.value.ui.on_top;
    setOnTop(lastOnTop);
  }
  clearTimeout(timer);
  timer = setTimeout(() => setConfig($state.snapshot(cfg.value)), 300);
}

export async function loadConfig(fade = false): Promise<void> {
  const c = await getConfig();
  if (c) cfg.value = c;
  lastOnTop = cfg.value.ui.on_top;
  applyUi(cfg.value, fade);
}

/** Theme swatch click / voice: the whole window cross-fades to the new colour. */
export function pickAccent(hex: string, save = true): void {
  cfg.value.ui.accent = hex;
  applyUi(cfg.value, true);
  if (save) saved();
}
