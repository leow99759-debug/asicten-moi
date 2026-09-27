<script lang="ts">
  // Command editor (SPEC §5.1): toolbar, tree | detail, breadcrumbs. State lives in
  // lib/editor.svelte.ts so unsaved edits survive switching pages.
  import { onMount } from "svelte";
  import Icon from "../components/Icon.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import EditorTree from "../components/EditorTree.svelte";
  import EditorDetail from "../components/EditorDetail.svelte";
  import { allFolders, fkey, ckey, rows as buildRows } from "../lib/tree";
  import * as E from "../lib/editor.svelte";
  import { ed } from "../lib/editor.svelte";
  import { t } from "../lib/i18n";

  let search: HTMLInputElement;
  let armed = $state<string | null>(null);
  let armTimer: ReturnType<typeof setTimeout>;

  const rows = $derived(buildRows(ed.cmds, ed.folders, ed.open, ed.query, t("editor.root")));
  const sel = $derived(E.resolve(ed.selected));
  const folderCount = $derived(allFolders(ed.cmds, ed.folders).length);
  const canPhrase = $derived(sel.kind === "command" || sel.kind === "phrase");

  onMount(() => {
    if (!ed.dirty) E.load();
  });

  /** Delete is two-step for folders/commands: first press arms, second deletes. */
  function arm(key = ed.selected) {
    if (!E.canDelete(key)) return;
    if (E.resolve(key).kind === "phrase" || armed === key) {
      armed = null;
      E.remove(key);
      return;
    }
    armed = key;
    clearTimeout(armTimer);
    armTimer = setTimeout(() => (armed = null), 3000);
  }
  $effect(() => {
    if (armed && armed !== ed.selected) armed = null;
  });
  $effect(() => {
    if (sel.kind === "command" || sel.kind === "phrase") E.openTab(sel.cmd.id);
  });
  const activeTab = $derived(sel.kind === "command" || sel.kind === "phrase" ? sel.cmd.id : null);
  let tabsEl: HTMLDivElement;
  $effect(() => {
    void activeTab;
    requestAnimationFrame(() => tabsEl?.querySelector(".tab.on")?.scrollIntoView({ inline: "nearest", block: "nearest" }));
  });

  function onkey(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey;
    if (!mod) return;
    const k = e.key.toLowerCase();
    const act: Record<string, () => void> = {
      s: E.save,
      f: () => search?.focus(),
      n: e.shiftKey ? E.addFolder : E.addCommand,
      d: E.duplicate,
    };
    // Cyrillic layout: same physical keys
    const ru: Record<string, string> = { ы: "s", а: "f", т: "n", в: "d" };
    const fn = act[ru[k] ?? k];
    if (fn) {
      e.preventDefault();
      fn();
    }
  }

  const crumbs = $derived.by(() => {
    const out: { label: string; key: string }[] = [{ label: t("editor.root"), key: "r" }];
    sel.path.forEach((name, i) => out.push({ label: name, key: fkey(sel.path.slice(0, i + 1)) }));
    if (sel.kind === "command" || sel.kind === "phrase") out.push({ label: sel.cmd.name, key: ckey(sel.cmd.id) });
    if (sel.kind === "phrase") out.push({ label: t("editor.phrases"), key: ed.selected });
    return out;
  });
</script>

<svelte:window onkeydown={onkey} />

