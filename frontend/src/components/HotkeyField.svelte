<script lang="ts">
  // Click, press the combo: it is recorded as-is (no typing «Ctrl+Alt+J» by hand).
  import { comboOf } from "../lib/hotkey";
  let { value, label, onchange }: { value: string; label: string; onchange?: (v: string) => void } = $props();
  let recording = $state(false);

  function keydown(e: KeyboardEvent) {
    if (!recording) return;
    e.preventDefault();
    if (e.code === "Escape") {
      recording = false;
      return;
    }
    const combo = comboOf(e);
    // a bare letter would fire while typing anywhere: require a modifier (F-keys are fine alone)
    if (!combo || (!combo.includes("+") && !/^F\d/.test(combo))) return;
    recording = false;
    if (combo !== value) onchange?.(combo);
  }
</script>

<button type="button" class="hk" class:rec={recording} aria-label={label} onclick={() => (recording = !recording)} onkeydown={keydown} onblur={() => (recording = false)}>
  {recording ? "Нажмите сочетание…" : value || "—"}
</button>

<style>
  .hk {
    height: 32px;
    min-width: 160px;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    background: var(--fill);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      box-shadow var(--t-fast) ease;
  }
  .hk:hover {
    background: var(--fill-hover);
  }
  .hk.rec {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.25);
    color: var(--accent);
  }
</style>
