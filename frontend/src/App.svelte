<script lang="ts">
  import "./styles/tokens.css";
  import { onMount } from "svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import WindowControls from "./components/WindowControls.svelte";
  import ListeningBar from "./components/ListeningBar.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import MainWindow from "./pages/MainWindow.svelte";
  import Dashboard from "./pages/Dashboard.svelte";
  import Placeholder from "./pages/Placeholder.svelte";
  import { app, connect } from "./lib/app.svelte";
  import { windowMaterial } from "./lib/commands";
  import { t } from "./lib/i18n";

  onMount(() => {
    const p = new URLSearchParams(location.search).get("page");
    if (p) app.page = p as typeof app.page;
    connect();
    // Windows 11 Mica behind the webview → let it show through (§3.2)
    windowMaterial().then((mica) => {
      if (mica) document.documentElement.dataset.material = "mica";
    });
  });
</script>

<div class="window">
  <div class="backdrop" aria-hidden="true"></div>
  <Sidebar />
  <div class="content">
    <header class="top" data-tauri-drag-region>
      <h1 class="section-title" data-tauri-drag-region>{t(`title.${app.page}`)}</h1>
      <div class="listen"><ListeningBar /></div>
      <WindowControls />
    </header>
    <main>
      {#key app.page}
        <div class="page">
          {#if app.page === "main"}
            <MainWindow />
          {:else if app.page === "dashboard"}
            <Dashboard />
          {:else if app.page === "editor"}
            <Placeholder icon="terminal" milestone="M5" />
          {:else if app.page === "addons"}
            <Placeholder icon="puzzle" milestone="M6" />
          {:else if app.page === "ai"}
            <Placeholder icon="sparkles" milestone="M13" />
          {:else if app.page === "settings"}
            <Placeholder icon="settings" milestone="M4" />
          {:else}
            <Placeholder icon="user" milestone="M4" />
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
    background: var(--bg-1);
    border: 1px solid var(--line);
    box-sizing: border-box;
  }
  :global(:root[data-material="mica"] body) {
    background: transparent;
  }
  :global(:root[data-material="mica"]) .window {
    background: rgba(16, 18, 23, 0.6);
  }
  /* soft accent light behind the glass (the orb's room glow) */
  .backdrop {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background:
      radial-gradient(900px 600px at 85% 55%, rgba(var(--accent-rgb), 0.1), transparent 60%),
      radial-gradient(700px 500px at 0% 0%, rgba(255, 255, 255, 0.025), transparent 60%);
  }
  .content {
    position: relative;
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    padding: 0 20px 20px 24px;
  }
  .top {
    height: 64px;
    flex: none;
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: center;
    gap: 24px;
  }
  .listen {
    justify-self: center;
    width: min(520px, 100%);
  }
  main {
    flex: 1;
    min-height: 0;
    position: relative;
  }
  .page {
    position: absolute;
    inset: 0;
    transition: opacity 200ms var(--ease-out), translate 200ms var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 6px;
    }
  }
</style>
