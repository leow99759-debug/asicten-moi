import { describe, expect, it } from "vitest";
import { only } from "./ipc";
import type { CoreEvent } from "./bindings/CoreEvent";

describe("ipc.only", () => {
  it("passes matching payload and ignores other kinds", () => {
    const got: string[] = [];
    const h = only("transcript", (p) => got.push(p.text));
    const events: CoreEvent[] = [
      { event: "state", payload: "listening" },
      { event: "transcript", payload: { text: "джарвис", is_final: false } },
      { event: "level", payload: { mic: 0.5, tts: 0 } },
    ];
    events.forEach(h);
    expect(got).toEqual(["джарвис"]);
  });
});
