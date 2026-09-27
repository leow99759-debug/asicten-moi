import { describe, expect, it } from "vitest";
import type { Command } from "./bindings/Command";
import { allFolders, rebase, rows, uniqueName } from "./tree";

const cmd = (id: string, folder: string[], name: string, phrases: string[] = []): Command => ({
  id, folder, name, phrases, optional: [], actions: [], reply: { clips: ["ok"], text: null },
  confirm: false, chainable: true, slots: {}, when: null, enabled: true,
});

const cmds = [
  cmd("a", ["Браузер"], "Открыть браузер", ["браузер", "интернет"]),
  cmd("b", ["Браузер", "Вкладки"], "Новая вкладка", ["новая вкладка"]),
  cmd("c", [], "Который час", ["сколько времени"]),
];

describe("tree.rows", () => {
  it("nests folders before commands and respects open state", () => {
    const closed = rows(cmds, [["Пустая"]], {}, "", "Джарвис");
    expect(closed.map((r) => r.label)).toEqual(["Джарвис", "Браузер", "Пустая", "Который час"]);
    expect(closed[0]).toMatchObject({ kind: "root", count: 3 });
    expect(closed[1]).toMatchObject({ count: 2, depth: 1 });
    const open = rows(cmds, [], { "f:Браузер": true, "c:a": true }, "", "Д");
    expect(open.map((r) => `${r.depth}${r.label}`)).toEqual(["0Д", "1Браузер", "2Вкладки", "2Открыть браузер", "3браузер", "3интернет", "1Который час"]);
  });

  it("search keeps matches, expands the way, shows matching phrases (ё = е)", () => {
    const r = rows(cmds, [["Пустая"]], {}, "ИНТЕРНЁТ".replace("Ё", "Е"), "Д");
    expect(r.map((x) => x.label)).toEqual(["Д", "Браузер", "Открыть браузер", "интернет"]);
    expect(rows(cmds, [], {}, "zzz", "Д")).toHaveLength(1);
  });
});

describe("tree helpers", () => {
  it("allFolders lists explicit and implied folders", () => {
    expect(allFolders(cmds, [["Пустая"]]).map((f) => f.join("/"))).toEqual(["Пустая", "Браузер", "Браузер/Вкладки"]);
  });
  it("rebase and uniqueName", () => {
    expect(rebase(["A", "B", "C"], ["A", "B"], ["X"])).toEqual(["X", "C"]);
    expect(rebase(["Z"], ["A"], ["X"])).toEqual(["Z"]);
    expect(uniqueName("Новая папка", ["Новая папка", "Новая папка 2"])).toBe("Новая папка 3");
  });
});
