<script lang="ts">
  // Text box of chips (SPEC §5.3 phrases): Enter or «,» adds, × / Backspace removes,
  // double-click puts a chip back into the box for editing, paste splits on commas/lines.
  import Icon from "./Icon.svelte";
  import { norm } from "../lib/phrases";

  let {
    values,
    label,
    placeholder = "",
    big = false,
    inserts = [],
    onchange,
  }: {
    values: string[];
    label: string;
    placeholder?: string;
    big?: boolean;
    /** Tokens offered as one-click inserts into the box, e.g. slots `{число}`. */
    inserts?: string[];
    onchange: (next: string[]) => void;
  } = $props();

  let text = $state("");
  let input: HTMLInputElement;

  function add(raw: string) {
    const items = raw
      .split(/[,\n]/)
      .map((s) => s.trim().replace(/\s+/g, " "))
      .filter(Boolean);
    const next = [...values];
    for (const s of items) if (!next.some((v) => norm(v) === norm(s))) next.push(s);
    if (next.length !== values.length) onchange(next);
    text = "";
  }
  const remove = (i: number) => onchange(values.filter((_, j) => j !== i));

  function key(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      add(text);
    } else if (e.key === "Backspace" && !text && values.length) {
      remove(values.length - 1);
    } else if (e.key === "Escape") {
      text = "";
    }
  }
  function editChip(i: number) {
    if (text.trim()) add(text);
    text = values[i];
    remove(i);
    input.focus();
  }
  function insert(tok: string) {
    text = `${text.trimEnd()}${text.trim() ? " " : ""}${tok}`;
    input.focus();
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="box" class:big onclick={(e) => e.target === e.currentTarget && input.focus()}>
  {#each values as v, i (v)}
    <span class="chip" ondblclick={() => editChip(i)} title={v}>
      <span class="txt">{v}</span>
      <button type="button" class="x" aria-label="×" tabindex="-1" onclick={() => remove(i)}><Icon name="x" size={12} stroke={2} /></button>
    </span>
  {/each}
  <input
    bind:this={input}
    bind:value={text}
    aria-label={label}
    placeholder={values.length ? "" : placeholder}
    onkeydown={key}
    onblur={() => text.trim() && add(text)}
    onpaste={(e) => {
      const s = e.clipboardData?.getData("text") ?? "";
      if (/[,\n]/.test(s)) {
        e.preventDefault();
        add(text + s);
      }
    }} />
</div>
{#if inserts.length}
  <div class="inserts">
    {#each inserts as tok (tok)}
      <button type="button" class="ins" onmousedown={(e) => e.preventDefault()} onclick={() => insert(tok)}>{tok}</button>
    {/each}
  </div>
{/if}

<style>
  .box {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    min-height: 36px;
    box-sizing: border-box;
    padding: 4px 6px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    cursor: text;
    transition:
      background-color var(--t-fast) ease,
      box-shadow var(--t-fast) ease;
  }
  .box:focus-within {
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    max-width: 100%;
    height: 24px;
    padding: 0 2px 0 8px;
    border-radius: var(--r-xs);
    background: rgba(255, 255, 255, 0.07);
    border: 1px solid var(--stroke);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
    cursor: default;
    transition:
      opacity var(--t-fast) var(--ease-out),
      scale var(--t-fast) var(--ease-out);
    @starting-style {
      opacity: 0;
      scale: 0.9;
    }
  }
  .big .chip {
    height: 28px;
    padding-left: 10px;
    font-size: 13px;
    color: var(--text);
  }
  .txt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .x {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 3px;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .x:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  input {
    flex: 1 1 120px;
    min-width: 80px;
    height: 26px;
    padding: 0 6px;
    border: 0;
    background: none;
    color: var(--text);
    font-size: 13px;
    user-select: text;
    outline: none;
  }
  input::placeholder {
    color: var(--text-3);
  }
  .inserts {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 6px;
  }
  .ins {
    height: 22px;
    padding: 0 8px;
    border-radius: 11px;
    border: 1px dashed var(--stroke-strong);
    background: none;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-3);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .ins:hover {
    background: var(--accent-soft);
    border-color: transparent;
    color: var(--accent-text);
  }
</style>
