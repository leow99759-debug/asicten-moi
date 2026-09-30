import { describe, expect, it } from "vitest";
import type { Outcome } from "./bindings/Outcome";
import { resultLine } from "./pill";

const out = (status: string, steps: object[], text: string | null = null): Outcome =>
  ({
    phrase: "x",
    commands: [{ id: "a", name: "Скриншот", status, steps, reply: { clips: ["ok"], text } }],
  }) as never;

describe("pill result", () => {
  it("prefers a short spoken output, then the reply, then the name", () => {
    expect(resultLine(out("done", [{ output: "Сейчас 10:31" }]))).toEqual({ ok: true, text: "Сейчас 10:31" });
    expect(resultLine(out("done", [{ output: null }], "Снимок сохранён"))).toEqual({ ok: true, text: "Снимок сохранён" });
    expect(resultLine(out("done", [{ output: "x".repeat(200) }]))).toEqual({ ok: true, text: "Скриншот" });
  });
  it("explains failures", () => {
    expect(resultLine({ phrase: "?", commands: [] })).toEqual({ ok: false, text: "Не понял команду" });
    expect(resultLine(out("error", [{ error: "экран не найден" }]))).toEqual({ ok: false, text: "Скриншот: экран не найден" });
    expect(resultLine(out("no_internet", [])).text).toBe("Нет интернета");
  });
});
