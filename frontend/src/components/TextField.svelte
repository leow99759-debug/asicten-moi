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
  input {
    height: 32px;
    min-width: 0;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--line-2);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    user-select: text;
    transition: border-color var(--t-fast) ease, box-shadow var(--t-fast) ease;
  }
  input:focus {
    outline: none;
    border-color: rgba(var(--accent-rgb), 0.6);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.15);
  }
</style>
