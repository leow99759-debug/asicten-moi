// Command editor tree (SPEC §5.1): folders → commands → phrases, flattened into visible
// rows so rendering and keyboard navigation are a plain list. Pure, unit-tested.
import type { Command } from "./bindings/Command";

export type Row =
  | { kind: "root"; key: string; depth: number; label: string; count: number; open: boolean }
  | { kind: "folder"; key: string; depth: number; path: string[]; label: string; count: number; open: boolean }
  | { kind: "command"; key: string; depth: number; id: string; label: string; count: number; open: boolean; off: boolean }
  | { kind: "phrase"; key: string; depth: number; id: string; index: number; label: string };

export const ROOT = "r";
export const fkey = (path: string[]) => (path.length ? "f:" + path.join("/") : ROOT);
export const ckey = (id: string) => "c:" + id;
export const pkey = (id: string, i: number) => `p:${i}:${id}`;

export const norm = (s: string) => s.toLowerCase().replaceAll("ё", "е").trim();
export const startsWith = (path: string[], prefix: string[]) => prefix.every((p, i) => path[i] === p);
/** `path` moved from under `from` to under `to` (unchanged when it isn't under `from`). */
export const rebase = (path: string[], from: string[], to: string[]) =>
  startsWith(path, from) ? [...to, ...path.slice(from.length)] : path;

/** «Новая папка», «Новая папка 2», … — first name not in `taken`. */
export function uniqueName(base: string, taken: string[]): string {
  let n = 1;
  let name = base;
  while (taken.includes(name)) name = `${base} ${++n}`;
  return name;
}

type Node = { path: string[]; kids: Map<string, Node>; cmds: Command[] };

function node(root: Node, path: string[]): Node {
  let n = root;
  path.forEach((name, i) => {
    let k = n.kids.get(name);
    if (!k) n.kids.set(name, (k = { path: path.slice(0, i + 1), kids: new Map(), cmds: [] }));
    n = k;
  });
  return n;
}

const byName = new Intl.Collator("ru", { numeric: true }).compare;

/** Every folder path: explicit (empty) ones plus those commands live in. */
export function allFolders(cmds: Command[], folders: string[][]): string[][] {
  const root: Node = { path: [], kids: new Map(), cmds: [] };
  folders.forEach((f) => node(root, f));
  cmds.forEach((c) => node(root, c.folder));
  const out: string[][] = [];
  const walk = (n: Node) => n.kids.forEach((k) => (out.push(k.path), walk(k)));
  walk(root);
  return out;
}

/**
 * Visible rows. `open` holds expanded keys; a non-empty `query` keeps only matching commands
 * (by name or phrase) and expands everything on the way to them.
 */
export function rows(cmds: Command[], folders: string[][], open: Record<string, boolean>, query: string, rootLabel: string): Row[] {
  const q = norm(query);
  const phraseHits = (c: Command) => c.phrases.map((p, i) => [p, i] as const).filter(([p]) => !q || norm(p).includes(q));
  const matches = (c: Command) => !q || norm(c.name).includes(q) || phraseHits(c).length > 0;

  const root: Node = { path: [], kids: new Map(), cmds: [] };
  folders.forEach((f) => node(root, f));
  cmds.filter(matches).forEach((c) => node(root, c.folder).cmds.push(c));
  const count = (n: Node): number => n.cmds.length + [...n.kids.values()].reduce((s, k) => s + count(k), 0);
  const isOpen = (key: string) => !!q || !!open[key];

  const out: Row[] = [];
  const walk = (n: Node, depth: number) => {
    const kids = [...n.kids.values()].filter((k) => !q || count(k) > 0).sort((a, b) => byName(a.path.at(-1)!, b.path.at(-1)!));
    for (const k of kids) {
      const key = fkey(k.path);
      out.push({ kind: "folder", key, depth, path: k.path, label: k.path.at(-1)!, count: count(k), open: isOpen(key) });
      if (isOpen(key)) walk(k, depth + 1);
    }
    for (const c of [...n.cmds].sort((a, b) => byName(a.name, b.name))) {
      const key = ckey(c.id);
      const hits = q && !norm(c.name).includes(q) ? phraseHits(c) : c.phrases.map((p, i) => [p, i] as const);
      out.push({ kind: "command", key, depth, id: c.id, label: c.name, count: c.phrases.length, open: isOpen(key), off: !c.enabled });
      if (isOpen(key)) for (const [p, i] of hits) out.push({ kind: "phrase", key: pkey(c.id, i), depth: depth + 1, id: c.id, index: i, label: p });
    }
  };
  const rootOpen = !!q || open[ROOT] !== false;
  out.push({ kind: "root", key: ROOT, depth: 0, label: rootLabel, count: count(root), open: rootOpen });
  if (rootOpen) walk(root, 1);
  return out;
}
