<script lang="ts">
  // §10.4 Настройки + §6.4 Синтез речи (tab «Голос»).
  import Tabs from "../components/Tabs.svelte";
  import SettingRow from "../components/SettingRow.svelte";
  import Toggle from "../components/Toggle.svelte";
  import Slider from "../components/Slider.svelte";
  import Select from "../components/Select.svelte";
  import TextField from "../components/TextField.svelte";
  import Icon, { type IconName } from "../components/Icon.svelte";
  import { cfg, saved } from "../lib/settings.svelte";
  import { app, SWATCHES } from "../lib/app.svelte";
  import { micDevices, previewVoice } from "../lib/commands";
  import type { VoiceEngine } from "../lib/bindings/VoiceEngine";
  import { t } from "../lib/i18n";
  import { onMount } from "svelte";

  let { tab = $bindable("general") }: { tab?: string } = $props();
  const c = $derived(cfg.value);
  let mics = $state<string[]>([]);
  let previewing = $state(false);

  onMount(async () => {
    mics = (await micDevices()) ?? [];
  });

  const tabs = [
    { id: "general", label: t("set.tab.general") },
    { id: "ui", label: t("set.tab.ui") },
    { id: "voice", label: t("set.tab.voice") },
    { id: "hotkeys", label: t("set.tab.hotkeys") },
    { id: "about", label: t("set.tab.about") },
  ];

  const engines: { id: VoiceEngine | "fish" | "openai"; icon: IconName; title: string; sub: string; soon?: boolean }[] = [
    { id: "jarvis", icon: "sparkles", title: t("voice.jarvis"), sub: t("voice.jarvis.sub") },
    { id: "windows", icon: "monitor", title: t("voice.windows"), sub: t("voice.windows.sub") },
    { id: "fish", icon: "globe", title: "Fish Audio", sub: t("voice.online.sub"), soon: true },
    { id: "openai", icon: "globe", title: "OpenAI", sub: t("voice.online.sub"), soon: true },
  ];

  function set(fn: () => void) {
    fn();
    saved();
  }

  function preview() {
    previewing = true;
    previewVoice();
    setTimeout(() => (previewing = false), 3200);
  }

  const WAVE = 44;
</script>

