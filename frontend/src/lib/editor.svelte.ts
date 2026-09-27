// Command editor state (SPEC §5.1). Edits stay local until «Сохранить» sends the whole set;
// the core keeps only own commands + changed built-ins (crates/core/src/commands/library.rs).
import type { Command } from "./bindings/Command";
import type { Library } from "./bindings/Library";
import { editorLibrary, editorSave } from "./commands";
import { t } from "./i18n";
import { allFolders, ckey, fkey, pkey, rebase, ROOT, startsWith, uniqueName } from "./tree";
import { inTauri } from "./window";

type Meta = { builtin: boolean; modified: boolean };

export const ed = $state({
  cmds: [] as Command[],
  meta: {} as Record<string, Meta>,
  folders: [] as string[][],
  open: {} as Record<string, boolean>,
  selected: ROOT,
  /** Open command tabs (SPEC §5.2), ids. */
  tabs: [] as string[],
  editing: null as string | null,
  query: "",
  dirty: false,
  saving: false,
  error: "",
});

export type Sel =
  | { kind: "root"; path: string[] }
  | { kind: "folder"; path: string[] }
  | { kind: "command"; cmd: Command; path: string[] }
  | { kind: "phrase"; cmd: Command; index: number; path: string[] };

export const cmdById = (id: string) => ed.cmds.find((c) => c.id === id);

/** What a tree key points at (`path` = folder the new items go to). */
export function resolve(key: string): Sel {
  if (key.startsWith("f:")) {
    const path = key.slice(2).split("/");
    return { kind: "folder", path };
  }
  if (key.startsWith("c:")) {
    const cmd = cmdById(key.slice(2));
    if (cmd) return { kind: "command", cmd, path: cmd.folder };
  }
  if (key.startsWith("p:")) {
    const [, i, ...rest] = key.split(":");
    const cmd = cmdById(rest.join(":"));
    if (cmd) return { kind: "phrase", cmd, index: Number(i), path: cmd.folder };
  }
  return { kind: "root", path: [] };
}

/** Why the brain would skip a command (mirrors core `validate`, short labels for the tree). */
export function issues(c: Command): string[] {
  const out: string[] = [];
  if (c.phrases.every((p) => !p.trim())) out.push(t("editor.issue.phrases"));
  if (!c.actions.length && !c.reply.clips.length && c.reply.text == null) out.push(t("editor.issue.actions"));
  return out;
}

function apply(lib: Library) {
  ed.cmds = lib.entries.map((e) => e.command);
  ed.tabs = ed.tabs.filter((id) => ed.cmds.some((c) => c.id === id));
  ed.meta = Object.fromEntries(lib.entries.map((e) => [e.command.id, { builtin: e.builtin, modified: e.modified }]));
  ed.folders = lib.folders;
  ed.dirty = false;
}

/** Browser preview: built-in packs straight from the repo (never bundled into the app). */
async function demo(): Promise<Library> {
  const files = import.meta.glob<{ default: { commands: Command[] } }>("../../../packs/*.json");
  const packs = await Promise.all(Object.values(files).map((f) => f()));
  const entries = packs
    .flatMap((p) => p.default.commands)
    .map((c) => ({ command: { ...c, folder: c.folder ?? [], optional: c.optional ?? [], enabled: true }, builtin: true, modified: false, issues: [] }));
  return { entries: entries as Library["entries"], folders: [] };
}

export async function load() {
  const lib = inTauri() ? await editorLibrary() : await demo();
  if (lib) apply(lib);
  if (!Object.keys(ed.open).length) ed.open = { [ROOT]: true };
}

export async function save() {
  if (!ed.dirty || ed.saving) return;
  ed.saving = true;
  ed.error = "";
  try {
    const lib = await editorSave($state.snapshot(ed.cmds), allFolders(ed.cmds, ed.folders));
    if (lib) apply(lib);
    else ed.dirty = false;
  } catch (e) {
    ed.error = String(e);
  } finally {
    ed.saving = false;
  }
}

export const touch = () => (ed.dirty = true);
const expand = (...keys: string[]) => keys.forEach((k) => (ed.open[k] = true));
const expandTo = (path: string[]) => expand(ROOT, ...path.map((_, i) => fkey(path.slice(0, i + 1))));

function select(key: string, rename = false) {
  ed.selected = key;
  ed.editing = rename ? key : null;
}

export function addFolder() {
  const parent = resolve(ed.selected).path;
  const siblings = allFolders(ed.cmds, ed.folders)
    .filter((f) => f.length === parent.length + 1 && startsWith(f, parent))
    .map((f) => f.at(-1)!);
  const path = [...parent, uniqueName(t("editor.new_folder"), siblings)];
  ed.folders.push(path);
  expandTo(path);
  select(fkey(path), true);
  touch();
}

export function addCommand() {
  const folder = [...resolve(ed.selected).path];
  const cmd: Command = {
    id: `user.${Date.now().toString(36)}`,
    folder,
    name: uniqueName(t("editor.new_command"), ed.cmds.map((c) => c.name)),
    phrases: [],
    optional: [],
    actions: [],
    reply: { clips: ["ok"], text: null },
    confirm: false,
    chainable: true,
    slots: {},
    when: null,
    enabled: true,
  };
  ed.cmds.push(cmd);
  ed.meta[cmd.id] = { builtin: false, modified: false };
  expandTo(folder);
  select(ckey(cmd.id), true);
  touch();
}

export function addPhrase() {
  const s = resolve(ed.selected);
  if (s.kind !== "command" && s.kind !== "phrase") return;
  s.cmd.phrases.push(t("editor.new_phrase"));
  expand(ckey(s.cmd.id));
  select(pkey(s.cmd.id, s.cmd.phrases.length - 1), true);
  touch();
}

