<script lang="ts">
  import Icon from "./Icon.svelte";
  import { minimize, close } from "../lib/window";
  import { t } from "../lib/i18n";
</script>

<div class="controls">
  <button type="button" aria-label={t("window.minimize")} title={t("window.minimize")} onclick={minimize}>
    <Icon name="minus" size={16} />
  </button>
  <button type="button" class="close" aria-label={t("window.close")} title={t("window.close")} onclick={close}>
    <Icon name="x" size={16} />
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 2px;
  }
  button {
    width: 34px;
    height: 30px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--r-sm);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    transition: background-color var(--t-fast) ease, color var(--t-fast) ease;
  }
  button:hover {
    background: rgba(255, 255, 255, 0.07);
    color: var(--text);
  }
  .close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
