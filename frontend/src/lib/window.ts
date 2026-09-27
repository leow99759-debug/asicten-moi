import { getCurrentWindow } from "@tauri-apps/api/window";

/** True when running inside the Tauri webview (false in plain browser / tests). */
export const inTauri = (): boolean => typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export async function minimize(): Promise<void> {
  if (inTauri()) await getCurrentWindow().minimize();
}

export async function close(): Promise<void> {
  if (inTauri()) await getCurrentWindow().close();
}

export async function setOnTop(on: boolean): Promise<void> {
  if (inTauri()) await getCurrentWindow().setAlwaysOnTop(on);
}
