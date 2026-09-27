<script lang="ts">
  // Command card (SPEC §5.3, video 26 t15/t18): name, Связывать / Подтверждать, action steps
  // with drag-to-reorder, quick type chips, phrase chips + live match preview, reply, «▶ Тест».
  import ActionRow from "./ActionRow.svelte";
  import Checkbox from "./Checkbox.svelte";
  import ChipInput from "./ChipInput.svelte";
  import MatchPreview from "./MatchPreview.svelte";
  import ReplyPicker from "./ReplyPicker.svelte";
  import Icon from "./Icon.svelte";
  import Toggle from "./Toggle.svelte";
  import type { Action } from "../lib/bindings/Action";
  import type { Command } from "../lib/bindings/Command";
  import type { CommandOutcome } from "../lib/bindings/CommandOutcome";
  import { DEFAULTS, GROUPS, QUICK, type ActionType } from "../lib/actions";
  import { ed, issues, runTest, setEnabled, touch } from "../lib/editor.svelte";
  import { t } from "../lib/i18n";
  import { SLOTS } from "../lib/phrases";
  import { inTauri } from "../lib/window";

  let { cmd }: { cmd: Command } = $props();

  const meta = $derived(ed.meta[cmd.id]);
  const bad = $derived(issues(cmd));
  let menu = $state(false);
  let drag = $state<number | null>(null);
  let over = $state<number | null>(null);

  function edit(fn: () => void) {
    fn();
    touch();
  }
  function add(type: ActionType) {
    edit(() => cmd.actions.push(structuredClone(DEFAULTS[type]) as Action));
    menu = false;
  }
  let testing = $state(false);
  let result = $state<{ ok: boolean; text: string } | null>(null);
  let hide: ReturnType<typeof setTimeout> | undefined;

  function verdict(o: CommandOutcome | null): { ok: boolean; text: string } {
    if (!o) return { ok: false, text: t(inTauri() ? "card.test_timeout" : "card.test_app") };
    if (o.status === "done") return { ok: true, text: t("card.test_done") };
    const err = o.steps.find((s) => s.error)?.error;
    return { ok: false, text: o.status === "error" && err ? err : t(`card.test_${o.status}`) };
  }
  async function test() {
    testing = true;
    result = null;
    clearTimeout(hide);
    try {
      result = verdict(await runTest(cmd));
    } catch (e) {
      result = { ok: false, text: String(e) };
    } finally {
      testing = false;
      hide = setTimeout(() => (result = null), 6000);
    }
  }
  // a stale result must not stick to the next opened command
  $effect(() => {
    void cmd.id;
    result = null;
  });

  function move(from: number, to: number) {
    if (from === to) return;
    edit(() => {
      const [a] = cmd.actions.splice(from, 1);
      cmd.actions.splice(to, 0, a);
    });
  }
</script>

<svelte:window onclick={() => (menu = false)} />

<header>
  <span class="badge-ic"><Icon name="terminal" size={20} /></span>
  <div class="grow">
    <h2 class="t-title">{cmd.name}</h2>
    <p class="t-caption">{[t("editor.root"), ...cmd.folder].join(" › ")}</p>
  </div>
  <Toggle checked={cmd.enabled} label={t("editor.enabled")} onchange={(v) => setEnabled(cmd.id, v)} />
  <button type="button" class="btn primary test" disabled={testing || bad.length > 0} title={t("card.test_hint")} onclick={test}>
    <Icon name="play" size={14} />
    {testing ? t("card.testing") : t("card.test")}
  </button>
