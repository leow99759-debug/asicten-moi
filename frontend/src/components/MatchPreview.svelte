<script lang="ts">
  // Live match preview (SPEC §5.3): «браузер ✓», «открой браузер ✓»… checked by the real core
  // matcher against the editor's unsaved set, plus a field to try any phrase.
  import Icon from "./Icon.svelte";
  import type { Command } from "../lib/bindings/Command";
  import type { Probe } from "../lib/bindings/Probe";
  import { editorProbe } from "../lib/commands";
  import { ed } from "../lib/editor.svelte";
  import { t } from "../lib/i18n";
  import { samples } from "../lib/phrases";

  let { cmd }: { cmd: Command } = $props();

  let own = $state("");
  let rows = $state<Probe[]>([]);
  /** Outside the app there is no matcher: samples are shown as fitting, typed phrases unknown. */
  let demo = $state(false);

  const typed = $derived(own.trim());
  const base = $derived(samples(cmd));
  const texts = $derived(typed && !base.includes(typed) ? [...base, typed] : base);

  $effect(() => {
    const list = texts;
    const target = $state.snapshot(cmd);
    const timer = setTimeout(async () => {
      const got = await editorProbe($state.snapshot(ed.cmds), target, list);
      demo = !got;
      rows = got ?? list.map((text, i) => ({ text, id: i < base.length ? target.id : null, name: null }));
    }, 180);
    return () => clearTimeout(timer);
  });

  const list = $derived(rows.slice(0, base.length));
  const custom = $derived(typed ? rows.find((r) => r.text === typed) : undefined);
</script>

{#snippet verdict(r: Probe)}
  {#if r.id === cmd.id}
    <span class="res ok"><Icon name="check" size={12} stroke={2.4} /></span>
  {:else if demo && typed && !base.includes(r.text)}
    <span class="res">{t("card.preview_app")}</span>
  {:else if r.id}
    <span class="res warn" title={t("card.preview_other")}><Icon name="alert" size={12} /> {r.name}</span>
  {:else}
    <span class="res warn">{t("card.preview_none")}</span>
  {/if}
{/snippet}

<div class="chips" aria-live="polite">
  {#each list as r (r.text)}
    <span class="probe" class:bad={r.id !== cmd.id}>{r.text} {@render verdict(r)}</span>
  {:else}
    <span class="t-caption">{t("card.preview_empty")}</span>
  {/each}
</div>
<div class="try">
  <Icon name="mic" size={14} />
  <input bind:value={own} placeholder={t("card.preview_try")} aria-label={t("card.preview_try")} />
  {#if custom}{@render verdict(custom)}{/if}
</div>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .probe {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 8px 0 10px;
    border-radius: 13px;
    background: rgba(var(--accent-rgb), 0.1);
    color: var(--text);
    font-size: 12.5px;
    font-weight: 500;
    transition: background-color var(--t-fast) ease;
  }
  .probe.bad {
    background: rgba(245, 184, 61, 0.1);
  }
  .res {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-3);
  }
  .res.ok {
    color: var(--ok);
  }
  .res.warn {
    color: var(--warn);
  }
  .try {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    margin-top: 8px;
    padding: 0 10px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    color: var(--text-3);
    transition: box-shadow var(--t-fast) ease;
  }
  .try:focus-within {
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  .try input {
    flex: 1;
    min-width: 0;
    height: 100%;
    border: 0;
    background: none;
    color: var(--text);
    font-size: 13px;
    user-select: text;
    outline: none;
  }
  .try input::placeholder {
    color: var(--text-3);
  }
</style>
