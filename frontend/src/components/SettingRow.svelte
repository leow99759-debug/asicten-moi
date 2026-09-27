<script lang="ts">
  import type { Snippet } from "svelte";
  let { label, desc, children, wide = false }: { label: string; desc?: string; children: Snippet; wide?: boolean } =
    $props();
</script>

<div class="row" class:wide>
  <div class="lbl">
    <span class="l">{label}</span>
    {#if desc}<span class="d">{desc}</span>{/if}
  </div>
  <div class="ctl">{@render children()}</div>
</div>

<style>
  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    align-items: center;
    gap: 24px;
    min-height: 52px;
    padding: 8px 16px;
    border-radius: var(--r-md);
    background: rgba(255, 255, 255, 0.025);
    border: 1px solid var(--line);
  }
  .row.wide {
    grid-template-columns: minmax(160px, 0.8fr) minmax(0, 1.2fr);
  }
  .row.wide .ctl {
    justify-content: stretch;
  }
  .row.wide .ctl > :global(*) {
    flex: 1;
  }
  .lbl {
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
  }
  .l {
    font-size: 13.5px;
  }
  .d {
    font-size: 12px;
    color: var(--text-3);
  }
  .ctl {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    min-width: 0;
  }
</style>
