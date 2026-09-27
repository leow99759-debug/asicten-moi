<script lang="ts">
  // Windows 11 caption buttons: 46×40, close turns red on hover.
  import Icon from "./Icon.svelte";
  import { minimize, close, toggleMaximize } from "../lib/window";
  import { t } from "../lib/i18n";

  let maximized = $state(false);
  async function max() {
    maximized = await toggleMaximize();
  }
</script>

<div class="controls">
  <button type="button" aria-label={t("window.minimize")} title={t("window.minimize")} onclick={minimize}>
    <Icon name="minus" size={16} stroke={1.5} />
  </button>
  <button type="button" aria-label={t("window.maximize")} title={t("window.maximize")} onclick={max}>
    <Icon name={maximized ? "restore" : "maximize"} size={14} stroke={1.5} />
  </button>
  <button type="button" class="close" aria-label={t("window.close")} title={t("window.close")} onclick={close}>
    <Icon name="x" size={16} stroke={1.5} />
  </button>
</div>

<style>
  .controls {
    display: flex;
    align-self: stretch;
  }
  button {
    width: 46px;
    display: grid;
    place-items: center;
    border: 0;
    background: transparent;
    color: var(--text-2);
    cursor: default;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  button:hover {
    background: var(--fill);
    color: var(--text);
  }
  button:active {
    background: var(--fill-press);
  }
  .close:hover {
    background: #c42b1c;
    color: #fff;
  }
  .close:active {
    background: #b0271a;
  }
  button:focus-visible {
    outline-offset: -2px;
  }
</style>
