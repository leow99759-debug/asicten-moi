<script lang="ts">
  // «Дополнения» (SPEC §9): Паки / Команды / Озвучка, search, «Только установленные», sort,
  // «Установлено: N/M», categories on the right. Local catalog, works offline.
  import { onMount } from "svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import Tabs from "../components/Tabs.svelte";
  import SearchField from "../components/SearchField.svelte";
  import Checkbox from "../components/Checkbox.svelte";
  import Select from "../components/Select.svelte";
  import PackCard from "../components/PackCard.svelte";
  import BrandIcon from "../components/BrandIcon.svelte";
  import Icon, { type IconName } from "../components/Icon.svelte";
  import * as A from "../lib/addons";
  import { ad, load, toggle } from "../lib/addons.svelte";
  import { cfg, saved } from "../lib/settings.svelte";
  import { app } from "../lib/app.svelte";
  import type { VoiceEngine } from "../lib/bindings/VoiceEngine";
  import { t } from "../lib/i18n";

  const PAGE = 120;
  let shown = $state(PAGE);

  onMount(load);

  const f = $derived({ q: ad.q, cat: ad.cat, only: ad.only });
  const packs = $derived(A.sortPacks(A.filterPacks(ad.list, f), ad.sort));
  const rows = $derived(A.commandRows(A.sortPacks(ad.list, ad.sort), f));
  const tot = $derived(A.totals(ad.list));
  $effect(() => {
    void f;
    shown = PAGE;
  });

  const forms = (key: string) => t(key).split("|") as [string, string, string];
  const catLabel = (c: string) => (c === A.ALL || c === A.POPULAR || c === A.DEFAULT ? t(`addons.cat.${c}`) : c);
  const catIcon: Record<string, IconName> = {
    [A.ALL]: "list",
    [A.POPULAR]: "sparkles",
    Система: "monitor",
    Браузеры: "globe",
    "Дизайн и творчество": "palette",
    "Видео, музыка и стриминг": "film",
    "Работа и разработка": "terminal",
    "Игровые сервисы": "gamepad",
    [A.DEFAULT]: "bolt",
  };

  const voices: { id: string; icon: string; title: string; sub: string; soon?: boolean }[] = [
    { id: "jarvis", icon: "i:sparkles", title: t("voice.jarvis"), sub: t("voice.jarvis.sub") },
    { id: "film", icon: "i:film", title: t("voice.film"), sub: t("addons.voice.jarvis") },
    { id: "windows", icon: "windows", title: t("voice.windows"), sub: t("voice.windows.sub") },
    { id: "fish", icon: "i:globe", title: "Fish Audio", sub: t("voice.online.sub"), soon: true },
    { id: "openai", icon: "i:globe", title: "OpenAI", sub: t("voice.online.sub"), soon: true },
  ];
  const pickVoice = (id: string) => {
    cfg.value.voice_engine = id as VoiceEngine;
    saved();
  };
</script>

