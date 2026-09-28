<script lang="ts">
  // video26/30 segmented tabs: the selected one is a blue pill that slides between tabs.
  let {
    tabs,
    value,
    onchange,
  }: { tabs: { id: string; label: string; disabled?: boolean }[]; value: string; onchange: (id: string) => void } =
    $props();
  let el: HTMLDivElement;
  let x = $state(-100);
  let w = $state(0);
  let ready = $state(false);
  $effect(() => {
    void value;
    const b = el?.querySelector<HTMLElement>(`[data-id="${value}"]`);
    if (b) {
      x = b.offsetLeft;
      w = b.offsetWidth;
    }
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
  <span class="ind" class:ready style="transform: translateX({x}px); width: {w}px"></span>
</div>

<style>
  .tabs {
    position: relative;
    display: inline-flex;
    gap: 4px;
    margin: 0 0 18px;
    padding: 3px;
    border: 1px solid var(--stroke);
    border-radius: 11px;
    background: rgba(0, 0, 0, 0.18);
    isolation: isolate;
  }
  button {
    position: relative;
    z-index: 1;
    height: 32px;
    min-width: 96px;
    padding: 0 16px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2);
    font-size: 13px;
    font-weight: 550;
    cursor: pointer;
    transition: color var(--t-base) ease;
  }
  button:hover {
    color: var(--text);
  }
  button.on {
    color: #fff;
  }
  .ind {
    position: absolute;
    z-index: 0;
    left: 0;
    top: 3px;
    height: 32px;
    border-radius: 8px;
    background: var(--accent);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.18),
      0 4px 14px rgba(var(--accent-rgb), 0.3);
  }
  .ind.ready {
    transition:
      transform var(--t-slow) var(--spring),
      width var(--t-slow) var(--spring);
  }
</style>
