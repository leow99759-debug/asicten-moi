<script lang="ts">
  // Fluent search box: icon, clear button, optional shortcut hint.
  import Icon from "./Icon.svelte";
  let {
    value = $bindable(""),
    el = $bindable<HTMLInputElement | undefined>(),
    placeholder,
    kbd = "",
  }: { value?: string; el?: HTMLInputElement; placeholder: string; kbd?: string } = $props();
</script>

<label class="search">
  <Icon name="search" size={16} />
  <input bind:this={el} bind:value {placeholder} aria-label={placeholder} onkeydown={(e) => e.key === "Escape" && (value = "")} />
  {#if value}
    <button type="button" class="clear" aria-label="×" onclick={() => (value = "")}><Icon name="x" size={12} stroke={2} /></button>
  {:else if kbd}
    <kbd>{kbd}</kbd>
  {/if}
</label>

<style>
  .search {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 0 1 260px;
    min-width: 170px;
    height: 32px;
    box-sizing: border-box;
    padding: 0 8px 0 10px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    color: var(--text-3);
    transition: background-color var(--t-fast) ease;
  }
  .search:hover {
    background: var(--fill-hover);
  }
  .search:focus-within {
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: none;
    outline: none;
    font-size: 13px;
    color: var(--text);
    user-select: text;
  }
  input::placeholder {
    color: var(--text-3);
  }
  kbd {
    padding: 1px 5px;
    border-radius: var(--r-xs);
    border: 1px solid var(--stroke);
    font: 600 10.5px/14px var(--font);
    color: var(--text-3);
    white-space: nowrap;
  }
  .clear {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: 10px;
    background: var(--fill-hover);
    color: var(--text-2);
    cursor: pointer;
  }
  .clear:hover {
    color: var(--text);
  }
</style>
