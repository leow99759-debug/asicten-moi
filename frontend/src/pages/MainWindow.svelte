<script lang="ts">
  // ОСНОВНОЕ ОКНО (video26 t08): history + control panel on the left, the orb glowing in
  // from the right edge behind them, the last command as a glass toast over the orb.
  import HistoryList from "../components/HistoryList.svelte";
  import Orb from "../components/Orb.svelte";
  import Slider from "../components/Slider.svelte";
  import Toggle from "../components/Toggle.svelte";
  import Icon from "../components/Icon.svelte";
  import BrandIcon from "../components/BrandIcon.svelte";
  import { app, statusLabel, toggleMic, togglePrefix, toggleSilent } from "../lib/app.svelte";
  import { BRANDS } from "../lib/brands";
  import { cfg, saved } from "../lib/settings.svelte";
  import { t } from "../lib/i18n";

  const level = $derived(Math.max(app.micLevel, app.ttsLevel));
  const active = $derived(app.state === "listening" || app.state === "speaking" || app.state === "processing");
  const micOff = $derived(app.state === "mic_off");
  const last = $derived(app.history[0]);
  const brand = (id: string | null) => {
    const b = id?.split(".")[0] ?? "";
    return BRANDS[b] ? b : "i:mic";
  };
  const cap = (s: string) => s.charAt(0).toUpperCase() + s.slice(1);
</script>

<div class="main">
  <div class="stage" aria-hidden="true"><Orb {level} {active} /></div>

  <div class="col">
    <header class="hd">
      <h1 class="t-display">{t("title.main")}</h1>
      <button
        type="button"
        class="listen"
        class:on={active}
        class:off={micOff}
        onclick={toggleMic}
        aria-pressed={micOff}
        title={micOff ? t("mic.on") : t("mic.off")}
        aria-label={micOff ? t("mic.on") : t("mic.off")}>
        <Icon name={micOff ? "micOff" : "mic"} size={16} />
      </button>
    </header>

    <section class="card panel">
      <h2 class="ph">{t("main.history")}</h2>
      <HistoryList items={app.history.slice(0, 6)} compact />
    </section>

    <section class="card panel">
      <h2 class="ph">{t("main.panel")}</h2>
      <div class="rows">
        <div class="row">
          <span>{t("panel.prefix")}</span>
          <Toggle small label={t("panel.prefix")} checked={app.prefixMode} onchange={togglePrefix} />
        </div>
        <div class="row">
          <span>{t("panel.silent")}</span>
          <Toggle small label={t("panel.silent")} checked={app.silentMode} onchange={toggleSilent} />
        </div>
        <div class="row">
          <span>{t("panel.avatar")}</span>
          <Toggle
            small
            label={t("panel.avatar")}
            checked={cfg.value.ui.avatar}
            onchange={(v) => {
              cfg.value.ui.avatar = v;
              saved();
            }} />
        </div>
        <div class="row">
          <span>{t("panel.onTop")}</span>
          <Toggle
            small
            label={t("panel.onTop")}
            checked={cfg.value.ui.on_top}
            onchange={(v) => {
              cfg.value.ui.on_top = v;
              saved();
            }} />
        </div>
        <div class="row vol">
          <span>{t("panel.volume")}</span>
          <div class="vs">
            <Slider
              label={t("panel.volume")}
              value={cfg.value.voice_volume}
              oninput={(v) => {
                cfg.value.voice_volume = v;
                saved();
              }} />
          </div>
        </div>
      </div>
    </section>
  </div>

  {#if last}
    {#key last.id}
      <div class="toast">
        <BrandIcon icon={brand(last.command_id)} name={last.phrase} size={44} />
        <span class="tt">
          <span class="tp">{cap(last.phrase.replace(/^джарвис,?\s*/i, ""))}</span>
          <span class="ts {last.command_id ? last.status : ''}">{statusLabel(last)}</span>
        </span>
      </div>
    {/key}
  {/if}
</div>

<style>
  .main {
    position: relative;
    min-height: 100%;
  }
  /* the orb bleeds off the right edge like the video */
  .stage {
    position: absolute;
    top: 50%;
    right: -16%;
    width: min(760px, 72%, calc(100vh - 80px));
    aspect-ratio: 1;
    translate: 0 -50%;
    pointer-events: none;
  }
  .col {
    position: relative;
    width: min(460px, 50%);
    min-width: 340px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .hd {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 44px;
    margin-bottom: 2px;
  }
  .listen {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 8px;
    background: var(--accent);
    color: #fff;
    cursor: pointer;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.2),
      0 4px 14px rgba(var(--accent-rgb), 0.35);
    transition:
      transform var(--t-fast) var(--ease-out),
      box-shadow var(--t-base) ease;
  }
  .listen:active {
    transform: scale(0.94);
  }
  .listen.off {
    background: var(--err);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.2);
  }
  .listen.on {
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.2),
      0 0 0 4px rgba(var(--accent-rgb), 0.25),
      0 4px 18px rgba(var(--accent-rgb), 0.5);
  }
  .panel {
    padding: 14px 14px 12px;
  }
  .ph {
    margin: 0 0 10px 2px;
    font-size: 14px;
    line-height: 20px;
    font-weight: 650;
    letter-spacing: -0.006em;
  }
  .rows {
    display: flex;
    flex-direction: column;
  }
  .row {
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 0 4px 0 2px;
    font-size: 13px;
    color: var(--text-2);
  }
  .row + .row {
    border-top: 1px solid var(--divider);
  }
  .vs {
    flex: 1;
    max-width: 230px;
  }
  .toast {
    --col: max(min(460px, 50%), 340px);
    position: absolute;
    left: calc(var(--col) + 28px);
    bottom: 8px;
    width: min(400px, calc(100% - var(--col) - 28px));
    box-sizing: border-box;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 18px 12px 12px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(38, 40, 48, 0.62);
    backdrop-filter: blur(18px) saturate(140%);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      0 18px 40px rgba(0, 0, 0, 0.4);
    animation: rise var(--t-slow) var(--ease-out) both;
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 8px;
    }
  }
  .toast :global(.bi) {
    border-radius: 12px;
  }
  .tt {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .tp {
    font-size: 17px;
    line-height: 22px;
    font-weight: 700;
    letter-spacing: -0.01em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ts {
    font-size: 13px;
    color: var(--text-2);
  }
  .ts.error {
    color: var(--err);
  }
  .ts.no_internet {
    color: var(--warn);
  }
  @media (max-width: 900px) {
    .stage {
      right: -40%;
      opacity: 0.6;
    }
    .toast {
      display: none;
    }
  }
</style>
