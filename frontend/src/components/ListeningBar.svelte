<script lang="ts">
  // Listening pill in the title bar (SPEC §3.2): mic, live transcript with «Джарвис»
  // highlighted, level bars. Idle it reads like a search field (Teams/Discord title bar).
  import Icon from "./Icon.svelte";
  import { app } from "../lib/app.svelte";
  import { activate } from "../lib/commands";
  import { t } from "../lib/i18n";

  const WAKE = /^(\s*)(джарвис|jarvis)([,!.\s]*)/i;
  const parts = $derived.by(() => {
    const m = app.transcript.match(WAKE);
    return m
      ? { lead: m[1], wake: m[2], rest: app.transcript.slice(m[0].length - m[3].length) }
      : { lead: "", wake: "", rest: app.transcript };
  });
  const live = $derived(app.state === "listening" || app.state === "processing");
  const BARS = 7;
  const bars = $derived(
    Array.from({ length: BARS }, (_, i) => {
      const center = 1 - Math.abs(i - (BARS - 1) / 2) / ((BARS - 1) / 2);
      return 0.2 + Math.min(1, app.micLevel * 1.6) * (0.3 + center * 0.7);
    }),
  );
</script>

<div class="bar" class:live>
  <button type="button" class="mic" aria-label={t("mic.activate")} title={t("mic.activate")} onclick={activate}>
    <Icon name={app.state === "mic_off" ? "micOff" : "mic"} size={14} stroke={2} />
  </button>
  <p class="text" aria-live="polite">
    {#if app.transcript}
      {parts.lead}{#if parts.wake}<mark>{parts.wake}</mark>{/if}{parts.rest}
    {:else}
      <span class="hint">{app.state === "mic_off" ? t("listen.micOff") : t("listen.hint")}</span>
    {/if}
  </p>
  <div class="level" aria-hidden="true">
    {#each bars as h, i (i)}
      <span style="transform: scaleY({live ? h : 0.2})"></span>
    {/each}
  </div>
</div>

<style>
  .bar {
    height: 30px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 12px 0 4px;
    border-radius: var(--r-md);
    background: var(--fill);
    border: 1px solid var(--stroke);
    transition:
      border-color var(--t-base) ease,
      box-shadow var(--t-base) ease,
      background-color var(--t-base) ease;
  }
  .bar.live {
    background: rgba(var(--accent-rgb), 0.08);
    border-color: rgba(var(--accent-rgb), 0.45);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.1);
  }
  .mic {
    width: 22px;
    height: 22px;
    flex: none;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .mic:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  .mic:active {
    transform: scale(0.9);
  }
  .live .mic {
    background: var(--accent);
    color: var(--on-accent);
  }
  .text {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 13px;
    line-height: 18px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  mark {
    background: none;
    color: var(--accent-text);
    font-weight: 650;
  }
  .hint {
    color: var(--text-3);
    font-weight: 450;
  }
  .level {
    display: flex;
    align-items: center;
    gap: 2px;
    height: 14px;
    flex: none;
  }
  .level span {
    width: 2px;
    height: 100%;
    border-radius: 1px;
    background: var(--text-3);
    transition: transform 90ms linear;
  }
  .live .level span {
    background: var(--accent-text);
  }
</style>
