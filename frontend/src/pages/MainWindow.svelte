<script lang="ts">
  // §3.4 Голосовой центр: quick toggles, volume and history on the left, the orb stage right.
  import HistoryList from "../components/HistoryList.svelte";
  import Orb from "../components/Orb.svelte";
  import Slider from "../components/Slider.svelte";
  import Icon from "../components/Icon.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import QuickTile from "../components/QuickTile.svelte";
  import { app, togglePrefix, toggleSilent } from "../lib/app.svelte";
  import { activate } from "../lib/commands";
  import { cfg, saved } from "../lib/settings.svelte";
  import { t } from "../lib/i18n";

  const level = $derived(Math.max(app.micLevel, app.ttsLevel));
  const active = $derived(app.state === "listening" || app.state === "speaking" || app.state === "processing");
  const last = $derived(app.history[0]);
</script>

<PageHeader title={t("title.main")} subtitle={t("main.subtitle")}>
  {#snippet actions()}
    <button type="button" class="btn primary" onclick={activate}><Icon name="mic" size={15} /> {t("main.listen")}</button>
  {/snippet}
</PageHeader>

<div class="grid">
  <div class="left">
    <div class="tiles">
      <QuickTile icon="wave" label={t("panel.prefix")} checked={app.prefixMode} onchange={togglePrefix} />
      <QuickTile icon="bellOff" label={t("panel.silent")} checked={app.silentMode} onchange={toggleSilent} />
      <QuickTile
        icon="person"
        label={t("panel.avatar")}
        checked={cfg.value.ui.avatar}
        onchange={(v) => {
          cfg.value.ui.avatar = v;
          saved();
        }} />
      <QuickTile
        icon="pin"
        label={t("panel.onTop")}
        checked={cfg.value.ui.on_top}
        onchange={(v) => {
          cfg.value.ui.on_top = v;
          saved();
        }} />
    </div>

    <div class="card vol">
      <Icon name="volume" size={18} />
      <span class="vl">{t("panel.volume")}</span>
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

    <section>
      <div class="gh">
        <h2 class="t-group">{t("main.history")}</h2>
        <span class="t-caption num">{app.history.length}</span>
      </div>
      <div class="card hist">
        <HistoryList items={app.history} />
      </div>
    </section>
  </div>

  <aside class="card stage">
    <div class="orb"><Orb {level} {active} /></div>
    <p class="state" class:on={active}>{t(`state.${app.state}`)}</p>
    {#if last}
      {#key last.id}
        <p class="last">
          <span class="lp">«{last.phrase}»</span>
          <span class="ls {last.status}">{t(`status.${last.status}`)}</span>
        </p>
      {/key}
    {/if}
  </aside>
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, 340px);
    gap: 20px;
    align-items: start;
  }
  .left {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
  }
  .vol {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 52px;
    padding: 0 16px;
    color: var(--text-2);
  }
  .vl {
    color: var(--text);
    font-weight: 500;
    flex: none;
  }
  .vs {
    flex: 1;
    min-width: 0;
  }
  section {
    margin-top: 12px;
  }
  .gh {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-right: 4px;
  }
  .hist {
    overflow: hidden;
  }
  .stage {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px 20px 24px;
    background:
      radial-gradient(120% 70% at 50% 40%, rgba(var(--accent-rgb), 0.1), transparent 70%),
      var(--bg-card);
  }
  .orb {
    width: 100%;
    aspect-ratio: 1;
    max-width: 290px;
  }
  .state {
    margin: 4px 0 0;
    font-size: 15px;
    line-height: 20px;
    font-weight: 650;
    letter-spacing: -0.01em;
    color: var(--text-2);
    transition: color var(--t-base) ease;
  }
  .state.on {
    color: var(--accent-text);
  }
  .last {
    margin: 10px 0 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    text-align: center;
    max-width: 100%;
    animation: fade var(--t-slow) var(--ease-out) both;
  }
  @keyframes fade {
    from {
      opacity: 0;
      translate: 0 4px;
    }
  }
  .lp {
    font-size: 13px;
    color: var(--text);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }
  .ls {
    font-size: 12px;
    color: var(--ok);
    font-weight: 600;
  }
  .ls.error {
    color: var(--err);
  }
  .ls.no_internet {
    color: var(--warn);
  }
  .ls.cancelled {
    color: var(--muted);
  }
  @media (max-width: 1040px) {
    .grid {
      grid-template-columns: 1fr;
    }
    .stage {
      position: static;
      order: -1;
      flex-direction: row;
      gap: 16px;
      padding: 12px 16px;
    }
    .orb {
      width: 96px;
      flex: none;
    }
  }
</style>
