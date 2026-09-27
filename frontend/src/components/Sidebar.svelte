<script lang="ts">
  // Navigation (SYSTEM.md): icon + label, selected = fill pill + accent indicator.
  // Bottom: Discord-style status panel — live state and mic mute.
  import Icon, { type IconName } from "./Icon.svelte";
  import Logo from "./Logo.svelte";
  import { app, type Page } from "../lib/app.svelte";
  import { setMode } from "../lib/commands";
  import { t } from "../lib/i18n";

  const items: { page: Page; icon: IconName; soon?: boolean }[] = [
    { page: "dashboard", icon: "home" },
    { page: "main", icon: "wave" },
    { page: "editor", icon: "terminal" },
    { page: "addons", icon: "puzzle" },
    { page: "ai", icon: "sparkles", soon: true },
  ];

  const micOff = $derived(app.state === "mic_off");
  const busy = $derived(app.state === "listening" || app.state === "processing" || app.state === "speaking");
  const level = $derived(Math.min(1, Math.max(app.micLevel, app.ttsLevel) * 1.4));

  function toggleMic() {
    const off = !micOff;
    app.state = off ? "mic_off" : "idle";
    setMode(off ? "mic_off" : "mic_on");
  }
</script>

{#snippet nav(page: Page, icon: IconName, soon = false)}
  <button
    type="button"
    class="item"
    class:active={app.page === page}
    aria-current={app.page === page ? "page" : undefined}
    title={t(`nav.${page}`)}
    onclick={() => (app.page = page)}>
    <Icon name={icon} size={18} />
    <span class="label">{t(`nav.${page}`)}</span>
    {#if soon}<span class="soon">{t("soon.badge")}</span>{/if}
  </button>
{/snippet}

<nav class="sidebar" aria-label={t("nav.aria")}>
  <div class="brand" data-tauri-drag-region>
    <Logo size={28} />
    <div class="name" data-tauri-drag-region>
      <span class="n">Jarvis</span>
      <span class="s">{t("app.subtitle")}</span>
    </div>
  </div>

  <div class="items">
    {#each items as it (it.page)}
      {@render nav(it.page, it.icon, it.soon)}
    {/each}
  </div>

  <div class="bottom">
    {@render nav("settings", "settings")}
    <div class="status" class:busy class:off={micOff}>
      <span class="dot" style="--lv: {level.toFixed(3)}" aria-hidden="true"><span></span></span>
      <span class="txt">
        <span class="who">{t("status.name")}</span>
        <span class="st">{t(`state.${app.state}`)}</span>
      </span>
      <button
        type="button"
        class="mic"
        class:muted={micOff}
        aria-pressed={micOff}
        aria-label={micOff ? t("mic.on") : t("mic.off")}
        title={micOff ? t("mic.on") : t("mic.off")}
        onclick={toggleMic}>
        <Icon name={micOff ? "micOff" : "mic"} size={17} />
      </button>
    </div>
  </div>
</nav>

<style>
  .sidebar {
    width: 236px;
    flex: none;
    display: flex;
    flex-direction: column;
    padding: 0 10px 10px;
    box-sizing: border-box;
  }
  .brand {
    height: 56px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 8px;
    margin-bottom: 6px;
  }
  .brand :global(svg) {
    flex: none;
    filter: drop-shadow(0 2px 10px rgba(var(--accent-rgb), 0.45));
  }
  .name {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .n {
    font-size: 15px;
    line-height: 18px;
    font-weight: 750;
    letter-spacing: -0.015em;
  }
  .s {
    font-size: 11.5px;
    line-height: 14px;
    color: var(--text-3);
    font-weight: 500;
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }
  .bottom {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .item {
    position: relative;
    height: 36px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-2);
    font-size: 13.5px;
    font-weight: 500;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .item:hover {
    background: var(--fill);
    color: var(--text);
  }
  .item:active {
    background: var(--fill-press);
  }
  .item.active {
    background: var(--fill);
    color: var(--text);
    font-weight: 600;
  }
  .item.active :global(svg) {
    color: var(--accent-text);
  }
  /* Fluent selection indicator */
  .item::before {
    content: "";
    position: absolute;
    left: 0;
    top: 50%;
    width: 3px;
    height: 16px;
    margin-top: -8px;
    border-radius: 2px;
    background: var(--accent);
    transform: scaleY(0);
    transition: transform var(--t-base) var(--spring);
  }
  .item.active::before {
    transform: scaleY(1);
  }
  .label {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .soon {
    font-size: 10.5px;
    line-height: 16px;
    font-weight: 600;
    padding: 0 6px;
    border-radius: var(--r-xs);
    background: var(--fill);
    color: var(--text-3);
  }

  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 52px;
    padding: 0 8px 0 10px;
    border-radius: var(--r-md);
    background: rgba(0, 0, 0, 0.22);
    border: 1px solid var(--stroke);
  }
  .dot {
    position: relative;
    width: 30px;
    height: 30px;
    flex: none;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent-soft);
  }
  .dot span {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--accent);
    box-shadow: 0 0 0 calc(var(--lv) * 7px) rgba(var(--accent-rgb), 0.28);
    transition: box-shadow 80ms linear;
  }
  .status.off .dot {
    background: var(--fill);
  }
  .status.off .dot span {
    background: var(--muted);
    box-shadow: none;
  }
  .txt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .who {
    font-size: 13px;
    line-height: 16px;
    font-weight: 650;
  }
  .st {
    font-size: 11.5px;
    line-height: 15px;
    color: var(--text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status.busy .st {
    color: var(--accent-text);
  }
  .mic {
    width: 32px;
    height: 32px;
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
    transform: scale(0.94);
  }
  .mic.muted {
    color: var(--err);
  }

  /* compact rail on narrow windows */
  @media (max-width: 999px) {
    .sidebar {
      width: 64px;
      padding: 0 8px 10px;
    }
    .brand {
      justify-content: center;
      padding: 0;
    }
    .name,
    .label,
    .soon,
    .txt,
    .mic {
      display: none;
    }
    .item {
      justify-content: center;
      padding: 0;
    }
    .status {
      justify-content: center;
      padding: 0;
      background: none;
      border: 0;
    }
  }
</style>
