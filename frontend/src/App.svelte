<script lang="ts">
  import "./styles/tokens.css";
  import { onMount } from "svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import WindowControls from "./components/WindowControls.svelte";
  import ListeningBar from "./components/ListeningBar.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import MainWindow from "./pages/MainWindow.svelte";
  import Placeholder from "./pages/Placeholder.svelte";
  import Settings from "./pages/Settings.svelte";
  import Editor from "./pages/Editor.svelte";
  import Addons from "./pages/Addons.svelte";
  import Orb from "./components/Orb.svelte";
  import { app, connect } from "./lib/app.svelte";
  import { windowMaterial } from "./lib/commands";

  let settingsTab = $state("general");
  let win: HTMLDivElement;
  // cards rise in only right after a page switch, not whenever one mounts later (Luxify tour)
  let entering = $state(true);
  $effect(() => {
    void app.page;
    entering = true;
    const id = setTimeout(() => (entering = false), 700);
    return () => clearTimeout(id);
  });
  const level = $derived(Math.max(app.micLevel, app.ttsLevel));
  const active = $derived(app.state === "listening" || app.state === "speaking" || app.state === "processing");

  onMount(() => {
    // window open (hidden window = destroyed webview, so mount = open): soft zoom-in like Luxify
    if (document.documentElement.dataset.motion !== "off" && !matchMedia("(prefers-reduced-motion: reduce)").matches)
      win.animate(
        [
          { opacity: 0, transform: "scale(0.975) translateY(8px)" },
          { opacity: 1, transform: "none" },
        ],
        { duration: 420, easing: "cubic-bezier(0.23, 1, 0.32, 1)" },
      );
    const q = new URLSearchParams(location.search);
    settingsTab = q.get("tab") ?? "general";
    const p = q.get("page");
    if (p) app.page = p as typeof app.page;
    connect();
    // Windows 11 Mica behind the webview → let it show through (§3.2)
    windowMaterial().then((mica) => {
      if (mica) document.documentElement.dataset.material = "mica";
    });
  });
</script>

<div class="window" bind:this={win}>
  <Sidebar />
  <div class="right">
    <!-- one orb spot for every page: live canvas on the main page, static rings elsewhere -->
    <div class="orb" aria-hidden="true">
      {#if app.page === "main"}<Orb {level} {active} />{:else}<div class="rings"></div>{/if}
    </div>
    <header class="titlebar" data-tauri-drag-region>
      <div class="listen"><ListeningBar /></div>
      <WindowControls />
    </header>
    <main class="layer">
      {#key app.page}
        <div class="page" class:enter={entering}>
          {#if app.page === "main"}
            <MainWindow />
          {:else if app.page === "editor"}
            <Editor />
          {:else if app.page === "addons"}
            <Addons />
          {:else if app.page === "ai"}
            <Placeholder page="ai" icon="sparkles" milestone="M13" />
          {:else if app.page === "settings"}
            <Settings bind:tab={settingsTab} />
          {:else}
            <Settings tab="about" />
          {/if}
        </div>
      {/key}
    </main>
  </div>
  <ConfirmDialog />
</div>

<style>
  .window {
    position: relative;
    height: 100%;
    display: flex;
    overflow: hidden;
    background:
      radial-gradient(90% 60% at 100% 0%, rgba(var(--accent-rgb), 0.07), transparent 60%),
      radial-gradient(70% 50% at 0% 100%, rgba(var(--accent-rgb), 0.04), transparent 60%),
      linear-gradient(180deg, #1b1c21, var(--bg-base));
  }
  :global(:root[data-material="mica"] body) {
    background: transparent;
  }
  :global(:root[data-material="mica"]) .window {
    background: rgba(20, 21, 25, 0.62);
  }
  .right {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
  }
  .titlebar {
    height: 38px;
    flex: none;
    position: relative;
    z-index: 5;
    display: grid;
    grid-template-columns: 1fr minmax(0, 440px) 1fr;
    align-items: center;
  }
  .listen {
    grid-column: 2;
  }
  .titlebar :global(.controls) {
    grid-column: 3;
    justify-self: end;
    height: 100%;
  }
  .layer {
    flex: 1;
    min-height: 0;
    position: relative;
    overflow: hidden;
  }
  /* Luxify: big accent rings from the right edge on every page. Same geometry as the canvas
     Orb (disc radii 1 / .8 / .61 / .44 of 42 % box), alphas pre-composited; static = free. */
  .orb {
    position: absolute;
    top: 50%;
    right: 0;
    height: 124%;
    aspect-ratio: 1;
    transform: translate(44%, -50%);
    pointer-events: none;
  }
  .rings {
    width: 100%;
    height: 100%;
    border-radius: 50%;
    background:
      radial-gradient(circle at 44% 40%, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.025) 20%, transparent 37%),
      radial-gradient(
        circle closest-side,
        rgba(var(--accent-rgb), 0.91) 0 35.9%,
        rgba(var(--accent-rgb), 0.65) 37.4% 49.7%,
        rgba(var(--accent-rgb), 0.39) 51.7% 65.2%,
        rgba(var(--accent-rgb), 0.2) 67.9% 81.5%,
        transparent 84.8%
      );
  }
  @media (max-width: 900px) {
    .orb {
      transform: translate(62%, -50%);
      opacity: 0.6;
    }
  }
  .page {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2px 28px 28px;
    box-sizing: border-box;
    transition:
      opacity var(--t-slow) var(--ease-out),
      translate var(--t-slow) var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 6px;
    }
  }
  /* page switch: blocks rise in a 40 ms staircase (Luxify t-rise) */
  .page.enter :global(:is(.ph, .hd, .tabs, .toolbar, .split, .group, .card.panel, .empty)) {
    animation: rise 500ms var(--ease-out) both;
  }
  .page.enter :global(:is(.ph, .hd, .tabs, .toolbar, .split, .group, .card.panel, .empty):nth-child(2)) {
    animation-delay: 40ms;
  }
  .page.enter :global(:is(.ph, .hd, .tabs, .toolbar, .split, .group, .card.panel, .empty):nth-child(3)) {
    animation-delay: 80ms;
  }
  .page.enter :global(:is(.ph, .hd, .tabs, .toolbar, .split, .group, .card.panel, .empty):nth-child(4)) {
    animation-delay: 120ms;
  }
  .page.enter :global(:is(.ph, .hd, .tabs, .toolbar, .split, .group, .card.panel, .empty):nth-child(n + 5)) {
    animation-delay: 160ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(10px);
    }
  }
</style>