<div class="editor">
  <PageHeader
    title={t("title.editor")}
    subtitle={t("editor.sub").replace("{c}", String(ed.cmds.length)).replace("{f}", String(folderCount)) +
      (ed.dirty ? ` · ${t("editor.unsaved")}` : "")}>
    {#snippet actions()}
      <button type="button" class="btn subtle" disabled={!ed.dirty || ed.saving} onclick={() => E.load()}>
        <Icon name="undo" size={16} />
        {t("editor.revert")}
      </button>
      <button type="button" class="btn primary" disabled={!ed.dirty || ed.saving} onclick={E.save}>
        <Icon name="check" size={16} stroke={2} />
        {ed.saving ? t("editor.saving") : t("editor.save")}
      </button>
    {/snippet}
  </PageHeader>

  {#if ed.error}
    <p class="error"><Icon name="alert" size={16} /> {t("editor.save_error").replace("{e}", ed.error)}</p>
  {/if}

  <div class="toolbar">
    <div class="tools">
      <button type="button" class="btn subtle" onclick={E.addFolder} title="Ctrl+Shift+N">
        <Icon name="folderPlus" size={16} />
        {t("editor.add_folder")}
      </button>
      <button type="button" class="btn subtle" onclick={E.addCommand} title="Ctrl+N">
        <Icon name="plus" size={16} />
        {t("editor.add_command")}
      </button>
      <button type="button" class="btn subtle" disabled={!canPhrase} onclick={E.addPhrase}>
        <Icon name="quote" size={16} />
        {t("editor.add_phrase")}
      </button>
      <span class="sep"></span>
      <button type="button" class="btn subtle" disabled={!E.canDuplicate(ed.selected)} onclick={E.duplicate} title="Ctrl+D">
        <Icon name="copy" size={16} />
        {t("editor.duplicate")}
      </button>
      <button
        type="button"
        class="btn subtle"
        class:danger={armed === ed.selected}
        disabled={!E.canDelete(ed.selected)}
        title={sel.kind === "command" && ed.meta[sel.cmd.id]?.builtin ? t("editor.delete_builtin") : "Del"}
        onclick={() => arm()}>
        <Icon name="trash" size={16} />
        {armed === ed.selected ? t("editor.delete_confirm") : t("editor.delete")}
      </button>
    </div>
    <label class="search">
      <Icon name="search" size={16} />
      <input bind:this={search} bind:value={ed.query} placeholder={t("editor.search")} aria-label={t("editor.search")}
        onkeydown={(e) => e.key === "Escape" && (ed.query = "")} />
      {#if ed.query}
        <button type="button" class="clear" aria-label="×" onclick={() => (ed.query = "")}><Icon name="x" size={12} stroke={2} /></button>
      {:else}
        <kbd>Ctrl F</kbd>
      {/if}
    </label>
  </div>

  <div class="split card">
    <div class="pane tree-pane">
      <EditorTree {rows} onarm={arm} />
    </div>
    <div class="pane detail-pane">
      <div class="tabs" role="tablist" bind:this={tabsEl}>
        <button type="button" role="tab" class="tab" aria-selected={!activeTab} class:on={!activeTab} onclick={() => (ed.selected = sel.path.length ? fkey(sel.path) : "r")}>
          <Icon name="list" size={14} />
          <span>{t("editor.tab_all")}</span>
        </button>
        {#each ed.tabs as id (id)}
          {@const c = E.cmdById(id)}
          {#if c}
            <div class="tab" role="tab" tabindex="0" aria-selected={activeTab === id} class:on={activeTab === id}
              onclick={() => (ed.selected = ckey(id))} onkeydown={(e) => e.key === "Enter" && (ed.selected = ckey(id))}>
              <Icon name="terminal" size={14} />
              <span>{c.name}</span>
              <button type="button" class="tx" aria-label={t("editor.tab_close")} onclick={(e) => (e.stopPropagation(), E.closeTab(id))}><Icon name="x" size={11} stroke={2} /></button>
            </div>
          {/if}
        {/each}
      </div>
      <div class="detail-body"><EditorDetail /></div>
    </div>
    <nav class="crumbs" aria-label={t("editor.crumbs")}>
      {#each crumbs as c, i (c.key + i)}
        {#if i}<Icon name="chevron" size={12} />{/if}
        <button type="button" class:last={i === crumbs.length - 1} onclick={() => (ed.selected = c.key)}>{c.label}</button>
      {/each}
    </nav>
  </div>
</div>

<style>
  .editor {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 480px;
  }
  .error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: -8px 0 16px;
    color: var(--err);
    font-size: 13px;
  }
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 2px;
    min-width: 0;
  }
  .tools .btn {
    padding: 0 10px;
    gap: 6px;
    color: var(--text);
  }
  .tools .btn :global(svg) {
    color: var(--text-2);
  }
  .tools .btn.danger {
    background: rgba(240, 97, 109, 0.14);
    color: var(--err);
  }
  .tools .btn.danger :global(svg) {
    color: var(--err);
  }
  .sep {
    width: 1px;
    height: 20px;
    margin: 0 6px;
    background: var(--stroke-strong);
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    width: 260px;
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
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: none;
    outline: none;
    font-size: 13px;
    color: var(--text);
    user-select: text;
  }
  .search input::placeholder {
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
  .split {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: clamp(248px, 26vw, 320px) 1fr;
    grid-template-rows: 1fr auto;
    overflow: hidden;
  }
  .pane {
    min-height: 0;
    overflow-y: auto;
  }
  .tree-pane {
    border-right: 1px solid var(--divider);
    background: rgba(0, 0, 0, 0.08);
  }
  .detail-body {
    padding: 16px 24px 24px;
    container-type: inline-size;
  }
  .tabs {
    position: sticky;
    top: 0;
    z-index: 5;
    display: flex;
    gap: 2px;
    height: 40px;
    align-items: flex-end;
    padding: 0 12px;
    border-bottom: 1px solid var(--divider);
    background: var(--bg-card);
    overflow-x: auto;
    scrollbar-width: none;
  }
  .tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: 0 1 auto;
    min-width: 72px;
    max-width: 190px;
    height: 32px;
    padding: 0 8px 0 10px;
    border: 0;
    border-radius: var(--r-sm) var(--r-sm) 0 0;
    background: none;
    color: var(--text-2);
    font-size: 12.5px;
    font-weight: 550;
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .tab span {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .tab:hover {
    background: var(--fill);
    color: var(--text);
  }
  .tab.on {
    color: var(--text);
    background: var(--fill);
  }
  .tab.on::after {
    content: "";
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: 0;
    height: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .tx {
    width: 18px;
    height: 18px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--r-xs);
    background: none;
    color: var(--text-3);
    opacity: 0;
    cursor: pointer;
  }
  .tab:hover .tx,
  .tab.on .tx {
    opacity: 1;
  }
  .tx:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  .crumbs {
    grid-column: 1 / -1;
    display: flex;
    align-items: center;
    gap: 2px;
    height: 34px;
    padding: 0 8px;
    border-top: 1px solid var(--divider);
    color: var(--text-3);
    overflow: hidden;
  }
  .crumbs button {
    height: 24px;
    padding: 0 6px;
    border: 0;
    border-radius: var(--r-xs);
    background: none;
    color: var(--text-2);
    font-size: 12px;
    font-weight: 500;
    white-space: nowrap;
    cursor: pointer;
    transition: background-color var(--t-fast) ease;
  }
  .crumbs button:hover {
    background: var(--fill);
    color: var(--text);
  }
  .crumbs button.last {
    color: var(--text);
    font-weight: 600;
  }
</style>
