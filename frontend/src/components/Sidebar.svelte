<script lang="ts">
  // Navigation rail (video26/video30, Luxify tour): logo on top, icon buttons; the selected
  // frame slides to the new button instead of jumping. Mic + profile at the bottom, labels
  // in tooltips that slide out to the right.
  import { untrack } from "svelte";
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

  const btns: Partial<Record<Page, HTMLButtonElement>> = {};
  let ind = $state({ y: 0, on: false, moved: false });
  let vh = $state(0); // profile sits at the bottom: re-measure on resize
  $effect(() => {
    void vh;
    const b = btns[app.page];
    // read the old flag untracked: this effect writes `ind`, tracking it would loop forever
    const was = untrack(() => ind.on);
    if (!b) return void (ind.on = false);
    ind = { y: b.offsetTop, on: true, moved: was };
  });

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
    bind:this={btns[page]}
    onclick={() => (app.page = page)}>
    <Icon name={icon} size={19} />
    <span class="tip">{t(`nav.${page}`)}</span>
  </button>
{/snippet}

<svelte:window bind:innerHeight={vh} />

<nav class="rail" aria-label={t("nav.aria")}>
  <span class="ind" class:on={ind.on} class:moved={ind.moved} style="translate: 0 {ind.y}px" aria-hidden="true"></span>
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
      onclick={toggleMic}>
      <Icon name={micOff ? "micOff" : "mic"} size={18} />
      <span class="tip">{micOff ? t("mic.on") : t("mic.off")}</span>
    </button>
    {@render nav("profile", "user")}
  </div>
</nav>

<style>
  .rail {
    position: relative;
    z-index: 6;
    width: 64px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 14px 0 16px;
    box-sizing: border-box;
    border-radius: 0 16px 16px 0;
    background: rgba(255, 255, 255, 0.035);
    box-shadow: inset -1px 0 0 var(--divider);
  }
  /* selected frame: white outline + inner glow (Luxify), slides between buttons */
  .ind {
    position: absolute;
    top: 0;
    left: 12px;
    width: 40px;
    height: 40px;
    box-sizing: border-box;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.7);
    background: rgba(255, 255, 255, 0.05);
    box-shadow: inset 0 0 14px rgba(255, 255, 255, 0.09);
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--t-base) ease;
  }
  .ind.on {
    opacity: 1;
  }
  .ind.moved {
    transition:
      translate 400ms var(--ease-out),
      opacity var(--t-base) ease;
  }
  .tip {
    position: absolute;
    left: calc(100% + 12px);
    top: 50%;
    padding: 5px 10px;
    border-radius: 8px;
    border: 1px solid var(--stroke-strong);
    background: #3a3b40;
    color: var(--text);
    font-size: 12.5px;
    line-height: 16px;
    font-weight: 500;
    white-space: nowrap;
    box-shadow: 0 10px 24px -10px rgba(0, 0, 0, 0.6);
    translate: -4px -50%;
    opacity: 0;
    pointer-events: none;
    transition:
      opacity 160ms ease,
      translate 160ms var(--ease-out);
  }
  .item:hover .tip,
  .item:focus-visible .tip {
    opacity: 1;
    translate: 0 -50%;
    transition-delay: 350ms;
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
    position: relative;
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
  .item:active :global(svg) {
    transform: scale(0.9);
  }
  .item :global(svg) {
    transition: transform var(--t-fast) var(--ease-out);
  }
  .item.active {
    color: var(--text);
  }
  .item.active:hover {
    background: transparent;
  }
  .mic.muted {
    color: var(--err);
  }
</style>