</header>
{#if result}
  <div class="result" class:ok={result.ok} role="status">
    <Icon name={result.ok ? "check" : "alert"} size={14} stroke={2} />
    {result.text}
  </div>
{/if}
<div class="badges">
  {#if meta?.builtin}<span class="chip">{t("editor.badge.builtin")}</span>{/if}
  {#if meta?.modified}<span class="chip accent">{t("editor.badge.modified")}</span>{/if}
  {#if !cmd.enabled}<span class="chip">{t("editor.badge.off")}</span>{/if}
  {#each bad as b (b)}<span class="chip warn"><Icon name="alert" size={12} /> {b}</span>{/each}
</div>

<div class="name-row">
  <label class="name">
    <span class="t-group">{t("card.name")}</span>
    <input value={cmd.name} onchange={(e) => e.currentTarget.value.trim() && edit(() => (cmd.name = e.currentTarget.value.trim()))} />
  </label>
  <Checkbox checked={cmd.chainable} label={t("card.chain")} title={t("card.chain_hint")} onchange={(v) => edit(() => (cmd.chainable = v))} />
  <Checkbox checked={cmd.confirm} label={t("card.confirm")} title={t("card.confirm_hint")} onchange={(v) => edit(() => (cmd.confirm = v))} />
</div>

<div class="divider"></div>

<div class="bar">
  <div class="menu-wrap">
    <button type="button" class="btn" aria-expanded={menu} onclick={(e) => (e.stopPropagation(), (menu = !menu))}>
      <Icon name="plus" size={16} />
      {t("card.add_action")}
    </button>
    {#if menu}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="flyout" onclick={(e) => e.stopPropagation()}>
        <div class="quick">
          {#each QUICK as [label, type] (type)}
            <button type="button" class="qchip" onclick={() => add(type)}>{t(label)}</button>
          {/each}
        </div>
        <div class="groups">
          {#each GROUPS as [group, types] (group)}
            <div class="grp">
              <div class="t-group">{group}</div>
              {#each types as ty (ty)}<button type="button" class="opt" onclick={() => add(ty)}>{ty.replace(`${group}.`, "")}</button>{/each}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
  <button type="button" class="btn" disabled title={t("card.soon_record")}><Icon name="record" size={16} /> {t("card.record")}</button>
  <button type="button" class="btn" disabled title={t("card.soon_ai")}><Icon name="sparkles" size={16} /> {t("card.ai")}</button>
</div>

<div class="quick-hints">
  {#each QUICK as [label, type] (type)}
    <button type="button" class="qchip" onclick={() => add(type)}><Icon name="plus" size={12} stroke={2} /> {t(label)}</button>
  {/each}
</div>

{#if cmd.actions.length}
  <div class="card list" role="list">
    {#each cmd.actions as a, i (i)}
      <div
        role="listitem"
        class="item"
        class:dragging={drag === i}
        class:over={over === i && drag !== null && drag !== i}
        draggable="true"
        ondragstart={(e) => {
          if (!(e.target as HTMLElement).closest(".grip")) return e.preventDefault();
          drag = i;
          e.dataTransfer?.setData("text/plain", String(i));
        }}
        ondragover={(e) => {
          if (drag === null) return;
          e.preventDefault();
          over = i;
        }}
        ondrop={(e) => {
          e.preventDefault();
          if (drag !== null) move(drag, i);
          drag = over = null;
        }}
        ondragend={() => (drag = over = null)}>
        <ActionRow
          action={a}
          index={i}
          onchange={(next) => edit(() => (cmd.actions[i] = next))}
          onremove={() => edit(() => cmd.actions.splice(i, 1))}
          onpause={() => edit(() => cmd.actions.splice(i + 1, 0, { type: "Lux.PauseMS", ms: 500 }))} />
      </div>
    {/each}
  </div>
{:else}
  <div class="empty card">
    <Icon name="bolt" size={18} />
    <span>{t("card.no_actions")}</span>
  </div>
{/if}

<h3 class="t-group">{t("editor.sec.phrases")}</h3>
<ChipInput
  big
  values={cmd.phrases}
  label={t("editor.sec.phrases")}
  placeholder={t("card.phrases_ph")}
  inserts={SLOTS}
  onchange={(v) => edit(() => (cmd.phrases = v))} />
<h3 class="t-group">{t("editor.sec.optional")}</h3>
<ChipInput
  values={cmd.optional}
  label={t("editor.sec.optional")}
  placeholder={t("card.optional_ph")}
  onchange={(v) => edit(() => (cmd.optional = v))} />
<h3 class="t-group">{t("card.preview")}</h3>
<MatchPreview {cmd} />
<h3 class="t-group">{t("editor.sec.reply")}</h3>
<ReplyPicker reply={cmd.reply} onchange={(r) => edit(() => (cmd.reply = r))} />

<style>
  header {
    display: flex;
    align-items: center;
    gap: 14px;
    margin-bottom: 12px;
  }
  header p {
    margin: 2px 0 0;
  }
  .grow {
    flex: 1;
    min-width: 0;
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
  .badges,
  .quick-hints {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .badges {
    margin-bottom: 16px;
  }
  .badges:empty {
    display: none;
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 22px;
    padding: 0 8px;
    border-radius: var(--r-xs);
    background: var(--fill);
    border: 1px solid var(--stroke);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-2);
  }
  .chip.accent {
    background: var(--accent-soft);
    border-color: transparent;
    color: var(--accent-text);
  }
  .chip.warn {
    background: rgba(245, 184, 61, 0.12);
    border-color: transparent;
    color: var(--warn);
  }
  .name-row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 8px;
  }
  .name {
    flex: 1 1 260px;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .name input {
    height: 32px;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    font-size: 13.5px;
    user-select: text;
    outline: none;
  }
  .name input:focus {
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  .divider {
    height: 1px;
    margin: 16px 0;
    background: var(--divider);
  }
  .bar {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .bar .btn :global(svg) {
    color: var(--text-2);
  }
  .menu-wrap {
    position: relative;
  }
  .flyout {
    position: absolute;
    z-index: 20;
    top: 38px;
    left: 0;
    width: min(560px, 100cqw);
    max-height: 360px;
    box-sizing: border-box;
    overflow-y: auto;
    padding: 12px;
    border-radius: var(--r-lg);
    background: var(--bg-flyout);
    box-shadow: var(--shadow-flyout);
    backdrop-filter: blur(24px) saturate(140%);
    transform-origin: top left;
    transition:
      opacity var(--t-base) var(--ease-out),
      scale var(--t-base) var(--ease-out);
    @starting-style {
      opacity: 0;
      scale: 0.97;
    }
  }
  .quick {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding-bottom: 12px;
    margin-bottom: 8px;
    border-bottom: 1px solid var(--divider);
  }
  .groups {
    columns: 3 150px;
    column-gap: 12px;
  }
  .grp {
    break-inside: avoid;
    margin-bottom: 10px;
  }
  .grp .t-group {
    margin: 0 0 2px 8px;
  }
  .opt {
    display: block;
    width: 100%;
    height: 28px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--r-sm);
    background: none;
    text-align: left;
    font-size: 12.5px;
    cursor: pointer;
  }
  .opt:hover {
    background: var(--fill-hover);
  }
  .qchip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 26px;
    padding: 0 10px;
    border-radius: 13px;
    border: 1px solid var(--stroke);
    background: var(--fill);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .qchip:hover {
    background: var(--accent-soft);
    color: var(--accent-text);
  }
  .quick-hints {
    margin: 12px 0;
  }
  .list > :global(* + *) {
    border-top: 1px solid var(--divider);
  }
  .item {
    transition:
      opacity var(--t-fast) ease,
      box-shadow var(--t-fast) ease;
  }
  .item.dragging {
    opacity: 0.4;
  }
  .item.over {
    box-shadow: inset 0 2px 0 var(--accent);
  }
  .empty {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px;
    color: var(--text-3);
    font-size: 13px;
  }
  h3.t-group {
    margin-top: 20px;
  }
  .test :global(svg) {
    color: currentColor;
  }
  .result {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: -4px 0 12px;
    padding: 8px 12px;
    border-radius: var(--r-sm);
    background: rgba(245, 184, 61, 0.1);
    color: var(--warn);
    font-size: 13px;
    font-weight: 500;
    transition:
      opacity var(--t-base) var(--ease-out),
      translate var(--t-base) var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 -4px;
    }
  }
  .result.ok {
    background: rgba(52, 199, 89, 0.1);
    color: var(--ok);
  }
</style>
