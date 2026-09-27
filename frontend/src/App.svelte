<script lang="ts">
  import { t } from "./lib/i18n";
  import { minimize, close } from "./lib/window";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
</script>

<div class="shell">
  <header class="titlebar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>{t("app.title")}</span>
    <div class="controls">
      <button type="button" aria-label={t("window.minimize")} title={t("window.minimize")} onclick={minimize}>&#x2013;</button>
      <button type="button" class="close" aria-label={t("window.close")} title={t("window.close")} onclick={close}>&#x2715;</button>
    </div>
  </header>
  <main></main>
  <ConfirmDialog />
</div>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    background: #15171c;
    color: #e5e7eb;
    font-family: "Segoe UI Variable", Inter, "Segoe UI", sans-serif;
    user-select: none;
  }
  :global(#app) {
    height: 100%;
  }
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
    border: 1px solid rgba(255, 255, 255, 0.06);
    box-sizing: border-box;
  }
  .titlebar {
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-left: 14px;
  }
  .title {
    font-size: 12px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    opacity: 0.7;
  }
  .controls button {
    width: 44px;
    height: 36px;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  .controls button:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .controls .close:hover {
    background: #e81123;
  }
  main {
    flex: 1;
  }
</style>
