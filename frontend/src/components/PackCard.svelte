<script lang="ts">
  // Add-on card (§9): app icon, name, what it does, one-click [Установить] / [✓ Установлено].
  import BrandIcon from "./BrandIcon.svelte";
  import Icon from "./Icon.svelte";
  import type { Addon } from "../lib/bindings/Addon";
  import { plural } from "../lib/addons";
  import { t } from "../lib/i18n";

  let { addon, busy = false, i = 0, ontoggle }: { addon: Addon; busy?: boolean; i?: number; ontoggle: (on: boolean) => void } =
    $props();
  const p = $derived(addon.pack);
  const words = $derived(t("addons.cmd_forms").split("|") as [string, string, string]);
</script>

<article class="pc card" class:on={addon.installed} style="--i: {Math.min(i, 12)}">
  <BrandIcon icon={p.icon} color={p.color} name={p.name} size={64} glyph />
  <div class="tx">
    <h3>{p.name}</h3>
    <p class="desc" title={p.description}>{p.description}</p>
  </div>
  <div class="foot">
    <span class="meta">{plural(p.commands.length, words)}{#if p.popular}{" · "}<span class="hot">{t("addons.popular")}</span>{/if}</span>
    {#if addon.default}
      <span class="badge"><Icon name="check" size={12} stroke={2.5} /> {t("addons.builtin")}</span>
    {:else if addon.installed}
      <button type="button" class="btn inst" disabled={busy} onclick={() => ontoggle(false)} aria-label={t("addons.uninstall")}>
        <span class="a"><Icon name="check" size={14} stroke={2.5} /> {t("addons.installed")}</span>
        <span class="b"><Icon name="trash" size={14} /> {t("addons.uninstall")}</span>
      </button>
    {:else}
      <button type="button" class="btn primary" disabled={busy} onclick={() => ontoggle(true)}>
        <Icon name="download" size={14} stroke={2} />
        {t("addons.install")}
      </button>
    {/if}
  </div>
</article>

<style>
  /* video30: big dark icon tile left, title + description right, status badge bottom-right */
  .pc {
    position: relative;
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    column-gap: 16px;
    align-items: start;
    padding: 12px 12px 12px;
    min-width: 0;
    animation: rise var(--t-slow) var(--ease-out) both;
    animation-delay: calc(var(--i) * 24ms);
    transition:
      background-color var(--t-fast) ease,
      border-color var(--t-fast) ease;
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 6px;
    }
  }
  .pc:hover {
    background: var(--bg-card-hover);
    border-color: var(--stroke-strong);
  }
  .tx {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding-top: 4px;
  }
  h3 {
    margin: 0;
    font-size: 14.5px;
    line-height: 20px;
    font-weight: 700;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    font-size: 11.5px;
    line-height: 16px;
    color: var(--text-3);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .hot {
    color: var(--warn);
  }
  .desc {
    margin: 0;
    font-size: 12px;
    line-height: 17px;
    color: var(--text-2);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    min-height: 34px;
  }
  .foot {
    grid-column: 2;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 6px;
  }
  .foot .btn {
    height: 30px;
    padding: 0 12px;
    font-size: 12.5px;
    gap: 6px;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 4px;
    font-size: 12.5px;
    font-weight: 550;
    color: var(--text-3);
  }
  /* installed: calm label, turns into «Удалить» on hover (same width, crossfade) */
  /* video30: green outlined «✓ Установлено» */
  .inst {
    display: inline-grid;
    color: var(--ok);
    border-color: rgba(62, 207, 142, 0.55);
    background: rgba(62, 207, 142, 0.08);
    box-shadow: none;
  }
  .inst > span {
    grid-area: 1 / 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    transition:
      opacity var(--t-fast) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .inst .b {
    opacity: 0;
    transform: translateY(3px);
    color: var(--err);
  }
  .inst:hover,
  .inst:focus-visible {
    background: color-mix(in srgb, var(--err) 12%, transparent);
    border-color: color-mix(in srgb, var(--err) 30%, transparent);
  }
  .inst:hover .a,
  .inst:focus-visible .a {
    opacity: 0;
    transform: translateY(-3px);
  }
  .inst:hover .b,
  .inst:focus-visible .b {
    opacity: 1;
    transform: none;
  }
</style>
