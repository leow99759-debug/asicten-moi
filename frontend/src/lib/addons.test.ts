import { describe, expect, it } from "vitest";
import type { Addon } from "./bindings/Addon";
import { categoryCount, commandRows, filterPacks, plural, sortPacks, totals } from "./addons";

const cmd = (id: string, name: string, phrases: string[]) =>
  ({ id, name, phrases, folder: [], optional: [], actions: [], reply: { clips: [] }, enabled: true }) as never;

const addon = (id: string, name: string, extra: Partial<Addon["pack"]>, installed = false, def = false): Addon => ({
  pack: { id, name, description: "", category: "", version: "", commands: [], folders: [], ...extra },
  installed,
  default: def,
});

const list = [
  addon("basic", "Основные", { commands: [cmd("b1", "Открыть браузер", ["браузер"])] }, true, true),
  addon("spotify", "Управление Spotify", { category: "Видео, музыка и стриминг", popular: true, commands: [cmd("s1", "Пауза", ["пауза", "стоп музыка"]), cmd("s2", "Дальше", ["следующий трек"])] }, true),
  addon("steam", "Steam", { category: "Игровые сервисы", commands: [cmd("st1", "Открыть Steam", ["стим"])] }),
];

describe("addons", () => {
  it("filters by category, installed and search through commands", () => {
    const ids = (a: Addon[]) => a.map((x) => x.pack.id);
    expect(ids(filterPacks(list, { q: "", cat: "all", only: false }))).toEqual(["basic", "spotify", "steam"]);
    expect(ids(filterPacks(list, { q: "", cat: "default", only: false }))).toEqual(["basic"]);
    expect(ids(filterPacks(list, { q: "", cat: "popular", only: false }))).toEqual(["spotify"]);
    expect(ids(filterPacks(list, { q: "", cat: "all", only: true }))).toEqual(["basic", "spotify"]);
    expect(ids(filterPacks(list, { q: "ПАУЗА", cat: "all", only: false }))).toEqual(["spotify"]);
    expect(categoryCount(list, "Игровые сервисы")).toBe(1);
  });

  it("sorts popular first, defaults last; by size", () => {
    expect(sortPacks(list, "popular").map((a) => a.pack.id)).toEqual(["spotify", "steam", "basic"]);
    expect(sortPacks(list, "size").map((a) => a.pack.id)[0]).toBe("spotify");
  });

  it("flattens commands and counts totals", () => {
    expect(commandRows(list, { q: "трек", cat: "all", only: false }).map((r) => r.cmd.id)).toEqual(["s2"]);
    expect(totals(list)).toEqual({ installed: 2, total: 3, commands: 3 });
  });

  it("russian plurals", () => {
    const w: [string, string, string] = ["пак", "пака", "паков"];
    expect([1, 2, 5, 11, 21, 104].map((n) => plural(n, w))).toEqual(["1 пак", "2 пака", "5 паков", "11 паков", "21 пак", "104 пака"]);
  });
});
