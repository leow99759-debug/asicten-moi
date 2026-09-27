<script lang="ts">
  let {
    value,
    options,
    label,
    onchange,
    disabled = false,
  }: {
    value: string;
    options: { value: string; label: string }[];
    label: string;
    onchange?: (v: string) => void;
    disabled?: boolean;
  } = $props();
</script>

<select aria-label={label} {value} {disabled} onchange={(e) => onchange?.(e.currentTarget.value)}>
  {#each options as o (o.value)}
    <option value={o.value}>{o.label}</option>
  {/each}
</select>

<style>
  select {
    appearance: none;
    height: 32px;
    min-width: 160px;
    max-width: 280px;
    padding: 0 32px 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--line-2);
    background:
      url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23aab' stroke-width='2.5' stroke-linecap='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E")
        no-repeat right 10px center,
      rgba(255, 255, 255, 0.05);
    color: var(--text);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    text-overflow: ellipsis;
  }
  select:disabled {
    opacity: 0.6;
    cursor: default;
  }
  option {
    background: #1d2027;
  }
</style>
