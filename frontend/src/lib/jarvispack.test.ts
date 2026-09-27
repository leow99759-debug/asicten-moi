import { describe, expect, it } from "vitest";
import type { Command } from "./bindings/Command";
import { adopt, packOf } from "./jarvispack";

const cmd = (id: string, folder: string[]) =>
  ({ id, folder, name: id, phrases: [id], optional: [], actions: [], reply: { clips: ["ok"], text: null }, confirm: false, chainable: true, slots: {}, when: null, enabled: true }) as Command;

describe("jarvispack", () => {
  const cmds = [cmd("a", ["Игры"]), cmd("b", ["Игры", "Steam"]), cmd("c", ["Работа"])];

  it("exports a folder relative to itself, or one command", () => {
    const p = packOf(cmds, [["Игры", "Пусто"]], { path: ["Игры"] }, "Игры");
    expect(p.commands.map((c) => [c.id, c.folder])).toEqual([["a", ["Игры"]], ["b", ["Игры", "Steam"]]]);
    expect(p.folders).toEqual([["Игры", "Пусто"]]);
    const one = packOf(cmds, [], { id: "c" }, "c");
    expect(one.commands).toHaveLength(1);
    expect(one.commands[0].folder).toEqual([]);
    expect(packOf(cmds, [], { path: [] }, "все").commands).toHaveLength(3);
  });

  it("imports under the target with fresh ids", () => {
    const p = packOf(cmds, [["Игры", "Пусто"]], { path: ["Игры"] }, "Игры");
    const r = adopt(p, ["Мои"], "x");
    expect(r.commands.map((c) => [c.id, c.folder.join("/")])).toEqual([["user.x0", "Мои/Игры"], ["user.x1", "Мои/Игры/Steam"]]);
    expect(r.folders).toEqual([["Мои", "Игры", "Пусто"]]);
  });
});
