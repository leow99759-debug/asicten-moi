<script lang="ts">
  // §3.4: «фраза → статус», time, click repeats the phrase.
  import Icon, { type IconName } from "./Icon.svelte";
  import type { HistoryEntry } from "../lib/bindings/HistoryEntry";
  import type { Status } from "../lib/bindings/Status";
  import { runText } from "../lib/commands";
  import { t } from "../lib/i18n";

  let { items }: { items: HistoryEntry[] } = $props();

  const STATUS: Record<Status, { icon: IconName; cls: string }> = {
    done: { icon: "check", cls: "ok" },
    error: { icon: "alert", cls: "err" },
    cancelled: { icon: "ban", cls: "muted" },
    no_internet: { icon: "wifiOff", cls: "warn" },
  };
  const time = (ms: number) =>
    new Date(ms).toLocaleTimeString("ru-RU", { hour: "2-digit", minute: "2-digit" });
</script>

{#if items.length === 0}
  <div class="empty">
    <Icon name="clock" size={22} />
    <p>{t("history.empty")}</p>
  </div>
{:else}
  <ul class="list">
    {#each items as h (h.id)}
      <li>
        <button type="button" class="row {STATUS[h.status].cls}" title={t("history.repeat")} onclick={() => runText(h.phrase)}>
          <span class="rail"></span>
          <span class="body">
            <span class="phrase">{h.phrase}</span>
            <span class="status">
              <Icon name={STATUS[h.status].icon} size={12} stroke={2.25} />
              {t(`status.${h.status}`)}
            </span>
          </span>
          <span class="meta">
            <span class="time">{time(h.ts)}</span>
            <span class="again"><Icon name="repeat" size={14} /></span>
          </span>
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
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    width: 100%;
    display: flex;
    align-items: stretch;
    gap: 10px;
    padding: 7px 8px 7px 6px;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background-color var(--t-fast) ease, transform var(--t-press) var(--ease-out);
  }
  .row:hover {
    background: var(--surface-hover);
  }
  .row:active {
    transform: scale(0.99);
  }
  .rail {
    width: 2px;
    border-radius: 2px;
    background: var(--accent);
    flex: none;
  }
  .row.err .rail {
    background: var(--err);
  }
  .row.warn .rail {
    background: var(--warn);
  }
  .row.muted .rail {
    background: var(--muted);
  }
  .body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .phrase {
    font-size: 13.5px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    color: var(--text-3);
  }
  .row.ok .status {
    color: color-mix(in srgb, var(--ok) 75%, var(--text-3));
  }
  .row.err .status {
    color: var(--err);
  }
  .row.warn .status {
    color: var(--warn);
  }
  .meta {
    display: flex;
    align-items: center;
    color: var(--text-3);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
    position: relative;
  }
  .again {
    position: absolute;
    right: 0;
    opacity: 0;
    color: var(--text);
    transition: opacity var(--t-fast) ease;
  }
  .row:hover .again {
    opacity: 1;
  }
  .row:hover .time {
    opacity: 0;
  }
  .time {
    transition: opacity var(--t-fast) ease;
  }
  .empty {
    display: grid;
    place-items: center;
    gap: 6px;
    padding: 32px 0;
    color: var(--text-3);
    text-align: center;
  }
  .empty p {
    margin: 0;
    font-size: 13px;
  }
</style>
