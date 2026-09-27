<script lang="ts">
  // Segmented control with a sliding pill (spatial continuity between tabs).
  let {
    tabs,
    value,
    onchange,
  }: { tabs: { id: string; label: string; disabled?: boolean }[]; value: string; onchange: (id: string) => void } =
    $props();
  let el: HTMLDivElement;
  let pill = $state({ x: 0, w: 0 });
  $effect(() => {
    void value;
    const b = el?.querySelector<HTMLElement>(`[data-id="${value}"]`);
    if (b) pill = { x: b.offsetLeft, w: b.offsetWidth };
  });
</script>

<div class="tabs" role="tablist" bind:this={el}>
  <span class="pill" style="transform: translateX({pill.x}px); width: {pill.w}px"></span>
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
</div>

<style>
  .tabs {
    position: relative;
    display: inline-flex;
    padding: 3px;
    gap: 2px;
    border-radius: var(--r-md);
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid var(--line);
  }
  .pill {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 0;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.09);
    box-shadow: inset 0 1px 0 var(--highlight), var(--shadow-sm);
    transition: transform var(--t-med) var(--ease-out), width var(--t-med) var(--ease-out);
  }
  button {
    position: relative;
    height: 30px;
    padding: 0 14px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--text-2);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: color var(--t-fast) ease;
  }
  button:hover:not(:disabled) {
    color: var(--text);
  }
  button.on {
    color: var(--text);
  }
  button:disabled {
    color: var(--text-3);
    cursor: default;
  }
</style>
