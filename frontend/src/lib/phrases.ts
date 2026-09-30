// Phrase helpers for the command card (SPEC §5.3): slot examples, preview samples, reply clips.
import type { Command } from "./bindings/Command";

/** Values the live preview and «▶ Тест» put into slots. */
export const SLOT_EXAMPLE: Record<string, string> = {
  "{число}": "50",
  "{время}": "10 минут",
  "{текст}": "привет",
  "{приложение}": "телеграм",
  "{кому}": "маме",
};
export const SLOTS = Object.keys(SLOT_EXAMPLE);

export const fill = (phrase: string) => phrase.replace(/\{[^}]*\}/g, (s) => SLOT_EXAMPLE[s] ?? s);

/** Comparison key: case, ё and spacing don't make a phrase different. */
export const norm = (s: string) => s.toLowerCase().replace(/ё/g, "е").trim().replace(/\s+/g, " ");

/** What a user might say: «браузер», «открой браузер», «открой браузер пожалуйста», … */
export function samples(c: Pick<Command, "phrases" | "optional">, max = 8): string[] {
  const opt = c.optional.map((o) => o.trim()).find(Boolean);
  const out = new Set<string>();
  c.phrases
    .map((p) => p.trim())
    .filter(Boolean)
    .forEach((p, i) => {
      const f = fill(p);
      out.add(f);
      if (opt) out.add(`${opt} ${f}`);
      if (i === 0) out.add(`${opt ? `${opt} ` : ""}${f} пожалуйста`);
    });
  return [...out].slice(0, max);
}

/** Voice pack categories offered as replies (§6.1), label keys `reply.<cat>`. */
export const REPLY_CLIPS = [
  "ok",
  "done",
  "loading",
  "ready",
  "thanks",
  "status",
  "greet",
  "off",
  "joke",
  "calibration",
  "diagnostics",
  "cancel",
] as const;
