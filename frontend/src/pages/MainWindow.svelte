<script lang="ts">
  // §3.4 Основное окно: history + control panel on the left, the orb on the right.
  import HistoryList from "../components/HistoryList.svelte";
  import Orb from "../components/Orb.svelte";
  import Toggle from "../components/Toggle.svelte";
  import Slider from "../components/Slider.svelte";
  import Icon from "../components/Icon.svelte";
  import { app, changeVolume, toggleOnTop, togglePrefix, toggleSilent } from "../lib/app.svelte";
  import { t } from "../lib/i18n";

  const level = $derived(Math.max(app.micLevel, app.ttsLevel));
  const last = $derived(app.history[0]);
</script>

<div class="main">
  <section class="left">
    <div class="glass card history">
      <header>
        <h2 class="card-title">{t("main.history")}</h2>
        <span class="count">{app.history.length}</span>
      </header>
      <div class="scroll">
        <HistoryList items={app.history} />
      </div>
    </div>

    <div class="glass card panel">
      <h2 class="card-title">{t("main.panel")}</h2>
      <div class="rows">
        <div class="row">
          <span>{t("panel.prefix")}</span>
          <Toggle label={t("panel.prefix")} checked={app.prefixMode} onchange={togglePrefix} />
        </div>
        <div class="row">
          <span>{t("panel.silent")}</span>
          <Toggle label={t("panel.silent")} checked={app.silentMode} onchange={toggleSilent} />
        </div>
        <div class="row">
          <span>{t("panel.avatar")}</span>
          <Toggle label={t("panel.avatar")} checked={app.avatar} onchange={(v) => (app.avatar = v)} />
        </div>
        <div class="row">
          <span>{t("panel.onTop")}</span>
          <Toggle label={t("panel.onTop")} checked={app.onTop} onchange={toggleOnTop} />
        </div>
        <div class="row vol">
          <span><Icon name="volume" size={16} /> {t("panel.volume")}</span>
          <Slider label={t("panel.volume")} value={app.volume} oninput={changeVolume} />
        </div>
      </div>
    </div>
  </section>

  <section class="stage">
    <div class="orb-wrap">
      <Orb {level} active={app.state === "listening" || app.state === "speaking"} />
    </div>
    {#if last}
      {#key last.id}
        <div class="outcome glass">
          <span class="badge {last.status}">
            <Icon
              name={last.status === "done" ? "check" : last.status === "no_internet" ? "wifiOff" : last.status === "cancelled" ? "ban" : "alert"}
              size={20}
              stroke={2.25} />
          </span>
          <span class="txt">
            <span class="ph">{last.phrase}</span>
            <span class="st">{t(`status.${last.status}`)}</span>
          </span>
        </div>
      {/key}
    {/if}
  </section>
</div>

<style>
  .main {
    height: 100%;
    display: grid;
    grid-template-columns: minmax(300px, 380px) 1fr;
    gap: 20px;
    min-height: 0;
  }
  .left {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
  }
  .card {
    padding: 16px;
  }
  .history {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    padding-right: 8px;
  }
  .history header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-right: 8px;
    margin-bottom: 10px;
  }
  .count {
    font-size: 12px;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }
  .scroll {
    overflow-y: auto;
    min-height: 0;
    padding-right: 4px;
    mask-image: linear-gradient(to bottom, #000 calc(100% - 24px), transparent);
  }
  .panel .rows {
    margin-top: 10px;
    display: flex;
    flex-direction: column;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    min-height: 38px;
    font-size: 13.5px;
    color: var(--text-2);
  }
  .row + .row {
    border-top: 1px solid var(--line);
  }
  .row.vol {
    display: grid;
    grid-template-columns: auto 1fr;
  }
  .row.vol > span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .stage {
    position: relative;
    min-width: 0;
    min-height: 0;
    display: grid;
    place-items: center;
  }
  .orb-wrap {
    position: absolute;
    inset: -40px -40px -20px -20px;
  }
  .outcome {
    position: absolute;
    left: 50%;
    bottom: 8px;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 22px 12px 12px;
    min-width: 280px;
    max-width: calc(100% - 32px);
    border-radius: var(--r-xl);
    box-shadow: var(--shadow-lg);
    transition: opacity 260ms var(--ease-out), transform 260ms var(--ease-out), filter 260ms var(--ease-out);
    @starting-style {
      opacity: 0;
      transform: translate(-50%, 10px) scale(0.97);
      filter: blur(4px);
    }
  }
  .badge {
    width: 44px;
    height: 44px;
    border-radius: 14px;
    display: grid;
    place-items: center;
    flex: none;
    background: rgba(52, 211, 153, 0.14);
    color: var(--ok);
  }
  .badge.error {
    background: rgba(248, 113, 113, 0.14);
    color: var(--err);
  }
  .badge.no_internet {
    background: rgba(251, 191, 36, 0.14);
    color: var(--warn);
  }
  .badge.cancelled {
    background: rgba(148, 163, 184, 0.14);
    color: var(--muted);
  }
  .txt {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .ph {
    font: 600 17px/1.25 var(--font-display);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .st {
    font-size: 13px;
    color: var(--text-2);
  }
</style>
