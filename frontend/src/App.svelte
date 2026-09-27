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
  import Settings from "./pages/Settings.svelte";
  import Editor from "./pages/Editor.svelte";
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
      {#key app.page}
        <div class="page">
          {#if app.page === "main"}
            <MainWindow />
          {:else if app.page === "dashboard"}
            <Dashboard />
          {:else if app.page === "editor"}
            <Editor />
          {:else if app.page === "addons"}
            <Placeholder page="addons" icon="puzzle" milestone="M6" />
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
    background: var(--bg-base);
  }
  :global(:root[data-material="mica"] body) {
    background: transparent;
  }
  :global(:root[data-material="mica"]) .window {
    background: rgba(12, 13, 16, 0.35);
  }
  :global(:root[data-material="mica"]) .layer {
    background: rgba(19, 21, 25, 0.78);
  }
  .right {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .titlebar {
    height: 48px;
    flex: none;
    display: grid;
    grid-template-columns: 1fr minmax(0, 460px) 1fr;
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
    background: var(--bg-layer);
    border-top: 1px solid var(--stroke);
    border-left: 1px solid var(--stroke);
    border-top-left-radius: var(--r-layer);
    overflow: hidden;
  }
  .page {
    position: absolute;
    inset: 0;
    overflow-y: auto;
    padding: 28px 32px 32px;
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
