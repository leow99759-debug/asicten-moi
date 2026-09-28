<script lang="ts">
  // §3.4: «фраза → статус», time, click repeats the phrase.
  import Icon, { type IconName } from "./Icon.svelte";
  import type { HistoryEntry } from "../lib/bindings/HistoryEntry";
  import type { Status } from "../lib/bindings/Status";
  import { runText } from "../lib/commands";
  import { statusLabel } from "../lib/app.svelte";
  import { t } from "../lib/i18n";

  let { items, compact = false }: { items: HistoryEntry[]; compact?: boolean } = $props();

  const STATUS: Record<Status, { icon: IconName; cls: string }> = {
    done: { icon: "check", cls: "ok" },
    error: { icon: "alert", cls: "err" },
    cancelled: { icon: "ban", cls: "muted" },
    no_internet: { icon: "wifiOff", cls: "warn" },
  };
  const UNKNOWN = { icon: "info" as IconName, cls: "muted" };
  const st = (h: HistoryEntry) => (h.command_id ? STATUS[h.status] : UNKNOWN);
  const time = (ms: number) =>
    new Date(ms).toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
</script>

{#if items.length === 0}
  <div class="empty">
    <Icon name="clock" size={20} />
    <p>{t("history.empty")}</p>
  </div>
{:else}
  <ul class="list" class:compact>
    {#each items as h, i (h.id)}
      <li style="--i: {Math.min(i, 8)}">
        <button type="button" class="row" title={t("history.repeat")} onclick={() => runText(h.phrase)}>
          {#if compact}
            <span class="bar {st(h).cls}"></span>
          {:else}
            <span class="chip {st(h).cls}"><Icon name={st(h).icon} size={12} stroke={2.5} /></span>
          {/if}
          <span class="body">
            <span class="phrase">{h.phrase}</span>
            <span class="status {st(h).cls}">{statusLabel(h)}</span>
          </span>
          {#if !compact}<span class="time num">{time(h.ts)}</span>{/if}
          <span class="again" aria-hidden="true"><Icon name="repeat" size={14} /></span>
        </button>
      </li>
    {/each}
  </ul>
{/if}

<style>
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  li + li {
    border-top: 1px solid var(--divider);
  }
  li {
    animation: rise var(--t-slow) var(--ease-out) both;
    animation-delay: calc(var(--i) * 25ms);
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 4px;
    }
  }
  .row {
    width: 100%;
    min-height: 52px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    border: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--t-fast) ease;
  }
  .row:hover {
    background: var(--fill);
  }
  .row:active {
    background: var(--fill-press);
  }
  .chip {
    width: 24px;
    height: 24px;
    flex: none;
    border-radius: 50%;
    display: grid;
    place-items: center;
  }
  .chip.ok {
    background: rgba(62, 207, 142, 0.14);
    color: var(--ok);
  }
  .chip.err {
    background: rgba(240, 97, 109, 0.14);
    color: var(--err);
  }
  .chip.warn {
    background: rgba(245, 184, 61, 0.14);
    color: var(--warn);
  }
  .chip.muted {
    background: var(--fill);
    color: var(--muted);
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .phrase {
    font-size: 13.5px;
    line-height: 20px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
  }
  .status.err {
    color: var(--err);
  }
  .status.warn {
    color: var(--warn);
  }
  .time {
    font-size: 12px;
    color: var(--text-3);
    flex: none;
  }
  .again {
    width: 16px;
    flex: none;
    color: var(--text-2);
    opacity: 0;
    transition: opacity var(--t-fast) ease;
  }
  .row:hover .again {
    opacity: 1;
  }
  /* video26: accent bar + phrase + status, no dividers */
  .compact li + li {
    border-top: 0;
  }
  .compact .row {
    min-height: 40px;
    padding: 4px 8px 4px 4px;
    gap: 10px;
    border-radius: 8px;
  }
  .compact .phrase {
    font-size: 13px;
    line-height: 18px;
    font-weight: 600;
  }
  .compact .status {
    font-size: 11.5px;
    line-height: 15px;
    color: var(--text-3);
  }
  .bar {
    width: 2px;
    align-self: stretch;
    margin: 3px 0;
    border-radius: 2px;
    background: var(--accent);
    flex: none;
  }
  .bar.err {
    background: var(--err);
  }
  .bar.warn {
    background: var(--warn);
  }
  .bar.muted {
    background: var(--muted);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 40px 16px;
    color: var(--text-3);
    text-align: center;
  }
  .empty p {
    margin: 0;
    font-size: 13px;
  }
</style>
