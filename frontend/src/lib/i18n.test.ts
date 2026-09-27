import { describe, expect, it } from "vitest";
import { t } from "./i18n";

describe("i18n", () => {
  it("returns Russian string for known key", () => {
    expect(t("window.close")).toBe("Закрыть");
  });
  it("falls back to key for unknown key", () => {
    expect(t("nope.missing")).toBe("nope.missing");
  });
});
