import { describe, expect, it } from "vitest";
import { comboOf } from "./hotkey";

const k = (code: string, m: Partial<KeyboardEvent> = {}) =>
  comboOf({ code, ctrlKey: false, altKey: false, shiftKey: false, metaKey: false, ...m });

describe("hotkey capture", () => {
  it("uses the physical key, so the RU layout still records latin letters", () => {
    expect(k("KeyJ", { ctrlKey: true, altKey: true })).toBe("Ctrl+Alt+J");
    expect(k("Digit5", { shiftKey: true })).toBe("Shift+5");
    expect(k("F9")).toBe("F9");
  });
  it("ignores bare modifiers", () => {
    expect(k("ControlLeft", { ctrlKey: true })).toBeNull();
  });
});
