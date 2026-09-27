<script lang="ts">
  // Windows quick-settings tile: icon + label, accent fill when on.
  import Icon, { type IconName } from "./Icon.svelte";
  let {
    icon,
    label,
    checked = false,
    onchange,
  }: { icon: IconName; label: string; checked?: boolean; onchange?: (v: boolean) => void } = $props();
</script>

<button type="button" class="tile" class:on={checked} aria-pressed={checked} onclick={() => onchange?.(!checked)}>
  <Icon name={icon} size={18} />
  <span>{label}</span>
</button>

<style>
  .tile {
    height: 76px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: flex-start;
    padding: 12px 12px 10px;
    border-radius: var(--r-md);
    border: 1px solid var(--stroke);
    background: var(--bg-card);
    color: var(--text-2);
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--t-base) ease,
      border-color var(--t-base) ease,
      color var(--t-base) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .tile:hover {
    background: var(--bg-card-hover);
    color: var(--text);
  }
  .tile:active {
    transform: scale(0.97);
  }
  span {
    font-size: 12.5px;
    line-height: 16px;
    font-weight: 600;
    color: var(--text);
  }
  .tile.on {
    background: var(--accent);
    border-color: rgba(255, 255, 255, 0.12);
    color: var(--on-accent);
  }
  .tile.on span {
    color: var(--on-accent);
  }
  .tile.on:hover {
    background: color-mix(in srgb, var(--accent) 90%, white);
  }
</style>
