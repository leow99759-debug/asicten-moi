<script lang="ts">
  let {
    value,
    label,
    placeholder = "",
    onchange,
  }: { value: string; label: string; placeholder?: string; onchange?: (v: string) => void } = $props();
</script>

<input type="text" aria-label={label} {placeholder} {value} onchange={(e) => onchange?.(e.currentTarget.value.trim())} />

<style>
  /* Fluent text box: accent underline on focus */
  input {
    height: 32px;
    min-width: 0;
    box-sizing: border-box;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    color: var(--text);
    font-size: 13px;
    font-weight: 450;
    user-select: text;
    transition:
      background-color var(--t-fast) ease,
      box-shadow var(--t-fast) ease;
  }
  input:hover {
    background: var(--fill-hover);
  }
  input:focus {
    outline: none;
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
</style>
