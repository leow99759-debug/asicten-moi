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
  import { app, connect } from "./lib/app.svelte";
  import { windowMaterial } from "./lib/commands";

  let settingsTab = $state("general");
  onMount(() => {
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

<div class="window">
  <Sidebar />
  <div class="right">
    <header class="titlebar" data-tauri-drag-region>
      <div class="listen"><ListeningBar /></div>
      <WindowControls />
    </header>
    <main class="layer">
      {#if app.page !== "main"}<div class="glow" aria-hidden="true"></div>{/if}
      {#key app.page}
        <div class="page">
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
  /* static orb rings peeking in from the right edge (video26/30), no animation = free */
  .glow {
    position: absolute;
    right: -300px;
    top: 50%;
    width: 640px;
    height: 640px;
    translate: 0 -40%;
    border-radius: 50%;
    pointer-events: none;
    background: radial-gradient(
      circle,
      rgba(var(--accent-rgb), 0.55) 0 22%,
      rgba(var(--accent-rgb), 0.3) 22.5% 31%,
      rgba(var(--accent-rgb), 0.16) 31.5% 40%,
      rgba(var(--accent-rgb), 0.07) 40.5% 50%,
      transparent 50.5%
    );
    filter: blur(1px);
    opacity: 0.7;
  }
  .page {
    position: absolute;
    inset: 0;
    overflow-y: auto;
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
</style>
