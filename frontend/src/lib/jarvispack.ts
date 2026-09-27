// .jarvispack export/import (SPEC §5.6): a Pack JSON. Folders are stored relative to what was
// exported and land under the folder selected on import; imported commands get fresh ids.
import type { Command } from "./bindings/Command";
import type { Pack } from "./bindings/Pack";
import { startsWith } from "./tree";

export const EXT = "jarvispack";

/** Pack of one command (`id`) or of a folder subtree (`path`, [] = everything). */
export function packOf(cmds: Command[], folders: string[][], pick: { id: string } | { path: string[] }, name: string): Pack {
  let list: Command[];
  let cut = 0;
  let empty: string[][] = [];
  if ("id" in pick) {
    list = cmds.filter((c) => c.id === pick.id).map((c) => ({ ...c, folder: [] }));
  } else {
    cut = Math.max(0, pick.path.length - 1); // keep the exported folder's own name
    list = cmds.filter((c) => startsWith(c.folder, pick.path)).map((c) => ({ ...c, folder: c.folder.slice(cut) }));
    empty = folders.filter((f) => startsWith(f, pick.path)).map((f) => f.slice(cut));
  }
  return { id: `export.${name}`, name, description: "", category: "", version: "1", commands: list, folders: empty };
}

/** Commands of an imported pack placed under `target` with ids that can't clash. */
export function adopt(pack: Pack, target: string[], stamp = Date.now().toString(36)): { commands: Command[]; folders: string[][] } {
  return {
    commands: pack.commands.map((c, i) => ({
      ...c,
      id: `user.${stamp}${i.toString(36)}`,
      folder: [...target, ...(c.folder ?? [])],
      optional: c.optional ?? [],
      enabled: c.enabled ?? true,
    })),
    folders: (pack.folders ?? []).map((f) => [...target, ...f]),
  };
}
