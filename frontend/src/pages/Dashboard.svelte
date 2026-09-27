<script lang="ts">
  // §3.3 Главная: greeting + status, section tiles, recent commands.
  import Icon, { type IconName } from "../components/Icon.svelte";
  import HistoryList from "../components/HistoryList.svelte";
  import { app, type Page } from "../lib/app.svelte";
  import { ad } from "../lib/addons.svelte";
  import { t } from "../lib/i18n";

  const tiles: { icon: IconName; label: () => string; sub: () => string; page: Page; tint: string; cat?: string }[] = [
    { icon: "wave", label: () => t("nav.main"), sub: () => t("tile.voice"), page: "main", tint: "var(--accent)" },
    { icon: "list", label: () => `${app.commandCount} ${t("tile.commands")}`, sub: () => t("tile.all"), page: "editor", tint: "#8b7cf6" },
    { icon: "terminal", label: () => t("nav.editor"), sub: () => t("tile.create"), page: "editor", tint: "#22c1a4" },
    { icon: "gamepad", label: () => t("tile.games"), sub: () => t("tile.packs"), page: "addons", tint: "#f97362", cat: "Игровые сервисы" },
    { icon: "globe", label: () => t("tile.browser"), sub: () => t("tile.packs"), page: "addons", tint: "#3fb3f5", cat: "Браузеры" },
    { icon: "sparkles", label: () => t("tile.ai"), sub: () => t("tile.off"), page: "ai", tint: "#e0a33a" },
  ];

  function open(tile: (typeof tiles)[number]) {
    if (tile.cat) {
      ad.tab = "packs";
      ad.cat = tile.cat;
    }
    app.page = tile.page;
  }

  const hour = new Date().getHours();
  const greet = hour < 5 ? "greet.night" : hour < 12 ? "greet.morning" : hour < 18 ? "greet.day" : "greet.evening";
</script>

<div class="dash">
  <header class="hero">
    <h1 class="t-display">{t(greet)}</h1>
    <p>
      <span class="live" class:off={app.state === "mic_off"}></span>
      {t(`state.${app.state}`)} · <span class="num">{app.commandCount} {t("tile.commands")}</span>
    </p>
  </header>

  <div class="tiles">
    {#each tiles as tile, i (i)}
      <button type="button" class="tile card" style="--i: {i}; --tint: {tile.tint}" onclick={() => open(tile)}>
        <span class="ic"><Icon name={tile.icon} size={18} /></span>
        <span class="tx">
          <span class="lb">{tile.label()}</span>
          <span class="sb">{tile.sub()}</span>
        </span>
        <span class="go"><Icon name="chevron" size={16} /></span>
      </button>
    {/each}
  </div>

  <section>
    <div class="gh">
      <h2 class="t-group">{t("dash.recent")}</h2>
      <button type="button" class="btn subtle more" onclick={() => (app.page = "main")}>{t("dash.all")}</button>
    </div>
    <div class="card hist">
      <HistoryList items={app.history.slice(0, 4)} />
    </div>
  </section>
</div>

<style>
  .dash {
    max-width: 880px;
    margin: 0 auto;
  }
  .hero {
    margin: 8px 0 24px;
  }
  .hero p {
    margin: 6px 0 0;
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--text-2);
  }
  .live {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--ok);
    box-shadow: 0 0 0 3px rgba(62, 207, 142, 0.18);
    margin-right: 2px;
  }
  .live.off {
    background: var(--muted);
    box-shadow: none;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
    margin-bottom: 28px;
  }
  .tile {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px;
    text-align: left;
    cursor: pointer;
    animation: rise var(--t-slow) var(--ease-out) both;
    animation-delay: calc(var(--i) * 30ms);
    transition:
      background-color var(--t-fast) ease,
      transform var(--t-fast) var(--ease-out);
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 6px;
    }
  }
  .tile:hover {
    background: var(--bg-card-hover);
  }
  .tile:active {
    transform: scale(0.98);
  }
  .ic {
    width: 36px;
    height: 36px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    color: var(--tint);
    background: color-mix(in srgb, var(--tint) 16%, transparent);
  }
  .tx {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .lb {
    font-size: 14px;
    line-height: 20px;
    font-weight: 600;
    letter-spacing: -0.006em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sb {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
  }
  .go {
    color: var(--text-3);
    transition:
      transform var(--t-base) var(--ease-out),
      color var(--t-fast) ease;
  }
  .tile:hover .go {
    color: var(--text-2);
    transform: translateX(2px);
  }
  .gh {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }
  .gh .t-group {
    margin-bottom: 0;
  }
  .more {
    height: 28px;
    padding: 0 8px;
    font-size: 12.5px;
    color: var(--accent-text);
  }
  .hist {
    overflow: hidden;
  }
  @media (max-width: 999px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
