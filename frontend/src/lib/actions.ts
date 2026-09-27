// Action catalogue for the command card (SPEC §5.3): a default per type (the compiler checks
// every type from crates/core/src/commands/action.rs is here), field kinds and quick chips.
import type { Action } from "./bindings/Action";

export type ActionType = Action["type"];

export const DEFAULTS: { [T in ActionType]: Extract<Action, { type: T }> } = {
  "Launch.File": { type: "Launch.File", path: "", args: "", workdir: null, admin: false },
  "Launch.Url": { type: "Launch.Url", url: "https://" },
  "Launch.Uwp": { type: "Launch.Uwp", aumid: "" },
  "Launch.Find": { type: "Launch.Find", name: "" },
  "Process.Kill": { type: "Process.Kill", name: "" },
  "Window.Close": { type: "Window.Close" },
  "Window.Minimize": { type: "Window.Minimize" },
  "Window.Maximize": { type: "Window.Maximize" },
  "Window.Restore": { type: "Window.Restore" },
  "Window.MinimizeAll": { type: "Window.MinimizeAll" },
  "Window.Focus": { type: "Window.Focus", target: "" },
  "Window.Snap": { type: "Window.Snap", side: "left" },
  "Window.MoveToMonitor": { type: "Window.MoveToMonitor", n: 2 },
  "Window.Fullscreen": { type: "Window.Fullscreen" },
  "Keys.Press": { type: "Keys.Press", keys: "" },
  "Keys.Type": { type: "Keys.Type", text: "" },
  "Keys.Hold": { type: "Keys.Hold", key: "", ms: 500 },
  "Mouse.ToCoords": { type: "Mouse.ToCoords", x: 960, y: 540 },
  "Mouse.ClickLeft": { type: "Mouse.ClickLeft", x: null, y: null },
  "Mouse.ClickRight": { type: "Mouse.ClickRight", x: null, y: null },
  "Mouse.Double": { type: "Mouse.Double", x: null, y: null },
  "Mouse.LC": { type: "Mouse.LC", n: 1 },
  "Mouse.Scroll": { type: "Mouse.Scroll", dy: -3 },
  "System.VolumeSet": { type: "System.VolumeSet", level: 50 },
  "System.VolumeUp": { type: "System.VolumeUp", step: 10 },
  "System.VolumeDown": { type: "System.VolumeDown", step: 10 },
  "System.Mute": { type: "System.Mute" },
  "System.Unmute": { type: "System.Unmute" },
  "Media.PlayPause": { type: "Media.PlayPause" },
  "Media.Next": { type: "Media.Next" },
  "Media.Prev": { type: "Media.Prev" },
  "Media.Stop": { type: "Media.Stop" },
  "Audio.SwitchDevice": { type: "Audio.SwitchDevice", name: "" },
  "System.Shutdown": { type: "System.Shutdown", delay_sec: 0 },
  "System.Restart": { type: "System.Restart", delay_sec: 0 },
  "System.Sleep": { type: "System.Sleep" },
  "System.Hibernate": { type: "System.Hibernate" },
  "System.Lock": { type: "System.Lock" },
  "System.Logoff": { type: "System.Logoff" },
  "System.CancelShutdown": { type: "System.CancelShutdown" },
  "System.PowerPlan": { type: "System.PowerPlan", plan: "balanced" },
  "System.Brightness": { type: "System.Brightness", level: 70 },
  "System.Screenshot": { type: "System.Screenshot", region: false },
  "System.OpenSettings": { type: "System.OpenSettings", uri: "ms-settings:" },
  "System.EmptyRecycleBin": { type: "System.EmptyRecycleBin" },
  "System.WiFi": { type: "System.WiFi", on: true },
  "System.Bluetooth": { type: "System.Bluetooth", on: true },
  "System.DND": { type: "System.DND", on: true },
  "System.GameMode": { type: "System.GameMode", on: true },
  "System.Clipboard": { type: "System.Clipboard", text: null },
  "System.Info": { type: "System.Info", what: "time" },
  "Lux.PauseMS": { type: "Lux.PauseMS", ms: 500 },
  "Lux.PauseSec": { type: "Lux.PauseSec", s: 1 },
  "Sound.PlayWav": { type: "Sound.PlayWav", path: "" },
  Speak: { type: "Speak", text: "", clip: null },
  Ask: { type: "Ask", question: "" },
  "Run.Command": { type: "Run.Command", cmd: "", powershell: false, trusted: false },
  "Run.Command.Hidden": { type: "Run.Command.Hidden", cmd: "", powershell: false, trusted: false },
  Timer: { type: "Timer", sec: 60, then_command: "" },
  Reminder: { type: "Reminder", sec: 600, text: "" },
  "Assistant.Mode": { type: "Assistant.Mode", mode: "silent", on: true },
  "Assistant.SetTheme": { type: "Assistant.SetTheme", color: "синий" },
  "Assistant.OpenPage": { type: "Assistant.OpenPage", page: "editor" },
  "Assistant.Repeat": { type: "Assistant.Repeat" },
  "Assistant.Cancel": { type: "Assistant.Cancel" },
  "PowerPoint.NewPresentation": { type: "PowerPoint.NewPresentation", topic: "", slides: null },
  "PowerPoint.ApplyTheme": { type: "PowerPoint.ApplyTheme", name: "" },
  "PowerPoint.SetVariantColor": { type: "PowerPoint.SetVariantColor", color: "" },
  "Word.Bold": { type: "Word.Bold" },
  "Word.Italic": { type: "Word.Italic" },
  "Word.NewParagraph": { type: "Word.NewParagraph" },
  "Word.Print": { type: "Word.Print" },
  "Word.Save": { type: "Word.Save" },
  "Word.Dictate": { type: "Word.Dictate", text: "" },
  "UIA.Click": { type: "UIA.Click", name: "", window: null },
  "UIA.SetText": { type: "UIA.SetText", name: "", text: "", window: null },
};

