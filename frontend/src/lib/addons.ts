// «Дополнения» (SPEC §9): pure catalog filtering/sorting, shared by the page and tests.
import type { Addon } from "./bindings/Addon";
import type { Command } from "./bindings/Command";

export const ALL = "all";
export const POPULAR = "popular";
export const DEFAULT = "default";
/** Right-hand category list; the middle ones are `pack.category` values. */
export const CATEGORIES = [
  ALL,
  POPULAR,
  "Система",
  "Браузеры",
  "Соцсети и общение",
  "Дизайн и творчество",
  "Видео, музыка и стриминг",
  "Работа и разработка",
  "Игровые сервисы",
  DEFAULT,
] as const;

export type Sort = "popular" | "name" | "size";
export type Filter = { q: string; cat: string; only: boolean };

export const norm = (s: string) => s.toLowerCase().replaceAll("ё", "е").trim();

export function inCategory(a: Addon, cat: string): boolean {
  if (cat === ALL) return true;
  if (cat === DEFAULT) return a.default;
  if (cat === POPULAR) return !!a.pack.popular;
  return !a.default && a.pack.category === cat;
}

const cmdHit = (c: Command, q: string) => norm(c.name).includes(q) || c.phrases.some((p) => norm(p).includes(q));

/** Search covers the pack text and its commands, so «пауза» finds Spotify. */
export function filterPacks(list: Addon[], f: Filter): Addon[] {
  const q = norm(f.q);
  return list.filter(
    (a) =>
      inCategory(a, f.cat) &&
      (!f.only || a.installed) &&
      (!q || norm(a.pack.name).includes(q) || norm(a.pack.description).includes(q) || a.pack.commands.some((c) => cmdHit(c, q))),
  );
}

export function sortPacks(list: Addon[], by: Sort): Addon[] {
  const name = (a: Addon, b: Addon) => a.pack.name.localeCompare(b.pack.name, "ru");
  const cmp: Record<Sort, (a: Addon, b: Addon) => number> = {
    popular: (a, b) => Number(!!b.pack.popular) - Number(!!a.pack.popular) || Number(a.default) - Number(b.default) || name(a, b),
    name,
    size: (a, b) => b.pack.commands.length - a.pack.commands.length || name(a, b),
  };
  return [...list].sort(cmp[by]);
}

export type Row = { cmd: Command; addon: Addon };

export function commandRows(list: Addon[], f: Filter): Row[] {
  const q = norm(f.q);
  return list
    .filter((a) => inCategory(a, f.cat) && (!f.only || a.installed))
    .flatMap((addon) => addon.pack.commands.map((cmd) => ({ cmd, addon })))
    .filter((r) => !q || cmdHit(r.cmd, q));
}

/** «Установлено: N/M» + «N паков · M команд» (installed). */
export function totals(list: Addon[]) {
  const on = list.filter((a) => a.installed);
  return {
    installed: on.length,
    total: list.length,
    commands: on.reduce((n, a) => n + a.pack.commands.length, 0),
  };
}

export function categoryCount(list: Addon[], cat: string): number {
  return list.filter((a) => inCategory(a, cat)).length;
}

/** Russian plural: 1 пак, 2 пака, 5 паков. */
export function plural(n: number, [one, few, many]: [string, string, string]): string {
  const m10 = n % 10;
  const m100 = n % 100;
  const w = m10 === 1 && m100 !== 11 ? one : m10 >= 2 && m10 <= 4 && (m100 < 12 || m100 > 14) ? few : many;
  return `${n} ${w}`;
}
