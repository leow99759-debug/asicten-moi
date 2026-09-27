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

/** Maximize ⇄ restore; returns the new maximized state. */
export async function toggleMaximize(): Promise<boolean> {
  if (!inTauri()) return false;
  const w = getCurrentWindow();
  await w.toggleMaximize();
  return w.isMaximized();
}

/** Native «Open file» dialog (action 📁 buttons); null when cancelled or outside Tauri. */
export async function pickFile(filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  if (!inTauri()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const r = await open({ multiple: false, directory: false, filters });
  return typeof r === "string" ? r : null;
}

export async function saveFile(defaultPath: string, filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
  if (!inTauri()) return null;
  const { save } = await import("@tauri-apps/plugin-dialog");
  return save({ defaultPath, filters });
}
