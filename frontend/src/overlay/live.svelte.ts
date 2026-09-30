// Shared state for the overlay windows: assistant state, levels, transcript, accent colour.
import { listen } from "@tauri-apps/api/event";
import type { AssistantState } from "../lib/bindings/AssistantState";
import { on } from "../lib/ipc";
import { applyUi, loadConfig } from "../lib/settings.svelte";
import { inTauri } from "../lib/window";
import { resultLine, type PillResult } from "../lib/pill";

export const live = $state({
  state: "idle" as AssistantState,
  level: 0,
  transcript: "",
  /** Last command's result until the next utterance starts. */
  result: null as PillResult | null,
});

export const isActive = (s: AssistantState) => s === "listening" || s === "processing" || s === "speaking";

export async function connectLive(demoState: AssistantState = "listening"): Promise<void> {
  if (!inTauri()) {
    applyUi();
    live.state = demoState;
    live.transcript = "Джарвис, включи музыку";
    if (new URLSearchParams(location.search).has("still")) {
      live.level = 0.45;
      return;
    }
    let t = 0;
    const tick = () => {
      t += 0.05;
      live.level = Math.max(0, Math.sin(t * 3) * 0.35 + Math.sin(t * 7.3) * 0.2 + 0.25);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
    return;
  }
  await loadConfig();
  await Promise.all([
    listen("config", () => loadConfig(true)),
    on("state", (s) => {
      live.state = s;
      if (s === "listening") live.transcript = "";
    }),
    on("level", (l) => (live.level = Math.max(l.mic, l.tts))),
    on("transcript", (t) => {
      if (!t.is_final) live.result = null;
      live.transcript = t.text;
    }),
    on("outcome", (o) => (live.result = resultLine(o))),
  ]);
}
