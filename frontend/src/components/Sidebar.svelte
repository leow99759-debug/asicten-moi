<script lang="ts">
  // Navigation rail (video26/video30): logo on top, icon buttons, selected = outlined glass
  // square; mic mute and profile at the bottom. Labels live in tooltips.
  import Icon, { type IconName } from "./Icon.svelte";
  import Logo from "./Logo.svelte";
  import { app, toggleMic, type Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n";

  const items: { page: Page; icon: IconName }[] = [
    { page: "main", icon: "home" },
    { page: "editor", icon: "terminal" },
    { page: "addons", icon: "puzzle" },
    { page: "ai", icon: "sparkles" },
    { page: "settings", icon: "settings" },
  ];

  const micOff = $derived(app.state === "mic_off");
  const busy = $derived(app.state === "listening" || app.state === "processing" || app.state === "speaking");
</script>

{#snippet nav(page: Page, icon: IconName)}
  <button
    type="button"
    class="item"
    class:active={app.page === page}
    aria-current={app.page === page ? "page" : undefined}
    aria-label={t(`nav.${page}`)}
    title={t(`nav.${page}`)}
    onclick={() => (app.page = page)}>
    <Icon name={icon} size={19} />
  </button>
{/snippet}

<nav class="rail" aria-label={t("nav.aria")}>
  <div class="brand" class:busy data-tauri-drag-region>
    <Logo size={30} />
  </div>

  <div class="items">
    {#each items as it (it.page)}
      {@render nav(it.page, it.icon)}
    {/each}
  </div>

  <div class="bottom">
    <button
      type="button"
      class="item mic"
      class:muted={micOff}
      aria-pressed={micOff}
      aria-label={micOff ? t("mic.on") : t("mic.off")}
      title={micOff ? t("mic.on") : t("mic.off")}
      onclick={toggleMic}>
      <Icon name={micOff ? "micOff" : "mic"} size={18} />
    </button>
    {@render nav("profile", "user")}
  </div>
</nav>

<style>
  .rail {
    width: 64px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 0 16px;
    box-sizing: border-box;
    border-right: 1px solid var(--divider);
    background: rgba(255, 255, 255, 0.012);
  }
  .brand {
    height: 40px;
    display: grid;
    place-items: center;
    margin-bottom: 56px;
  }
  .brand :global(svg) {
    filter: drop-shadow(0 2px 12px rgba(var(--accent-rgb), 0.5));
    transition: filter var(--t-slow) ease;
  }
  .brand.busy :global(svg) {
    filter: drop-shadow(0 0 16px rgba(var(--accent-rgb), 0.85));
  }
  .items {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    flex: 1;
  }
  .bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
  .item {
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      border-color var(--t-fast) ease,
      color var(--t-fast) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .item:hover {
    background: var(--fill);
    color: var(--text);
  }
  .item:active {
    transform: scale(0.94);
  }
  .item.active {
    color: var(--text);
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.28);
    box-shadow: var(--rim);
  }
  .mic.muted {
    color: var(--err);
  }
</style>
