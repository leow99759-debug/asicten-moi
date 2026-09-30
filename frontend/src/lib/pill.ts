// Result line of the listening pill (overlay/Pill.svelte): what got done, in a few words.
import type { Outcome } from "./bindings/Outcome";
import { t } from "./i18n";

export type PillResult = { ok: boolean; text: string };

const SHORT = 60;

export function resultLine(o: Outcome): PillResult {
  const last = o.commands.at(-1);
  if (!last) return { ok: false, text: t("pill.unknown") };
  if (last.status === "done") {
    const said = last.steps.map((s) => s.output).find((x) => x && x.length <= SHORT);
    return { ok: true, text: said ?? last.reply.text ?? last.name };
  }
  if (last.status === "no_internet") return { ok: false, text: t("pill.offline") };
  if (last.status === "cancelled") return { ok: false, text: t("pill.cancelled") };
  const err = last.steps.find((s) => s.error)?.error;
  return { ok: false, text: err ? `${last.name}: ${err}` : last.name };
}
