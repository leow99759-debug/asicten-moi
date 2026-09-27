import { describe, expect, it } from "vitest";
import { fill, norm, samples } from "./phrases";

describe("phrases", () => {
  it("builds SPEC §5.3 preview samples", () => {
    expect(samples({ phrases: ["браузер"], optional: ["открой", "запусти"] })).toEqual([
      "браузер",
      "открой браузер",
      "открой браузер пожалуйста",
    ]);
    expect(samples({ phrases: ["громкость на {число}", " "], optional: [] })).toEqual([
      "громкость на 50",
      "громкость на 50 пожалуйста",
    ]);
    expect(samples({ phrases: ["a", "b", "c", "d", "e"], optional: ["x"] }, 4)).toHaveLength(4);
  });

  it("fills slots and normalizes", () => {
    expect(fill("выключи через {время}")).toBe("выключи через 10 минут");
    expect(fill("{неизвестно}")).toBe("{неизвестно}");
    expect(norm("  Открой   ЁЛКУ ")).toBe("открой елку");
  });
});
