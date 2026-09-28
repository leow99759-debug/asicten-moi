<script lang="ts">
  // video26 t43: one glass card with the title inside; each row is its own rounded box.
  import type { Snippet } from "svelte";
  let { title, children }: { title?: string; children: Snippet } = $props();
</script>

<section class="card group">
  {#if title}<h2 class="gt">{title}</h2>{/if}
  <div class="list">{@render children()}</div>
</section>

<style>
  .group {
    margin-bottom: 16px;
    padding: 14px;
  }
  .gt {
    margin: 0 0 12px 2px;
    font-size: 15px;
    line-height: 20px;
    font-weight: 650;
    letter-spacing: -0.008em;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .list > :global(*) {
    border: 1px solid var(--stroke);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.025);
  }
</style>
