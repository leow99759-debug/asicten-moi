<script lang="ts">
  // Fluent checkbox as a bordered chip (video 26: «Связывать», «Подтверждать»).
  import Icon from "./Icon.svelte";
  let { checked, label, title, onchange }: { checked: boolean; label: string; title?: string; onchange: (v: boolean) => void } =
    $props();
</script>

<button type="button" role="checkbox" aria-checked={checked} class="cb pressable" class:on={checked} {title} onclick={() => onchange(!checked)}>
  <span class="box"><Icon name="check" size={12} stroke={2.5} /></span>
  {label}
</button>

<style>
  .cb {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 12px 0 10px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    background: var(--fill);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }
  .cb:hover {
    background: var(--fill-hover);
  }
  .box {
    width: 16px;
    height: 16px;
    display: grid;
    place-items: center;
    border-radius: var(--r-xs);
    border: 1px solid rgba(255, 255, 255, 0.5);
    color: transparent;
    transition:
      background-color var(--t-fast) ease,
      border-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .on .box {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--on-accent);
  }
</style>
