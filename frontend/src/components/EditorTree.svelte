<script lang="ts">
  // Command tree (SPEC §5.1): flat rows with indentation, keyboard like Explorer's tree,
  // inline rename, HTML5 drag & drop. Row geometry: 32 px, 16 px per level.
  import Icon from "./Icon.svelte";
  import Toggle from "./Toggle.svelte";
  import type { Row } from "../lib/tree";
  import { canDrop, cmdById, drop, ed, issues, remove, rename, setEnabled } from "../lib/editor.svelte";
  import { t } from "../lib/i18n";

  let { rows, onarm }: { rows: Row[]; onarm: (key: string) => void } = $props();

  let el: HTMLDivElement;
  let dragKey = $state<string | null>(null);
  let dropKey = $state<string | null>(null);

  const expandable = (r: Row) => r.kind !== "phrase" && (r.kind !== "command" || r.count > 0);
  const icon = (r: Row) => (r.kind === "root" ? "sparkles" : r.kind === "folder" ? "folder" : r.kind === "command" ? "terminal" : "quote");

  function focusRow(key: string) {
    ed.selected = key;
    ed.editing = null;
    requestAnimationFrame(() => el?.querySelector<HTMLElement>(`[data-key="${CSS.escape(key)}"]`)?.focus());
  }

  function toggle(r: Row, open = !("open" in r && r.open)) {
    ed.open[r.key] = open;
  }

  function onkey(e: KeyboardEvent) {
    if (ed.editing) return;
    const i = rows.findIndex((r) => r.key === ed.selected);
    const r = rows[i];
    if (!r) return;
    const go = (j: number) => rows[j] && focusRow(rows[j].key);
    switch (e.key) {
      case "ArrowDown":
        go(i + 1);
        break;
      case "ArrowUp":
        go(i - 1);
        break;
      case "Home":
        go(0);
        break;
      case "End":
        go(rows.length - 1);
        break;
      case "ArrowRight":
        if (expandable(r) && "open" in r && !r.open) toggle(r, true);
        else if (rows[i + 1]?.depth > r.depth) go(i + 1);
        break;
      case "ArrowLeft":
        if ("open" in r && r.open && expandable(r)) toggle(r, false);
        else for (let j = i - 1; j >= 0; j--) if (rows[j].depth < r.depth) return void go(j);
        break;
      case "Enter":
      case "F2":
        if (r.kind !== "root") ed.editing = r.key;
        break;
      case "Delete":
        onarm(r.key);
        break;
      default:
        return;
    }
    e.preventDefault();
  }

  function commit(key: string, input: HTMLInputElement) {
    if (ed.editing === key) rename(key, input.value);
    focusRow(ed.selected);
  }

  function autofocus(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<div class="tree" role="tree" aria-label={t("editor.tree")} tabindex="-1" bind:this={el} onkeydown={onkey}>
  {#each rows as r (r.key)}
    {@const cmd = r.kind === "command" ? cmdById(r.id) : undefined}
    {@const bad = cmd ? issues(cmd).length > 0 : false}
    <!-- keyboard lives on the tree (roving tabindex), not on each row -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="row"
      class:sel={ed.selected === r.key}
      class:off={r.kind === "command" && r.off}
      class:drop={dropKey === r.key}
      class:dragging={dragKey === r.key}
      role="treeitem"
      aria-level={r.depth + 1}
      aria-selected={ed.selected === r.key}
      aria-expanded={expandable(r) && "open" in r ? r.open : undefined}
      tabindex={ed.selected === r.key ? 0 : -1}
      data-key={r.key}
      style="--d: {r.depth}"
      draggable={r.kind !== "root" && ed.editing !== r.key}
      onclick={() => focusRow(r.key)}
      ondblclick={() => (expandable(r) && r.kind !== "command" ? toggle(r) : r.kind !== "root" && (ed.editing = r.key))}
      ondragstart={(e) => {
        dragKey = r.key;
        e.dataTransfer?.setData("text/plain", r.label);
        if (e.dataTransfer) e.dataTransfer.effectAllowed = "move";
      }}
      ondragend={() => (dragKey = dropKey = null)}
      ondragover={(e) => {
        if (dragKey && canDrop(dragKey, r.key)) {
          e.preventDefault();
          dropKey = r.key;
        }
      }}
      ondragleave={() => dropKey === r.key && (dropKey = null)}
      ondrop={(e) => {
        e.preventDefault();
        if (dragKey) drop(dragKey, r.key);
        dragKey = dropKey = null;
      }}>
      {#if expandable(r)}
        <button type="button" class="chev" class:open={"open" in r && r.open} tabindex="-1" aria-hidden="true" onclick={(e) => (e.stopPropagation(), toggle(r))}>
          <Icon name="chevron" size={14} stroke={2} />
        </button>
      {:else}
        <span class="chev"></span>
      {/if}
      <span class="ic {r.kind}"><Icon name={icon(r)} size={16} /></span>
      {#if ed.editing === r.key}
        <input
          class="edit"
          value={r.label}
          use:autofocus
          onclick={(e) => e.stopPropagation()}
          onblur={(e) => commit(r.key, e.currentTarget)}
          onkeydown={(e) => {
            e.stopPropagation();
            if (e.key === "Enter") commit(r.key, e.currentTarget);
            if (e.key === "Escape") (ed.editing = null), focusRow(r.key);
          }} />
      {:else}
        <span class="label" class:phrase={r.kind === "phrase"}>{r.label}</span>
      {/if}
      {#if r.kind === "root" || r.kind === "folder"}
        <span class="count num">{r.count}</span>
      {:else if r.kind === "command"}
        {#if bad}<span class="warn" title={cmd ? issues(cmd).join(", ") : ""}><Icon name="alert" size={14} /></span>{/if}
        <Toggle small checked={!r.off} label={t("editor.enabled")} onchange={(v) => setEnabled(r.id, v)} />
      {:else if r.kind === "phrase"}
        <button type="button" class="x" tabindex="-1" aria-label={t("editor.delete")} onclick={(e) => (e.stopPropagation(), remove(r.key))}>
          <Icon name="x" size={12} stroke={2} />
        </button>
      {/if}
    </div>
  {:else}
    <p class="empty t-caption">{t("editor.search_empty")}</p>
  {/each}
</div>

<style>
  .tree {
    padding: 6px;
    outline: none;
  }
  .row {
    position: relative;
    height: 32px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px 0 calc(4px + var(--d) * 16px);
    border-radius: var(--r-sm);
    cursor: default;
    transition: background-color var(--t-fast) ease;
  }
  .row:hover {
    background: var(--fill);
  }
  .row.sel {
    background: var(--fill-hover);
  }
  .row.sel::before {
    content: "";
    position: absolute;
    left: 0;
    top: 8px;
    width: 3px;
    height: 16px;
    border-radius: 2px;
    background: var(--accent);
  }
  .row:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .row.drop {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 1px rgba(var(--accent-rgb), 0.55);
  }
  .row.dragging {
    opacity: 0.45;
  }
  .row.off .label,
  .row.off .ic {
    opacity: 0.45;
  }
  .chev {
    width: 16px;
    height: 16px;
    flex: none;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--r-xs);
    background: none;
    color: var(--text-3);
    cursor: pointer;
    transition:
      rotate var(--t-base) var(--spring),
      color var(--t-fast) ease;
  }
  .chev:hover {
    color: var(--text);
  }
  .chev.open {
    rotate: 90deg;
  }
  .ic {
    display: grid;
    flex: none;
    color: var(--text-2);
  }
  .ic.root {
    color: var(--accent-text);
  }
  .ic.folder {
    color: #e8b558;
  }
  .ic.phrase {
    color: var(--text-3);
  }
  .label {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: 13px;
    font-weight: 500;
  }
  .label.phrase {
    color: var(--text-2);
    font-weight: 450;
  }
  .row.sel .label {
    color: var(--text);
  }
  .edit {
    flex: 1;
    min-width: 0;
    height: 24px;
    padding: 0 6px;
    border: 1px solid var(--accent);
    border-radius: var(--r-xs);
    background: rgba(0, 0, 0, 0.3);
    font-size: 13px;
    user-select: text;
    outline: none;
  }
  .count {
    font-size: 12px;
    color: var(--text-3);
  }
  .warn {
    display: grid;
    color: var(--warn);
  }
  .x {
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--r-xs);
    background: none;
    color: var(--text-3);
    opacity: 0;
    cursor: pointer;
    transition:
      opacity var(--t-fast) ease,
      background-color var(--t-fast) ease;
  }
  .row:hover .x,
  .row.sel .x {
    opacity: 1;
  }
  .x:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  .empty {
    padding: 16px 12px;
    margin: 0;
  }
</style>
