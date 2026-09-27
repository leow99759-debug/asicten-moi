// Reactive app state fed by core events (crates/core/src/ipc.rs). Outside Tauri (browser
// preview, screenshots) it shows demo data so every screen can be designed without a PC.
import type { AssistantState } from "./bindings/AssistantState";
import type { HistoryEntry } from "./bindings/HistoryEntry";
import { on } from "./ipc";
import { history as loadHistory, setMode, setVoiceVolume, uiSnapshot } from "./commands";
import { inTauri, setOnTop } from "./window";

export type Page = "dashboard" | "main" | "editor" | "addons" | "ai" | "settings" | "profile";

export const app = $state({
  page: "main" as Page,
  state: "idle" as AssistantState,
  transcript: "",
  transcriptFinal: false,
  micLevel: 0,
  ttsLevel: 0,
  history: [] as HistoryEntry[],
  lastSay: "",
  prefixMode: true,
  silentMode: false,
  avatar: true,
  onTop: false,
  volume: 80,
  commandCount: 0,
});

/** Theme swatches (§3.1, §10.4) and the Russian names Jarvis understands («тема фиолетовая»). */
export const SWATCHES: Record<string, string> = {
  синий: "#3b82f6",
  фиолетовый: "#6d5ef6",
  пурпурный: "#a21caf",
  розовый: "#d946ef",
  зеленый: "#22c55e",
  оранжевый: "#f97316",
  красный: "#f43f5e",
  белый: "#f1f5f9",
};

export function setAccent(color: string): void {
  const key = color.toLowerCase().replace("ё", "е").replace(/(ая|ое|ую|ой)$/, "ый");
  const hex = SWATCHES[key] ?? (/^#[0-9a-f]{6}$/i.test(color) ? color : null);
  if (!hex) return;
  const n = parseInt(hex.slice(1), 16);
  const root = document.documentElement.style;
  root.setProperty("--accent", hex);
  root.setProperty("--accent-rgb", `${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}`);
  root.setProperty("--on-accent", key === "белый" ? "#111" : "#fff");
}

export function togglePrefix(on: boolean): void {
  app.prefixMode = on;
  setMode(on ? "prefix_on" : "prefix_off");
}

export function toggleSilent(on: boolean): void {
  app.silentMode = on;
  setMode(on ? "silent_on" : "silent_off");
}

export function toggleOnTop(on: boolean): void {
  app.onTop = on;
  setOnTop(on);
}

let volTimer: ReturnType<typeof setTimeout> | undefined;
export function changeVolume(v: number): void {
  app.volume = v;
  clearTimeout(volTimer); // save once the thumb rests
  volTimer = setTimeout(() => setVoiceVolume(v), 250);
}

export async function connect(): Promise<void> {
  if (!inTauri()) return demo();
  const [snap, hist] = await Promise.all([uiSnapshot(), loadHistory(500)]);
  app.history = hist ?? [];
  if (snap) {
    app.prefixMode = snap.prefix_mode;
    app.silentMode = snap.silent_mode;
    app.volume = snap.voice_volume;
    app.commandCount = snap.commands;
    if (!snap.mic_enabled) app.state = "mic_off";
  }
  await Promise.all([
    on("state", (s) => (app.state = s)),
    on("transcript", (t) => {
      app.transcript = t.text;
      app.transcriptFinal = t.is_final;
    }),
    on("level", (l) => {
      app.micLevel = l.mic;
      app.ttsLevel = l.tts;
    }),
    on("history", (h) => (app.history = [h, ...app.history].slice(0, 500))),
    on("say", (s) => (app.lastSay = s)),
    on("ui", (u) => {
      if (u.kind === "set_theme") setAccent(u.value);
      if (u.kind === "open_page") app.page = u.value as Page;
    }),
  ]);
}

function demo(): void {
  const now = Date.now();
  const rows: [string, HistoryEntry["status"]][] = [
    ["Джарвис, включи музыку", "done"],
    ["Сверни все окна", "done"],
    ["Открой браузер", "done"],
    ["Громкость на пятьдесят", "done"],
    ["Какая погода в Москве", "no_internet"],
    ["Выключи компьютер", "cancelled"],
    ["Запусти фотошоп", "error"],
  ];
  app.history = rows.map(([phrase, status], i) => ({
    id: rows.length - i,
    ts: now - i * 1000 * 60 * 7,
    phrase,
    command_id: "demo",
    status,
  }));
  app.commandCount = 985;
  app.state = "listening";
  app.transcript = "Джарвис, включи музыку";
  let t = 0;
  const tick = () => {
    t += 0.05;
    app.micLevel = Math.max(0, Math.sin(t * 3) * 0.35 + Math.sin(t * 7.3) * 0.2 + 0.25);
    requestAnimationFrame(tick);
  };
  if (!new URLSearchParams(location.search).has("still")) requestAnimationFrame(tick);
  else app.micLevel = 0.45;
}
