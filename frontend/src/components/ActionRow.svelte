<script lang="ts">
  // One step of a command (SPEC §5.3): № | type | parameters | ⏱ 📁 ⊗. Grip drags to reorder.
  import Icon from "./Icon.svelte";
  import type { Action } from "../lib/bindings/Action";
  import { appName, fieldKind, fields, GROUPS, parseValue, retype, type ActionType } from "../lib/actions";
  import { t } from "../lib/i18n";
  import { pickFile } from "../lib/window";

  let {
    action,
    index,
    onchange,
    onremove,
    onpause,
  }: { action: Action; index: number; onchange: (a: Action) => void; onremove: () => void; onpause: () => void } = $props();

  const rec = $derived(action as Record<string, unknown>);
  const keys = $derived(fields(action.type));
  const hasFile = $derived(keys.some((k) => fieldKind(action.type, k) === "file"));
  const label = (k: string) => {
    const s = t(`act.f.${k}`);
    return s.startsWith("act.f.") ? k : s;
  };

  function set(k: string, v: unknown) {
    onchange({ ...action, [k]: v } as Action);
  }

  async function browse() {
    const k = keys.find((f) => fieldKind(action.type, f) === "file");
    const ext = action.type === "Sound.PlayWav" ? ["wav"] : ["exe", "lnk", "bat", "cmd"];
    const p = await pickFile([{ name: t("act.files"), extensions: ext }]);
    if (k && p) set(k, p);
  }
</script>

<div class="row">
  <span class="grip" aria-hidden="true"><Icon name="grip" size={14} /></span>
  <span class="n num">{index + 1}</span>
  <select class="type" aria-label={t("act.type")} value={action.type} onchange={(e) => onchange(retype(action, e.currentTarget.value as ActionType))}>
    {#each GROUPS as [group, types] (group)}
      <optgroup label={group}>
        {#each types as ty (ty)}<option value={ty}>{ty}</option>{/each}
      </optgroup>
    {/each}
  </select>
  <div class="params">
    {#each keys as k (k)}
      {@const kind = fieldKind(action.type, k)}
      {@const v = rec[k]}
      {#if kind === "bool"}
        <label class="flag"><input type="checkbox" checked={!!v} onchange={(e) => set(k, e.currentTarget.checked)} /> {label(k)}</label>
      {:else if typeof kind === "object"}
        <select class="field sel" aria-label={label(k)} value={String(v)} onchange={(e) => set(k, e.currentTarget.value)}>
          {#each kind.options as o (o)}<option value={o}>{t(`act.o.${o}`).startsWith("act.o.") ? o : t(`act.o.${o}`)}</option>{/each}
        </select>
      {:else}
        <label class="field" class:num={kind === "num"} class:file={kind === "file"}>
          {#if kind === "file" && v}<span class="app" title={String(v)}><Icon name="app" size={14} /> {appName(String(v))}</span>{/if}
          <input
            value={v ?? ""}
            placeholder={label(k)}
            title={label(k)}
            inputmode={kind === "num" ? "numeric" : undefined}
            onchange={(e) => set(k, parseValue(kind, e.currentTarget.value, v === null || k === "workdir" || k === "window"))} />
        </label>
      {/if}
    {:else}
      <span class="none t-caption">{t("act.no_params")}</span>
    {/each}
  </div>
  <div class="tools">
    <button type="button" class="ib" title={t("act.pause_after")} onclick={onpause}><Icon name="clock" size={15} /></button>
    {#if hasFile}<button type="button" class="ib" title={t("act.browse")} onclick={browse}><Icon name="folder" size={15} /></button>{/if}
    <button type="button" class="ib del" title={t("editor.delete")} onclick={onremove}><Icon name="x" size={15} /></button>
  </div>
</div>

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    gap: 8px;
    min-height: 48px;
    padding: 8px 8px 8px 4px;
    box-sizing: border-box;
  }
  .grip {
    display: grid;
    height: 32px;
    place-items: center;
    width: 16px;
    color: var(--text-3);
    opacity: 0;
    cursor: grab;
    transition: opacity var(--t-fast) ease;
  }
  .row:hover .grip {
    opacity: 1;
  }
  .n {
    width: 18px;
    flex: none;
    text-align: right;
    color: var(--text-3);
    font-size: 12px;
    line-height: 32px;
  }
  select,
  input {
    height: 32px;
    box-sizing: border-box;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background-color: var(--fill);
    color: var(--text);
    font-size: 13px;
    outline: none;
    transition: background-color var(--t-fast) ease;
  }
  select:hover,
  input:hover {
    background-color: var(--fill-hover);
  }
  input:focus,
  select:focus {
    background-color: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  select {
    appearance: none;
    padding: 0 26px 0 10px;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23a4abb8' stroke-width='2.5' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='m6 9 6 6 6-6'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 9px center;
    cursor: pointer;
  }
  select option,
  select optgroup {
    background: #1e2026;
  }
  .type {
    width: 176px;
    flex: none;
    font-weight: 600;
    color: var(--accent-text);
  }
  .params {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .field {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1 1 140px;
    min-width: 0;
  }
  .field.file {
    flex: 3 1 240px;
  }
  .field.num {
    flex: 0 1 88px;
  }
  .field.sel {
    flex: 0 1 150px;
  }
  .field input {
    width: 100%;
    padding: 0 10px;
    user-select: text;
  }
  .app {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: none;
    max-width: 45%;
    height: 24px;
    margin-right: 6px;
    padding: 0 8px;
    border-radius: var(--r-xs);
    background: var(--accent-soft);
    color: var(--accent-text);
    font-size: 12px;
    font-weight: 600;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .flag {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 4px;
    font-size: 12.5px;
    color: var(--text-2);
    cursor: pointer;
  }
  .flag input {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
  }
  .none {
    padding-left: 2px;
  }
  .tools {
    display: flex;
    gap: 2px;
    flex: none;
    padding-top: 1px;
  }
  .ib {
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    padding: 0;
    border: 0;
    border-radius: var(--r-sm);
    background: none;
    color: var(--text-3);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .ib:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  .ib.del:hover {
    color: var(--err);
  }
  /* narrow card: type + tools on the first line, parameters get the full second line */
  @container (max-width: 620px) {
    .type {
      flex: 1;
    }
    .tools {
      order: 1;
    }
    .params {
      order: 2;
      flex-basis: 100%;
      padding-left: 50px;
    }
  }
</style>
