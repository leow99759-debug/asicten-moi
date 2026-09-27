import { invoke } from "@tauri-apps/api/core";
import type { Action } from "./bindings/Action";
import type { Addon } from "./bindings/Addon";
import type { Config } from "./bindings/Config";
import type { Command } from "./bindings/Command";
import type { HistoryEntry } from "./bindings/HistoryEntry";
import type { Library } from "./bindings/Library";
import type { ModeCommand } from "./bindings/ModeCommand";
import type { Pack } from "./bindings/Pack";
import type { Probe } from "./bindings/Probe";
import type { Reply } from "./bindings/Reply";
import type { UiSnapshot } from "./bindings/UiSnapshot";
import { inTauri } from "./window";

/** Typed wrappers over Rust #[tauri::command]s (crates/app/src/lib.rs). No-ops outside Tauri. */
const call = <T>(cmd: string, args?: Record<string, unknown>): Promise<T | undefined> =>
  inTauri() ? invoke<T>(cmd, args) : Promise.resolve(undefined);

export const setMode = (cmd: ModeCommand) => call("set_mode", { cmd });
export const activate = () => call("activate");
export const confirmAnswer = (yes: boolean) => call("confirm_answer", { yes });
export const runText = (text: string) => call("run_text", { text });
export const history = (limit = 500) => call<HistoryEntry[]>("history", { limit });
export const uiSnapshot = () => call<UiSnapshot>("ui_snapshot");
export const setVoiceVolume = (volume: number) => call("set_voice_volume", { volume });
export const windowMaterial = () => call<boolean>("window_material");
export const getConfig = () => call<Config>("get_config");
export const setConfig = (config: Config) => call("set_config", { config });
export const micDevices = () => call<string[]>("mic_devices");
export const previewVoice = () => call("preview_voice");
export const avatarEdit = (on: boolean) => call("avatar_edit", { on });
export const hudPreview = () => call("hud_preview");
export const editorLibrary = () => call<Library>("editor_library");
export const editorSave = (commands: Command[], folders: string[][]) =>
  call<Library>("editor_save", { commands, folders });
export const editorProbe = (commands: Command[], target: Command, texts: string[]) =>
  call<Probe[]>("editor_probe", { commands, target, texts });
export const editorTest = (command: Command, sample: string) => call("editor_test", { command, sample });
export const sayReply = (reply: Reply) => call("say_reply", { reply });
export const recorderStart = () => call("recorder_start");
export const recorderStop = () => call<Action[]>("recorder_stop");
export const packWrite = (path: string, pack: Pack) => call("pack_write", { path, pack });
export const packRead = (path: string) => call<[Pack, string[]]>("pack_read", { path });
export const addonsList = () => call<Addon[]>("addons_list");
export const addonSet = (id: string, on: boolean) => call<number>("addon_set", { id, on });
