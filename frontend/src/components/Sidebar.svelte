<script lang="ts">
  import Icon, { type IconName } from "./Icon.svelte";
  import Logo from "./Logo.svelte";
  import { app, type Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n";

  const items: { page: Page; icon: IconName }[] = [
    { page: "main", icon: "home" },
    { page: "editor", icon: "terminal" },
    { page: "addons", icon: "puzzle" },
    { page: "ai", icon: "sparkles" },
    { page: "settings", icon: "settings" },
  ];
</script>

<nav class="sidebar" data-tauri-drag-region>
  <button
    type="button"
    class="logo pressable"
    aria-label={t("nav.dashboard")}
    title={t("nav.dashboard")}
    onclick={() => (app.page = "dashboard")}>
    <Logo size={34} />
  </button>
  <div class="items">
    {#each items as it (it.page)}
      <button
        type="button"
        class="item pressable"
        class:active={app.page === it.page}
        aria-label={t(`nav.${it.page}`)}
        aria-current={app.page === it.page ? "page" : undefined}
        data-tip={t(`nav.${it.page}`)}
        onclick={() => (app.page = it.page)}>
        <Icon name={it.icon} />
      </button>
    {/each}
  </div>
  <button
    type="button"
    class="item pressable"
    class:active={app.page === "profile"}
    aria-label={t("nav.profile")}
    data-tip={t("nav.profile")}
    onclick={() => (app.page = "profile")}>
    <Icon name="user" />
  </button>
</nav>

<style>
  .sidebar {
    width: 64px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 0 16px;
    gap: 8px;
    background: rgba(12, 14, 18, 0.55);
    border-right: 1px solid var(--line);
  }
  .logo {
    border: 0;
    padding: 0;
    background: none;
    cursor: pointer;
    border-radius: 50%;
    margin-bottom: 18px;
    filter: drop-shadow(0 4px 14px rgba(var(--accent-rgb), 0.45));
  }
  .items {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex: 1;
    justify-content: center;
  }
  .item {
    position: relative;
    width: 40px;
    height: 40px;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    border: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition:
      transform var(--t-press) var(--ease-out),
      background-color var(--t-fast) ease,
      color var(--t-fast) ease,
      border-color var(--t-fast) ease;
  }
  .item:hover {
    color: var(--text);
    background: var(--surface-hover);
  }
  .item.active {
    color: var(--text);
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--line-2);
  }
  .item.active::before {
    content: "";
    position: absolute;
    left: -12px;
    top: 10px;
    bottom: 10px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
  }
  /* tooltip: delayed first time, instant while moving along the bar */
  .item::after {
    content: attr(data-tip);
    position: absolute;
    left: calc(100% + 12px);
    top: 50%;
    transform: translate(-4px, -50%);
    padding: 5px 10px;
    border-radius: var(--r-sm);
    background: #2a2e37;
    border: 1px solid var(--line-2);
    color: var(--text);
    font-size: 12px;
    white-space: nowrap;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--t-fast) var(--ease-out), transform var(--t-fast) var(--ease-out);
    transition-delay: 0ms;
    z-index: 10;
  }
  .item:hover::after {
    opacity: 1;
    transform: translate(0, -50%);
    transition-delay: 350ms;
  }
  .items:hover .item:hover::after {
    transition-delay: 120ms;
  }
</style>
