<script lang="ts">
  // Grouped list (Windows Settings / iOS): one card, rows split by hairlines.
  import type { Snippet } from "svelte";
  let { title, children }: { title?: string; children: Snippet } = $props();
</script>

<section class="group">
  {#if title}<h2 class="t-group">{title}</h2>{/if}
  <div class="card list">{@render children()}</div>
</section>

<style>
  .group {
    margin-bottom: 24px;
  }
  .list {
    overflow: hidden;
  }
  .list > :global(* + *) {
    border-top: 1px solid var(--divider);
  }
</style>
