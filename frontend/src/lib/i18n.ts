import ru from "../i18n/ru.json";

export type I18nKey = keyof typeof ru;

const dict: Record<string, string> = ru;

/** Translate a UI key; returns the key itself when missing so gaps are visible. */
export function t(key: I18nKey | string): string {
  return dict[key] ?? key;
}
