/** KeyboardEvent → "Ctrl+Alt+J" (physical key, so the RU layout still gives J). Null = modifiers only. */
export function comboOf(e: Pick<KeyboardEvent, "code" | "ctrlKey" | "altKey" | "shiftKey" | "metaKey">): string | null {
  const key = e.code.startsWith("Key")
    ? e.code.slice(3)
    : e.code.startsWith("Digit")
      ? e.code.slice(5)
      : /^(F\d{1,2}|Space|Enter|Tab|Backquote|Minus|Equal|Comma|Period|Slash|Semicolon|Quote|BracketLeft|BracketRight|Backslash|Home|End|PageUp|PageDown|Insert|Delete|ArrowUp|ArrowDown|ArrowLeft|ArrowRight|Numpad\w+|Pause)$/.test(e.code)
        ? e.code
        : null;
  if (!key) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean);
  return [...mods, key].join("+");
}
