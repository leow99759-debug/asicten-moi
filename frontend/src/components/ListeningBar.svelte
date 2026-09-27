<script lang="ts">
  // Listening pill (SPEC §3.2): accent mic disc, live transcript with «Джарвис» highlighted,
  // level bars on the right. Visible while listening/processing.
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
  const BARS = 9;
  const bars = $derived(
    Array.from({ length: BARS }, (_, i) => {
      const center = 1 - Math.abs(i - (BARS - 1) / 2) / ((BARS - 1) / 2);
      return 0.18 + Math.min(1, app.micLevel * 1.6) * (0.35 + center * 0.65);
    }),
  );
</script>

<div class="bar" class:live>
  <button type="button" class="mic pressable" aria-label={t("mic.activate")} title={t("mic.activate")} onclick={activate}>
    <Icon name={app.state === "mic_off" ? "micOff" : "mic"} size={18} stroke={2} />
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
      <span style="transform: scaleY({live ? h : 0.18})"></span>
    {/each}
  </div>
</div>

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 48px;
    padding: 0 18px 0 6px;
    border-radius: 24px;
    background: var(--surface-2);
    backdrop-filter: blur(var(--blur)) saturate(140%);
    border: 1px solid var(--line);
    box-shadow: inset 0 1px 0 var(--highlight), var(--shadow-sm);
    min-width: 0;
    transition: border-color var(--t-med) ease, box-shadow var(--t-med) ease;
  }
  .bar.live {
    border-color: rgba(var(--accent-rgb), 0.35);
    box-shadow: inset 0 1px 0 var(--highlight), 0 0 0 4px rgba(var(--accent-rgb), 0.08);
  }
  .mic {
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 50%;
    border: 0;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--on-accent);
    cursor: pointer;
    box-shadow: 0 4px 14px rgba(var(--accent-rgb), 0.45);
  }
  .text {
    flex: 1;
    min-width: 0;
    margin: 0;
    font-size: 16px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  mark {
    background: rgba(var(--accent-rgb), 0.9);
    color: var(--on-accent);
    border-radius: 6px;
    padding: 1px 6px;
    margin-right: 2px;
  }
  .hint {
    color: var(--text-3);
    font-weight: 400;
  }
  .level {
    display: flex;
    align-items: center;
    gap: 3px;
    height: 22px;
    flex: none;
  }
  .level span {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: var(--text-2);
    transform-origin: center;
    transition: transform 90ms linear;
  }
  .bar.live .level span {
    background: var(--text);
  }
</style>