<div class="settings">
  <Tabs {tabs} value={tab} onchange={(id) => (tab = id)} />

  {#key tab}
    <div class="panel">
      {#if tab === "general"}
        <section class="glass group">
          <header><Icon name="mic" size={18} /><div><h2>{t("set.modes")}</h2><p>{t("set.modes.sub")}</p></div></header>
          <SettingRow label={t("set.sensitivity")} desc={t("set.sensitivity.sub")} wide>
            <Slider label={t("set.sensitivity")} value={c.wake_sensitivity} oninput={(v) => set(() => (c.wake_sensitivity = v))} />
          </SettingRow>
          {#each ["prefix_on", "prefix_off", "silent_on", "silent_off", "mic_off"] as const as k (k)}
            <SettingRow label={t(`set.phrase.${k}`)} wide>
              <TextField label={t(`set.phrase.${k}`)} value={c.mode_phrases[k]} onchange={(v) => set(() => (c.mode_phrases[k] = v))} />
            </SettingRow>
          {/each}
        </section>
        <section class="glass group">
          <header><Icon name="monitor" size={18} /><div><h2>{t("set.system")}</h2><p>{t("set.system.sub")}</p></div></header>
          <SettingRow label={t("set.mic")} desc={t("set.mic.sub")}>
            <Select
              label={t("set.mic")}
              value={c.mic_device ?? ""}
              options={[{ value: "", label: t("set.mic.default") }, ...mics.map((m) => ({ value: m, label: m }))]}
              onchange={(v) => set(() => (c.mic_device = v || null))} />
          </SettingRow>
          <SettingRow label={t("set.tray")}>
            <Toggle label={t("set.tray")} checked={c.close_to_tray} onchange={(v) => set(() => (c.close_to_tray = v))} />
          </SettingRow>
          <SettingRow label={t("set.autostart")}>
            <Toggle label={t("set.autostart")} checked={c.autostart} onchange={(v) => set(() => (c.autostart = v))} />
          </SettingRow>
          <SettingRow label={t("set.memory")} desc={t("set.memory.sub")}>
            <Toggle label={t("set.memory")} checked={c.memory_saver} onchange={(v) => set(() => (c.memory_saver = v))} />
          </SettingRow>
        </section>
      {:else if tab === "ui"}
        <section class="glass group">
          <header><Icon name="sparkles" size={18} /><div><h2>{t("set.ui")}</h2><p>{t("set.ui.sub")}</p></div></header>
          <SettingRow label={t("set.accent")}>
            <div class="swatches" role="radiogroup" aria-label={t("set.accent")}>
              {#each Object.entries(SWATCHES) as [name, hex] (name)}
                <button
                  type="button"
                  role="radio"
                  aria-checked={c.ui.accent === hex}
                  aria-label={name}
                  title={name}
                  class="sw pressable"
                  class:on={c.ui.accent === hex}
                  style="--c: {hex}"
                  onclick={() => set(() => (c.ui.accent = hex))}></button>
              {/each}
              <label class="sw custom pressable" title={t("set.accent.custom")}>
                <input type="color" value={c.ui.accent} oninput={(e) => set(() => (c.ui.accent = e.currentTarget.value))} />
                <span>+</span>
              </label>
            </div>
          </SettingRow>
          <SettingRow label={t("set.transparency")} wide>
            <Slider label={t("set.transparency")} value={c.ui.transparency} oninput={(v) => set(() => (c.ui.transparency = v))} />
          </SettingRow>
          <SettingRow label={t("set.blur")} wide>
            <Slider label={t("set.blur")} value={c.ui.blur} oninput={(v) => set(() => (c.ui.blur = v))} />
          </SettingRow>
          <SettingRow label={t("set.animations")}>
            <Toggle label={t("set.animations")} checked={c.ui.animations} onchange={(v) => set(() => (c.ui.animations = v))} />
          </SettingRow>
          <SettingRow label={t("panel.avatar")}>
            <Toggle label={t("panel.avatar")} checked={c.ui.avatar} onchange={(v) => set(() => (c.ui.avatar = v))} />
          </SettingRow>
          <SettingRow label={t("set.hud")} desc={t("set.hud.sub")}>
            <Toggle label={t("set.hud")} checked={c.ui.hud} onchange={(v) => set(() => (c.ui.hud = v))} />
          </SettingRow>
          <SettingRow label={t("panel.onTop")}>
            <Toggle label={t("panel.onTop")} checked={c.ui.on_top} onchange={(v) => set(() => (c.ui.on_top = v))} />
          </SettingRow>
          <SettingRow label={t("set.lang")}>
            <Select label={t("set.lang")} value="ru" options={[{ value: "ru", label: "Русский" }]} disabled />
          </SettingRow>
        </section>
      {:else if tab === "voice"}
        <section class="glass group">
          <header><Icon name="volume" size={18} /><div><h2>{t("voice.title")}</h2><p>{t("voice.sub")}</p></div></header>
          <div class="engines" role="radiogroup" aria-label={t("voice.title")}>
            {#each engines as e (e.id)}
              <button
                type="button"
                role="radio"
                aria-checked={c.voice_engine === e.id}
                disabled={e.soon}
                class="engine pressable"
                class:on={c.voice_engine === e.id}
                onclick={() => set(() => (c.voice_engine = e.id as VoiceEngine))}>
                <span class="eic"><Icon name={e.icon} size={20} /></span>
                <span class="etx">
                  <span class="et">{e.title}{#if e.soon}<span class="soon">{t("soon.badge")}</span>{/if}</span>
                  <span class="es">{e.sub}</span>
                </span>
                <span class="radio"></span>
              </button>
            {/each}
          </div>
          <div class="wave" class:live={previewing} aria-hidden="true">
            {#each Array.from({ length: WAVE }) as _, i (i)}
              <span style="--i: {i}; --h: {0.25 + 0.75 * Math.abs(Math.sin(i * 0.55) * Math.cos(i * 0.21))}"></span>
            {/each}
          </div>
          <SettingRow label={t("voice.speed")} wide>
            <Slider
              label={t("voice.speed")}
              min={50}
              max={200}
              suffix="%"
              value={Math.round(c.voice_speed * 100)}
              oninput={(v) => set(() => (c.voice_speed = v / 100))} />
          </SettingRow>
          <SettingRow label={t("panel.volume")} wide>
            <Slider label={t("panel.volume")} value={c.voice_volume} oninput={(v) => set(() => (c.voice_volume = v))} />
          </SettingRow>
          <SettingRow label={t("voice.fx")} desc={t("voice.fx.sub")}>
            <Toggle label={t("voice.fx")} checked={c.voice_fx} disabled={c.voice_engine !== "jarvis"} onchange={(v) => set(() => (c.voice_fx = v))} />
          </SettingRow>
          {#if c.voice_engine === "windows"}
            <p class="note"><Icon name="alert" size={14} /> {t("voice.windows.note")}</p>
          {/if}
          <div class="actions">
            <button type="button" class="btn primary pressable" onclick={preview}>
              <Icon name="volume" size={16} /> {t("voice.preview")}
            </button>
          </div>
        </section>
      {:else if tab === "hotkeys"}
        <section class="glass group">
          <header><Icon name="terminal" size={18} /><div><h2>{t("set.tab.hotkeys")}</h2><p>{t("set.hotkeys.sub")}</p></div></header>
          {#each ["push_to_talk", "toggle_window", "toggle_mic"] as const as k (k)}
            <SettingRow label={t(`hk.${k}`)} wide>
              <TextField label={t(`hk.${k}`)} value={c.hotkeys[k]} onchange={(v) => set(() => (c.hotkeys[k] = v))} />
            </SettingRow>
          {/each}
          <p class="note"><Icon name="alert" size={14} /> {t("set.hotkeys.restart")}</p>
        </section>
      {:else}
        <section class="glass group about">
          <header><Icon name="user" size={18} /><div><h2>{t("app.name")}</h2><p>{t("about.version")} 0.1.0 · {app.commandCount} {t("tile.commands")}</p></div></header>
          <p class="credits">{t("about.credits")}</p>
        </section>
      {/if}
    </div>
  {/key}
</div>

<style>
  .settings {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
  }
  .panel {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(420px, 1fr));
    align-content: start;
    gap: 16px;
    padding-right: 4px;
    transition: opacity 180ms var(--ease-out), translate 180ms var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 4px;
    }
  }
  .group {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .group header {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    margin-bottom: 6px;
    color: var(--text-2);
  }
  .group header :global(svg) {
    margin-top: 2px;
  }
  h2 {
    margin: 0;
    font: 600 15px/1.3 var(--font);
    color: var(--text);
  }
  header p {
    margin: 2px 0 0;
    font-size: 12px;
    color: var(--text-3);
  }
  .swatches {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .sw {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 0;
    padding: 0;
    background: var(--c);
    cursor: pointer;
    box-shadow: 0 0 0 0 transparent;
    transition: box-shadow var(--t-fast) ease, transform var(--t-press) var(--ease-out);
  }
  .sw.on {
    box-shadow: 0 0 0 2px var(--bg-1), 0 0 0 4px var(--c);
  }
  .sw.custom {
    position: relative;
    display: grid;
    place-items: center;
    background: rgba(255, 255, 255, 0.06);
    border: 1px dashed var(--line-2);
    color: var(--text-2);
    font-size: 15px;
    line-height: 1;
  }
  .sw.custom input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .engines {
    display: grid;
    gap: 8px;
  }
  .engine {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--r-md);
    border: 1px solid var(--line);
    background: rgba(255, 255, 255, 0.025);
    text-align: left;
    cursor: pointer;
    transition:
      transform var(--t-press) var(--ease-out),
      border-color var(--t-fast) ease,
      background-color var(--t-fast) ease;
  }
  .engine:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .engine.on {
    border-color: rgba(var(--accent-rgb), 0.55);
    background: rgba(var(--accent-rgb), 0.08);
  }
  .engine:disabled {
    opacity: 0.55;
    cursor: default;
  }
  .eic {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-2);
    flex: none;
  }
  .engine.on .eic {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .etx {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .et {
    font-weight: 600;
    font-size: 13.5px;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .es {
    font-size: 12px;
    color: var(--text-3);
  }
  .soon {
    font-size: 10.5px;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-2);
  }
  .radio {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: 1.5px solid var(--line-2);
    flex: none;
    transition: border-color var(--t-fast) ease, border-width var(--t-fast) var(--ease-out);
  }
  .engine.on .radio {
    border: 5px solid var(--accent);
  }
  .wave {
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    margin: 6px 0;
  }
  .wave span {
    width: 3px;
    height: 100%;
    border-radius: 2px;
    background: rgba(var(--accent-rgb), 0.55);
    transform: scaleY(calc(var(--h) * 0.35));
    transition: transform 300ms var(--ease-out);
  }
  .wave.live span {
    animation: wave 900ms ease-in-out infinite alternate;
    animation-delay: calc(var(--i) * -41ms);
    background: var(--accent);
  }
  @keyframes wave {
    from {
      transform: scaleY(calc(var(--h) * 0.25));
    }
    to {
      transform: scaleY(var(--h));
    }
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    margin-top: 6px;
  }
  .btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 16px;
    border-radius: var(--r-md);
    border: 1px solid var(--line-2);
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    font-size: 13px;
    cursor: pointer;
  }
  .btn.primary {
    background: var(--accent);
    border-color: transparent;
    color: var(--on-accent);
    box-shadow: 0 6px 18px -6px rgba(var(--accent-rgb), 0.6);
  }
  .note {
    display: flex;
    gap: 6px;
    align-items: flex-start;
    margin: 4px 2px 0;
    font-size: 12px;
    color: var(--text-3);
  }
  .note :global(svg) {
    flex: none;
    margin-top: 1px;
  }
  .credits {
    margin: 0;
    font-size: 13px;
    color: var(--text-2);
    line-height: 1.6;
    white-space: pre-line;
  }
</style>
