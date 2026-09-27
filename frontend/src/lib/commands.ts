import { invoke } from "@tauri-apps/api/core";
import type { HistoryEntry } from "./bindings/HistoryEntry";
import type { ModeCommand } from "./bindings/ModeCommand";
import { inTauri } from "./window";

/** Typed wrappers over Rust #[tauri::command]s (crates/app/src/lib.rs). No-ops outside Tauri. */
const call = <T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> =>
  inTauri() ? invoke<T>(cmd, args) : Promise.resolve(undefined);

export const setMode = (cmd: ModeCommand) => call("set_mode", { cmd });
export const activate = () => call("activate");
export const confirmAnswer = (yes: boolean) => call("confirm_answer", { yes });
export const runText = (text: string) => call("run_text", { text });
export const history = (limit = 500) => call<HistoryEntry[]>("history", { limit });
