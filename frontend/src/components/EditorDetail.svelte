<script lang="ts">
  // Right pane of the editor: what the selected tree node is. Command card editing grows
  // in CommandCard (T061/T062); here: overview, folder list and phrase text.
  import Icon from "./Icon.svelte";
  import CommandCard from "./CommandCard.svelte";
  import { allFolders, ckey, fkey, startsWith } from "../lib/tree";
  import { ed, rename, resolve } from "../lib/editor.svelte";
  import { t } from "../lib/i18n";

  const sel = $derived(resolve(ed.selected));

  const stats = $derived.by(() => {
    const own = ed.cmds.filter((c) => !ed.meta[c.id]?.builtin).length;
    const modified = ed.cmds.filter((c) => ed.meta[c.id]?.modified).length;
    const off = ed.cmds.filter((c) => !c.enabled).length;
    return [
      [t("editor.stat.own"), own],
      [t("editor.stat.modified"), modified],
      [t("editor.stat.off"), off],
      [t("editor.stat.folders"), allFolders(ed.cmds, ed.folders).length],
    ] as const;
  });

  const KB: [string, string][] = [
    ["Ctrl N", "editor.kb.new"],
    ["Ctrl Shift N", "editor.kb.folder"],
    ["F2", "editor.kb.rename"],
    ["Ctrl D", "editor.kb.dup"],
    ["Del", "editor.kb.delete"],
    ["Ctrl F", "editor.kb.search"],
    ["Ctrl S", "editor.kb.save"],
  ];
</script>

{#key ed.selected}
  <div class="detail">
    {#if sel.kind === "root"}
      <header>
        <span class="badge-ic root"><Icon name="sparkles" size={20} /></span>
        <div>
          <h2 class="t-title">{t("editor.root")}</h2>
          <p class="t-caption">{t("editor.root_stats")}: <span class="num">{ed.cmds.length}</span></p>
        </div>
      </header>
      <div class="stats">
        {#each stats as [label, n] (label)}
          <div class="stat card"><span class="n num">{n}</span><span class="t-caption">{label}</span></div>
        {/each}
      </div>
      <h3 class="t-group">{t("editor.shortcuts")}</h3>
      <div class="card list">
        {#each KB as [keys, label] (keys)}
          <div class="kb">
            <span>{t(label)}</span>
            <span class="keys">{#each keys.split(" ") as k (k)}<kbd>{k}</kbd>{/each}</span>
          </div>
        {/each}
        <div class="kb hint"><Icon name="move" size={14} /> {t("editor.kb.drag")}</div>
      </div>
    {:else if sel.kind === "folder"}
      {@const inside = ed.cmds.filter((c) => c.folder.join("/") === sel.path.join("/")).sort((a, b) => a.name.localeCompare(b.name, "ru"))}
      {@const total = ed.cmds.filter((c) => startsWith(c.folder, sel.path)).length}
      <header>
        <span class="badge-ic folder"><Icon name="folder" size={20} /></span>
        <div>
          <h2 class="t-title">{sel.path.at(-1)}</h2>
          <p class="t-caption"><span class="num">{total}</span> · {[t("editor.root"), ...sel.path.slice(0, -1)].join(" › ")}</p>
        </div>
      </header>
      {#if inside.length}
        <h3 class="t-group">{t("editor.folder_cmds")}</h3>
        <div class="card list">
          {#each inside as c (c.id)}
            <button type="button" class="item" class:off={!c.enabled} onclick={() => ((ed.open[fkey(sel.path)] = true), (ed.selected = ckey(c.id)))}>
              <Icon name="terminal" size={16} />
              <span class="grow">{c.name}</span>
              <span class="t-caption">{c.phrases.slice(0, 3).join(", ")}</span>
              <Icon name="chevron" size={14} />
            </button>
          {/each}
        </div>
      {:else}
        <p class="t-caption">{t("editor.folder_empty")}</p>
      {/if}
    {:else if sel.kind === "command"}
      <CommandCard cmd={sel.cmd} />
    {:else}
      {@const c = sel.cmd}
      {@const i = sel.index}
      <header>
        <span class="badge-ic"><Icon name="quote" size={20} /></span>
        <div>
          <h2 class="t-title">«{c.phrases[i]}»</h2>
          <p class="t-caption">{t("editor.phrase_of").replace("{n}", c.name)}</p>
        </div>
      </header>
      <label class="field">
        <span class="t-group">{t("editor.phrase_text")}</span>
        <input value={c.phrases[i]} onchange={(e) => rename(ed.selected, e.currentTarget.value)} />
      </label>
      <p class="t-caption">{t("editor.phrase_hint")}</p>
    {/if}
  </div>
{/key}

<style>
  .detail {
    max-width: 720px;
    transition:
      opacity var(--t-base) var(--ease-out),
      translate var(--t-base) var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 4px;
    }
  }
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 16px;
  }
  header p {
    margin: 2px 0 0;
  }
  .badge-ic {
    width: 40px;
    height: 40px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    background: var(--fill);
    border: 1px solid var(--stroke);
    color: var(--text-2);
  }
  .badge-ic.root {
    color: var(--accent-text);
    background: var(--accent-soft);
  }
  .badge-ic.folder {
    color: #e8b558;
    background: rgba(232, 181, 88, 0.12);
  }
  .grow {
    flex: 1;
    min-width: 0;
  }
  h3.t-group {
    margin-top: 20px;
  }
  .stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 12px 14px;
  }
  .stat .n {
    font-size: 22px;
    line-height: 28px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }
  .list {
    overflow: hidden;
  }
  .list > :global(* + *) {
    border-top: 1px solid var(--divider);
  }
  .kb {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 40px;
    padding: 0 14px;
  }
  .kb.hint {
    justify-content: flex-start;
    gap: 8px;
    color: var(--text-2);
    font-size: 12.5px;
  }
  .keys {
    display: flex;
    gap: 4px;
  }
  kbd {
    min-width: 22px;
    height: 22px;
    box-sizing: border-box;
    padding: 0 6px;
    display: inline-grid;
    place-items: center;
    border-radius: var(--r-xs);
    border: 1px solid var(--stroke-strong);
    border-bottom-width: 2px;
    background: var(--fill);
    font: 600 11.5px/1 var(--font);
    color: var(--text-2);
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 14px;
    border: 0;
    background: none;
    color: var(--text);
    text-align: left;
    cursor: pointer;
    transition: background-color var(--t-fast) ease;
  }
  .item:hover {
    background: var(--fill);
  }
  .item > :global(svg) {
    color: var(--text-3);
    flex: none;
  }
  .item .t-caption {
    max-width: 45%;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .item .grow {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .item.off .grow {
    opacity: 0.45;
  }
  .field {
    display: flex;
    flex-direction: column;
  }
  .field input {
    height: 36px;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    font-size: 14px;
    user-select: text;
    outline: none;
  }
  .field input:focus {
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
</style>