/** Built-ins can't be deleted (only switched off); nor can folders holding them. */
export function canDelete(key: string): boolean {
  const s = resolve(key);
  if (s.kind === "root") return false;
  if (s.kind === "command") return !ed.meta[s.cmd.id]?.builtin;
  if (s.kind === "folder") return !ed.cmds.some((c) => startsWith(c.folder, s.path) && ed.meta[c.id]?.builtin);
  return true;
}

export function remove(key = ed.selected) {
  if (!canDelete(key)) return;
  const s = resolve(key);
  if (s.kind === "phrase") {
    s.cmd.phrases.splice(s.index, 1);
    select(ckey(s.cmd.id));
  } else if (s.kind === "command") {
    ed.cmds = ed.cmds.filter((c) => c.id !== s.cmd.id);
    ed.tabs = ed.tabs.filter((id) => id !== s.cmd.id);
    select(fkey(s.path));
  } else if (s.kind === "folder") {
    ed.cmds = ed.cmds.filter((c) => !startsWith(c.folder, s.path));
    ed.tabs = ed.tabs.filter((id) => ed.cmds.some((c) => c.id === id));
    ed.folders = ed.folders.filter((f) => !startsWith(f, s.path));
    select(fkey(s.path.slice(0, -1)));
  }
  touch();
}

export const canDuplicate = (key: string) => resolve(key).kind === "command";

export function duplicate() {
  const s = resolve(ed.selected);
  if (s.kind !== "command") return;
  const copy: Command = { ...$state.snapshot(s.cmd), id: `user.${Date.now().toString(36)}` };
  copy.name = uniqueName(`${s.cmd.name} ${t("editor.copy_suffix")}`, ed.cmds.map((c) => c.name));
  ed.cmds.push(copy);
  ed.meta[copy.id] = { builtin: false, modified: false };
  select(ckey(copy.id), true);
  touch();
}

/** Inline rename from the tree (folder name, command name or phrase text). */
export function rename(key: string, value: string) {
  ed.editing = null;
  const v = value.trim();
  const s = resolve(key);
  if (!v) return;
  if (s.kind === "folder" && v !== s.path.at(-1)) {
    const to = [...s.path.slice(0, -1), v.replaceAll("/", "∕")];
    moveFolder(s.path, to);
    select(fkey(to));
  } else if (s.kind === "command" && v !== s.cmd.name) {
    s.cmd.name = v;
    touch();
  } else if (s.kind === "phrase" && v !== s.cmd.phrases[s.index]) {
    s.cmd.phrases[s.index] = v;
    touch();
  }
}

function moveFolder(from: string[], to: string[]) {
  for (const c of ed.cmds) if (startsWith(c.folder, from)) c.folder = rebase(c.folder, from, to);
  ed.folders = ed.folders.map((f) => rebase(f, from, to));
  if (!ed.folders.some((f) => f.join("/") === to.join("/"))) ed.folders.push(to);
  const was = ed.open[fkey(from)];
  expandTo(to.slice(0, -1));
  if (was) ed.open[fkey(to)] = true;
  touch();
}

/** Drag & drop: folder/command into a folder (or root), phrase into another command. */
export function canDrop(drag: string, target: string): boolean {
  const d = resolve(drag);
  const tg = resolve(target);
  if (drag === target) return false;
  if (d.kind === "phrase") return (tg.kind === "command" || tg.kind === "phrase") && tg.cmd.id !== d.cmd.id;
  if (tg.kind !== "folder" && tg.kind !== "root") return false;
  if (d.kind === "command") return d.cmd.folder.join("/") !== tg.path.join("/");
  if (d.kind === "folder") return !startsWith(tg.path, d.path) && d.path.slice(0, -1).join("/") !== tg.path.join("/");
  return false;
}

export function drop(drag: string, target: string) {
  if (!canDrop(drag, target)) return;
  const d = resolve(drag);
  const tg = resolve(target);
  if (d.kind === "phrase" && (tg.kind === "command" || tg.kind === "phrase")) {
    const [p] = d.cmd.phrases.splice(d.index, 1);
    tg.cmd.phrases.push(p);
    expand(ckey(tg.cmd.id));
    select(pkey(tg.cmd.id, tg.cmd.phrases.length - 1));
    touch();
  } else if (d.kind === "command") {
    // the old folder stays even when this was its last command
    if (d.path.length && !ed.folders.some((f) => f.join("/") === d.path.join("/"))) ed.folders.push([...d.path]);
    d.cmd.folder = [...tg.path];
    expandTo(tg.path);
    touch();
  } else if (d.kind === "folder") {
    const to = [...tg.path, d.path.at(-1)!];
    moveFolder(d.path, to);
    select(fkey(to));
  }
}

export function setEnabled(id: string, on: boolean) {
  const c = cmdById(id);
  if (c && c.enabled !== on) {
    c.enabled = on;
    touch();
  }
}

/** Keep a tab for the command being looked at (max 6, oldest inactive one drops). */
export function openTab(id: string) {
  if (ed.tabs.includes(id)) return;
  ed.tabs.push(id);
  if (ed.tabs.length > 6) ed.tabs.splice(0, 1);
}

export function closeTab(id: string) {
  ed.tabs = ed.tabs.filter((t) => t !== id);
  const s = resolve(ed.selected);
  if ((s.kind === "command" || s.kind === "phrase") && s.cmd.id === id) ed.selected = ed.tabs.length ? ckey(ed.tabs.at(-1)!) : ROOT;
}
