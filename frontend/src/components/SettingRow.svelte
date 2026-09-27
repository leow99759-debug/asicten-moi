<script lang="ts">
  // One row of a Group: optional icon, title + description, control on the right.
  import type { Snippet } from "svelte";
  import Icon, { type IconName } from "./Icon.svelte";
  let {
    label,
    desc,
    icon,
    children,
    wide = false,
  }: { label: string; desc?: string; icon?: IconName; children: Snippet; wide?: boolean } = $props();
</script>

<div class="row" class:wide>
  {#if icon}<span class="ic"><Icon name={icon} size={18} /></span>{/if}
  <div class="lbl">
    <span class="l">{label}</span>
    {#if desc}<span class="d">{desc}</span>{/if}
  </div>
  <div class="ctl">{@render children()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    min-height: 48px;
    padding: 8px 16px;
    box-sizing: border-box;
  }
  .ic {
    width: 20px;
    flex: none;
    color: var(--text-2);
    display: grid;
    place-items: center;
  }
  .lbl {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .l {
    font-size: 13.5px;
    line-height: 20px;
    font-weight: 500;
  }
  .d {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
  }
  .ctl {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    flex: none;
    min-width: 0;
  }
  .wide .ctl {
    flex: 0 1 320px;
  }
  .wide .ctl > :global(*) {
    flex: 1;
  }
</style>
