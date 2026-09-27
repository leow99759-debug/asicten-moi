<script lang="ts">
  // Fluent SelectorBar: text tabs, a short accent indicator slides to the selected one.
  let {
    tabs,
    value,
    onchange,
  }: { tabs: { id: string; label: string; disabled?: boolean }[]; value: string; onchange: (id: string) => void } =
    $props();
  let el: HTMLDivElement;
  let x = $state(-100);
  let ready = $state(false);
  $effect(() => {
    void value;
    const b = el?.querySelector<HTMLElement>(`[data-id="${value}"]`);
    if (b) x = b.offsetLeft + b.offsetWidth / 2 - 8;
    requestAnimationFrame(() => (ready = true));
  });
</script>

<div class="tabs" role="tablist" bind:this={el}>
  {#each tabs as tab (tab.id)}
    <button
      type="button"
      role="tab"
      data-id={tab.id}
      aria-selected={value === tab.id}
      disabled={tab.disabled}
      class:on={value === tab.id}
      onclick={() => onchange(tab.id)}>{tab.label}</button>
  {/each}
  <span class="ind" class:ready style="transform: translateX({x}px)"></span>
</div>

<style>
  .tabs {
    position: relative;
    display: flex;
    gap: 4px;
    margin: 0 0 20px -10px;
  }
  button {
    position: relative;
    height: 36px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-2);
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition:
      color var(--t-fast) ease,
      background-color var(--t-fast) ease;
  }
  button:hover {
    color: var(--text);
    background: var(--fill);
  }
  button.on {
    color: var(--text);
  }
  .ind {
    position: absolute;
    left: 0;
    bottom: -3px;
    width: 16px;
    height: 3px;
    border-radius: 2px;
    background: var(--accent);
  }
  .ind.ready {
    transition: transform var(--t-slow) var(--spring);
  }
</style>