export const TYPES = Object.keys(DEFAULTS) as ActionType[];

/** Types grouped by prefix for the type picker. */
export const GROUPS: [string, ActionType[]][] = Object.entries(
  TYPES.reduce<Record<string, ActionType[]>>((g, t) => ((g[t.includes(".") ? t.split(".")[0] : "Assistant"] ??= []).push(t), g), {}),
);

/** Quick chips under «+ Действие» (video 26): Программа, Клавиши, Клик, Сайт, Пауза. */
export const QUICK: [string, ActionType][] = [
  ["act.quick.app", "Launch.File"],
  ["act.quick.keys", "Keys.Press"],
  ["act.quick.click", "Mouse.ClickLeft"],
  ["act.quick.site", "Launch.Url"],
  ["act.quick.pause", "Lux.PauseMS"],
];

export type FieldKind = "text" | "num" | "bool" | "file" | { options: string[] };

const ENUMS: Record<string, string[]> = {
  side: ["left", "right", "top"],
  plan: ["balanced", "high", "saver"],
  mode: ["prefix", "silent", "mic_off"],
};
/** Nullable fields that hold numbers (the rest of the nullable ones are text). */
const NULL_NUM = new Set(["x", "y", "slides"]);

export function fieldKind(type: ActionType, key: string): FieldKind {
  const v = (DEFAULTS[type] as Record<string, unknown>)[key];
  if (ENUMS[key]) return { options: ENUMS[key] };
  if (key === "path") return "file";
  if (typeof v === "boolean") return "bool";
  if (typeof v === "number" || (v === null && NULL_NUM.has(key))) return "num";
  return "text";
}

export const fields = (type: ActionType) => Object.keys(DEFAULTS[type]).filter((k) => k !== "type");

/** Parsed input: numbers stay numbers, `{число}` slots and empty optional values pass through. */
export function parseValue(kind: FieldKind, raw: string, nullable: boolean): unknown {
  const s = raw.trim();
  if (nullable && s === "") return null;
  if (kind === "num") return /^-?\d+(\.\d+)?$/.test(s) ? Number(s) : s;
  return raw;
}

/** New action of another type, keeping fields that exist in both. */
export function retype(a: Action, type: ActionType): Action {
  const next = { ...DEFAULTS[type] } as Record<string, unknown>;
  for (const k of Object.keys(next)) if (k !== "type" && k in a) next[k] = (a as Record<string, unknown>)[k];
  return next as Action;
}

/** «Launch.File» → app name for the row icon label: file name without extension, %VAR% as is. */
export function appName(path: string): string {
  const base = path.split(/[\\/]/).pop() ?? "";
  return base.replace(/\.(exe|lnk|bat|cmd)$/i, "");
}
