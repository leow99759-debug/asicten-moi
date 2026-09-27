<script lang="ts">
  // §3.3 Главная: hero + tiles. Tiles open their section.
  import Logo from "../components/Logo.svelte";
  import Icon, { type IconName } from "../components/Icon.svelte";
  import { app, type Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n";

  const tiles: { icon: IconName; label: () => string; sub: () => string; page: Page }[] = [
    { icon: "sparkles", label: () => t("tile.ai"), sub: () => t("tile.off"), page: "ai" },
    { icon: "list", label: () => `${app.commandCount} ${t("tile.commands")}`, sub: () => t("tile.all"), page: "editor" },
    { icon: "terminal", label: () => t("nav.editor"), sub: () => t("tile.create"), page: "editor" },
    { icon: "gamepad", label: () => t("tile.games"), sub: () => t("tile.packs"), page: "addons" },
    { icon: "globe", label: () => t("tile.browser"), sub: () => t("tile.packs"), page: "addons" },
    { icon: "monitor", label: () => t("tile.system"), sub: () => t("tile.packs"), page: "addons" },
  ];
</script>

<div class="dash">
  <div class="hero">
    <div class="mark"><Logo size={88} /></div>
    <h1>{t("app.name")}</h1>
    <p>{t("app.tagline")}</p>
  </div>
  <div class="tiles">
    {#each tiles as tile, i (i)}
      <button type="button" class="tile glass pressable" style="--i: {i}" onclick={() => (app.page = tile.page)}>
        <span class="ic"><Icon name={tile.icon} size={22} /></span>
        <span class="lb">{tile.label()}</span>
        <span class="sb">{tile.sub()}</span>
      </button>
    {/each}
  </div>
</div>

<style>
  .dash {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 40px;
  }
  .hero {
    text-align: center;
  }
  .mark {
    display: inline-block;
    filter: drop-shadow(0 12px 40px rgba(var(--accent-rgb), 0.55));
  }
  h1 {
    margin: 18px 0 6px;
    font: 700 34px/1.1 var(--font-display);
    letter-spacing: -0.02em;
  }
  p {
    margin: 0;
    color: var(--text-2);
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 180px);
    gap: 12px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 16px;
    text-align: left;
    cursor: pointer;
    border-radius: var(--r-lg);
    transition:
      transform var(--t-press) var(--ease-out),
      background-color var(--t-fast) ease,
      border-color var(--t-fast) ease,
      opacity 400ms var(--ease-out),
      translate 400ms var(--ease-out);
    transition-delay: 0ms, 0ms, 0ms, calc(var(--i) * 40ms), calc(var(--i) * 40ms);
    @starting-style {
      opacity: 0;
      translate: 0 8px;
    }
  }
  .tile:hover {
    background: var(--surface-2);
    border-color: var(--line-2);
  }
  .ic {
    width: 38px;
    height: 38px;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    background: var(--accent-soft);
    color: var(--accent);
    margin-bottom: 10px;
  }
  .lb {
    font-weight: 600;
    font-size: 14px;
  }
  .sb {
    font-size: 12px;
    color: var(--text-3);
  }
</style>