<div class="addons">
  <PageHeader
    title={t("title.addons")}
    subtitle={`${A.plural(tot.installed, forms("addons.pack_forms"))} · ${A.plural(tot.commands, forms("addons.cmd_forms"))}`}>
    {#snippet actions()}
      <span class="count" aria-live="polite">{t("addons.installed_n")} <b>{tot.installed}/{tot.total}</b></span>
    {/snippet}
  </PageHeader>

  <Tabs
    tabs={[
      { id: "packs", label: t("addons.tab.packs") },
      { id: "commands", label: t("addons.tab.commands") },
      { id: "voice", label: t("addons.tab.voice") },
    ]}
    value={ad.tab}
    onchange={(id) => (ad.tab = id)} />

  {#if ad.error}
    <p class="error" role="alert"><Icon name="alert" size={16} /> {ad.error}</p>
  {/if}

  {#if ad.tab === "voice"}
    <div class="grid">
      {#each voices as v, i (v.id)}
        {@const on = cfg.value.voice_engine === v.id}
        <article class="vc card" class:sel={on} style="--i: {i}">
          <div class="top">
            <BrandIcon icon={v.icon} size={44} />
            <div class="tx">
              <h3>{v.title}</h3>
              <span class="meta">{v.soon ? t("soon.badge") : t("addons.voice.offline")}</span>
            </div>
          </div>
          <p class="desc">{v.sub}</p>
          <div class="foot">
            {#if v.soon}
              <span class="badge">{t("addons.voice.soon")}</span>
            {:else if on}
              <span class="badge ok"><Icon name="check" size={12} stroke={2.5} /> {t("addons.voice.active")}</span>
            {:else}
              <button type="button" class="btn" onclick={() => pickVoice(v.id)}>{t("addons.voice.pick")}</button>
            {/if}
          </div>
        </article>
      {/each}
    </div>
    <button type="button" class="btn subtle more" onclick={() => (app.page = "settings")}>
      <Icon name="settings" size={14} /> {t("addons.voice.settings")}
    </button>
  {:else}
    <div class="toolbar">
      <SearchField bind:value={ad.q} placeholder={t(ad.tab === "packs" ? "addons.search" : "addons.search_cmd")} />
      <Checkbox checked={ad.only} label={t("addons.only")} onchange={(v) => (ad.only = v)} />
      <span class="grow"></span>
      <Select
        label={t("addons.sort")}
        value={ad.sort}
        options={[
          { value: "popular", label: t("addons.sort.popular") },
          { value: "name", label: t("addons.sort.name") },
          { value: "size", label: t("addons.sort.size") },
        ]}
        onchange={(v) => (ad.sort = v as A.Sort)} />
    </div>

    <div class="body">
      <div class="main">
        {#if ad.tab === "packs"}
          {#if packs.length}
            <div class="grid">
              {#each packs as a, i (a.pack.id)}
                <PackCard addon={a} {i} busy={ad.busy[a.pack.id]} ontoggle={(on) => toggle(a.pack.id, on)} />
              {/each}
            </div>
          {:else}
            <p class="empty">{t("addons.empty")}</p>
          {/if}
        {:else if rows.length}
          <div class="list card">
            {#each rows.slice(0, shown) as r (r.addon.pack.id + r.cmd.id)}
              <div class="row">
                <BrandIcon icon={r.addon.pack.icon} color={r.addon.pack.color} name={r.addon.pack.name} size={28} />
                <div class="rt">
                  <span class="rn">{r.cmd.name}</span>
                  <span class="rp">«{r.cmd.phrases.slice(0, 3).join("», «")}»</span>
                </div>
                <span class="rk">{r.addon.pack.name}</span>
                <span class="dot" class:on={r.addon.installed} title={r.addon.installed ? t("addons.installed") : t("addons.not_installed")}></span>
              </div>
            {/each}
          </div>
          {#if rows.length > shown}
            <button type="button" class="btn subtle more" onclick={() => (shown += PAGE)}>
              {t("addons.more").replace("{n}", String(rows.length - shown))}
            </button>
          {/if}
        {:else}
          <p class="empty">{t("addons.empty")}</p>
        {/if}
      </div>

      <nav class="cats" aria-label={t("addons.cats")}>
        <h2 class="t-group">{t("addons.cats")}</h2>
        {#each A.CATEGORIES as c (c)}
          <button type="button" class="cat" class:on={ad.cat === c} aria-current={ad.cat === c} onclick={() => (ad.cat = c)}>
            <Icon name={catIcon[c]} size={16} />
            <span class="cl">{catLabel(c)}</span>
            <span class="cn">{A.categoryCount(ad.list, c)}</span>
          </button>
        {/each}
      </nav>
    </div>
  {/if}
</div>

<style>
  .addons {
    max-width: 1180px;
    margin: 0 auto;
  }
  .count {
    font-size: 13px;
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }
  .count b {
    color: var(--text);
    font-weight: 650;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
  }
  .toolbar :global(.search) {
    flex: 0 1 320px;
  }
  .grow {
    flex: 1;
  }
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 236px;
    gap: 24px;
    align-items: start;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 10px;
  }
  /* video30: categories = stacked pills in a bordered glass panel, selected is blue */
  .cats {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--stroke);
    border-radius: 12px;
    background: var(--bg-card);
    box-shadow: var(--shadow-card);
  }
  .cats .t-group {
    margin: 4px 0 4px 8px;
  }
  .cat {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 34px;
    padding: 7px 12px;
    line-height: 18px;
    border: 1px solid var(--stroke);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.03);
    color: var(--text-2);
    font-size: 12.5px;
    font-weight: 550;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease;
  }
  .cat:hover {
    background: var(--fill-hover);
    color: var(--text);
  }
  .cat.on {
    background: var(--accent);
    border-color: rgba(255, 255, 255, 0.16);
    color: #fff;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.18);
  }
  .cat.on .cn {
    color: rgba(255, 255, 255, 0.8);
  }
  .cl {
    flex: 1;
    min-width: 0;
  }
  .cn {
    font-size: 11.5px;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }
  .list {
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 48px;
    padding: 0 14px;
    border-top: 1px solid var(--divider);
    content-visibility: auto;
    contain-intrinsic-size: auto 48px;
  }
  .row:first-child {
    border-top: 0;
  }
  .row:hover {
    background: var(--fill-press);
  }
  .rt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .rn {
    font-size: 13.5px;
    font-weight: 600;
    line-height: 18px;
  }
  .rp,
  .rk {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .rk {
    flex: 0 1 200px;
    color: var(--text-3);
    text-align: right;
  }
  .dot {
    width: 8px;
    height: 8px;
    flex: none;
    border-radius: 50%;
    background: var(--stroke-strong);
  }
  .dot.on {
    background: var(--ok);
  }
  .more {
    margin: 10px auto 0;
    display: flex;
    color: var(--accent-text);
  }
  .empty {
    padding: 48px 0;
    text-align: center;
    color: var(--text-3);
  }
  .error {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 12px;
    color: var(--err);
  }
  /* Озвучка cards share PackCard's shape */
  .vc {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    animation: rise var(--t-slow) var(--ease-out) both;
    animation-delay: calc(var(--i) * 24ms);
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 6px;
    }
  }
  .vc.sel {
    border-color: color-mix(in srgb, var(--accent) 45%, transparent);
    background: color-mix(in srgb, var(--accent) 7%, var(--bg-card));
  }
  .top {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .tx {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  h3 {
    margin: 0;
    font-size: 14.5px;
    line-height: 20px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .meta {
    font-size: 12px;
    line-height: 16px;
    color: var(--text-3);
    font-weight: 500;
  }
  .desc {
    margin: 0;
    flex: 1;
    min-height: 36px;
    font-size: 12.5px;
    line-height: 18px;
    color: var(--text-2);
  }
  .foot {
    display: flex;
    justify-content: flex-end;
  }
  .foot .btn {
    height: 30px;
    padding: 0 12px;
    font-size: 12.5px;
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    font-size: 12.5px;
    font-weight: 550;
    color: var(--text-3);
  }
  .badge.ok {
    color: var(--ok);
  }
  @media (max-width: 1099px) {
    .body {
      grid-template-columns: minmax(0, 1fr) 200px;
      gap: 16px;
    }
    .rk {
      display: none;
    }
  }
</style>
