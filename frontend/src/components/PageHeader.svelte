<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, subtitle, actions }: { title: string; subtitle?: string; actions?: Snippet } = $props();
</script>

<header class="ph">
  <div class="tx">
    <h1 class="t-display">{title}</h1>
    {#if subtitle}<p>{subtitle}</p>{/if}
  </div>
  {#if actions}<div class="act">{@render actions()}</div>{/if}
</header>

<style>
  .ph {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 24px;
  }
  .tx {
    min-width: 0;
  }
  p {
    margin: 4px 0 0;
    color: var(--text-2);
    font-size: 13.5px;
  }
  .act {
    display: flex;
    gap: 8px;
    flex: none;
  }
</style>
