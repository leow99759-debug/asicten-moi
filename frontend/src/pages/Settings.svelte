<script lang="ts">
  // §10.4 Настройки + §6.4 Синтез речи (tab «Голос»). Layout: SYSTEM.md grouped lists.
  import Tabs from "../components/Tabs.svelte";
  import Group from "../components/Group.svelte";
  import SettingRow from "../components/SettingRow.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import Toggle from "../components/Toggle.svelte";
  import Slider from "../components/Slider.svelte";
  import Select from "../components/Select.svelte";
  import TextField from "../components/TextField.svelte";
  import Logo from "../components/Logo.svelte";
  import Icon, { type IconName } from "../components/Icon.svelte";
  import { cfg, loadConfig, saved } from "../lib/settings.svelte";
  import { app, SWATCHES } from "../lib/app.svelte";
  import { avatarEdit, classroomConnect, classroomDisconnect, hudPreview, micDevices, previewVoice, setConfig } from "../lib/commands";
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
    { id: "ai", label: t("set.tab.ai") },
    { id: "hotkeys", label: t("set.tab.hotkeys") },
    { id: "about", label: t("set.tab.about") },
  ];

  const engines: { id: VoiceEngine | "fish" | "eleven" | "openai"; icon: IconName; title: string; sub: string; soon?: boolean }[] = [
    { id: "jarvis", icon: "sparkles", title: t("voice.jarvis"), sub: t("voice.jarvis.sub") },
    { id: "windows", icon: "monitor", title: t("voice.windows"), sub: t("voice.windows.sub") },
    { id: "eleven", icon: "globe", title: "ElevenLabs", sub: t("voice.fish.sub") },
    { id: "fish", icon: "globe", title: "Fish Audio", sub: t("voice.fish.sub") },
    { id: "openai", icon: "globe", title: "OpenAI", sub: t("voice.online.sub"), soon: true },
  ];

  function setKey(i: number, v: string) {
    set(() => {
      const keys = [...c.online.gemini_keys];
      while (keys.length < 2) keys.push("");
      keys[i] = v;
      c.online.gemini_keys = keys;
    });
  }

  let gc = $state<{ busy: boolean; error: string }>({ busy: false, error: "" });

  async function connectClassroom() {
    gc = { busy: true, error: "" };
    try {
      await setConfig($state.snapshot(cfg.value));
      await classroomConnect();
      await loadConfig();
      gc = { busy: false, error: "" };
    } catch (e) {
      gc = { busy: false, error: String(e) };
    }
  }

  async function disconnectClassroom() {
    await classroomDisconnect();
    await loadConfig();
  }

  function set(fn: () => void) {
    fn();
    saved();
  }

  function preview() {
    previewing = true;
    previewVoice();
    setTimeout(() => (previewing = false), 3200);
  }

  const WAVE = 56;
</script>

<div class="settings">
  <PageHeader title={t("title.settings")} subtitle={t("set.subtitle")} />
  <Tabs {tabs} value={tab} onchange={(id) => (tab = id)} />

  {#key tab}
    <div class="panel">
      {#if tab === "general"}
        <Group title={t("set.activation")}>
          <SettingRow icon="mic" label={t("set.sensitivity")} desc={t("set.sensitivity.sub")} wide>
            <Slider label={t("set.sensitivity")} value={c.wake_sensitivity} oninput={(v) => set(() => (c.wake_sensitivity = v))} />
          </SettingRow>
          <SettingRow icon="wave" label={t("set.mic")} desc={t("set.mic.sub")}>
            <Select
              label={t("set.mic")}
              value={c.mic_device ?? ""}
              options={[{ value: "", label: t("set.mic.default") }, ...mics.map((m) => ({ value: m, label: m }))]}
              onchange={(v) => set(() => (c.mic_device = v || null))} />
          </SettingRow>
        </Group>
        <Group title={t("set.modes")}>
          {#each ["prefix_on", "prefix_off", "silent_on", "silent_off", "mic_off"] as const as k (k)}
            <SettingRow label={t(`set.phrase.${k}`)} wide>
              <TextField label={t(`set.phrase.${k}`)} value={c.mode_phrases[k]} onchange={(v) => set(() => (c.mode_phrases[k] = v))} />
            </SettingRow>
          {/each}
        </Group>
        <Group title={t("set.system")}>
          <SettingRow icon="minus" label={t("set.tray")} desc={t("set.tray.sub")}>
            <Toggle label={t("set.tray")} checked={c.close_to_tray} onchange={(v) => set(() => (c.close_to_tray = v))} />
          </SettingRow>
          <SettingRow icon="bolt" label={t("set.autostart")} desc={t("set.autostart.sub")}>
            <Toggle label={t("set.autostart")} checked={c.autostart} onchange={(v) => set(() => (c.autostart = v))} />
          </SettingRow>
          <SettingRow icon="leaf" label={t("set.memory")} desc={t("set.memory.sub")}>
            <Toggle label={t("set.memory")} checked={c.memory_saver} onchange={(v) => set(() => (c.memory_saver = v))} />
          </SettingRow>
        </Group>
      {:else if tab === "ui"}
        <Group title={t("set.look")}>
          <SettingRow icon="palette" label={t("set.accent")}>
            <div class="swatches" role="radiogroup" aria-label={t("set.accent")}>
              {#each Object.entries(SWATCHES) as [name, hex] (name)}
                <button
                  type="button"
                  role="radio"
                  aria-checked={c.ui.accent === hex}
                  aria-label={name}
                  title={name}
                  class="sw"
                  class:on={c.ui.accent === hex}
                  style="--c: {hex}"
                  onclick={() => set(() => (c.ui.accent = hex))}>
                  <Icon name="check" size={12} stroke={3} />
                </button>
              {/each}
              <label class="sw custom" title={t("set.accent.custom")}>
                <input type="color" value={c.ui.accent} oninput={(e) => set(() => (c.ui.accent = e.currentTarget.value))} />
                <span>+</span>
              </label>
            </div>
          </SettingRow>
          <SettingRow icon="eye" label={t("set.transparency")} desc={t("set.transparency.sub")} wide>
            <Slider label={t("set.transparency")} value={c.ui.transparency} oninput={(v) => set(() => (c.ui.transparency = v))} />
          </SettingRow>
          <SettingRow icon="drop" label={t("set.blur")} wide>
            <Slider label={t("set.blur")} value={c.ui.blur} oninput={(v) => set(() => (c.ui.blur = v))} />
          </SettingRow>
          <SettingRow icon="sparkles" label={t("set.animations")} desc={t("set.animations.sub")}>
            <Toggle label={t("set.animations")} checked={c.ui.animations} onchange={(v) => set(() => (c.ui.animations = v))} />
          </SettingRow>
        </Group>
        <Group title={t("set.overlay")}>
          <SettingRow icon="person" label={t("panel.avatar")} desc={t("set.avatar.sub")}>
            {#if c.ui.avatar}
              <button type="button" class="btn" onclick={() => avatarEdit(true)}><Icon name="move" size={14} /> {t("avatar.move")}</button>
            {/if}
            <Toggle label={t("panel.avatar")} checked={c.ui.avatar} onchange={(v) => set(() => (c.ui.avatar = v))} />
          </SettingRow>
          <SettingRow icon="mic" label={t("set.pill")} desc={t("set.pill.sub")}>
            <Toggle label={t("set.pill")} checked={c.ui.pill} onchange={(v) => set(() => (c.ui.pill = v))} />
          </SettingRow>
          <SettingRow icon="target" label={t("set.hud")} desc={t("set.hud.sub")}>
            {#if c.ui.hud}
              <button type="button" class="btn" onclick={() => hudPreview()}>{t("hud.preview")}</button>
            {/if}
            <Toggle label={t("set.hud")} checked={c.ui.hud} onchange={(v) => set(() => (c.ui.hud = v))} />
          </SettingRow>
          <SettingRow icon="pin" label={t("panel.onTop")} desc={t("set.ontop.sub")}>
            <Toggle label={t("panel.onTop")} checked={c.ui.on_top} onchange={(v) => set(() => (c.ui.on_top = v))} />
          </SettingRow>
        </Group>
        <Group title={t("set.region")}>
          <SettingRow icon="globe" label={t("set.lang")}>
            <Select label={t("set.lang")} value="ru" options={[{ value: "ru", label: "Русский" }]} disabled />
          </SettingRow>
        </Group>
      {:else if tab === "voice"}
        <h2 class="t-group">{t("voice.title")}</h2>
        <div class="engines" role="radiogroup" aria-label={t("voice.title")}>
          {#each engines as e (e.id)}
            <button
              type="button"
              role="radio"
              aria-checked={c.voice_engine === e.id}
              disabled={e.soon}
              class="engine card"
              class:on={c.voice_engine === e.id}
              onclick={() => (e.id === "fish" || e.id === "eleven" ? (tab = "ai") : set(() => (c.voice_engine = e.id as VoiceEngine)))}>
              <span class="eic"><Icon name={e.icon} size={18} /></span>
              <span class="etx">
                <span class="et">{e.title}{#if e.soon}<span class="soon">{t("soon.badge")}</span>{/if}</span>
                <span class="es">{e.sub}</span>
              </span>
              <span class="radio" aria-hidden="true"></span>
            </button>
          {/each}
        </div>

        <div class="card preview">
          <div class="wave" class:live={previewing} aria-hidden="true">
            {#each Array.from({ length: WAVE }) as _, i (i)}
              <span style="--i: {i}; --h: {0.22 + 0.78 * Math.abs(Math.sin(i * 0.5) * Math.cos(i * 0.17))}"></span>
            {/each}
          </div>
          <div class="pv">
            <div>
              <p class="t-subtitle">{t("voice.sample")}</p>
              <p class="t-caption">«{t("voice.sample.text")}»</p>
            </div>
            <button type="button" class="btn primary" onclick={preview}>
              <Icon name="play" size={14} /> {t("voice.preview")}
            </button>
          </div>
        </div>

        <Group title={t("voice.sound")}>
          <SettingRow icon="bolt" label={t("voice.speed")} wide>
            <Slider
              label={t("voice.speed")}
              min={50}
              max={200}
              suffix="%"
              value={Math.round(c.voice_speed * 100)}
              oninput={(v) => set(() => (c.voice_speed = v / 100))} />
          </SettingRow>
          <SettingRow icon="volume" label={t("panel.volume")} wide>
            <Slider label={t("panel.volume")} value={c.voice_volume} oninput={(v) => set(() => (c.voice_volume = v))} />
          </SettingRow>
          <SettingRow icon="film" label={t("voice.fx")} desc={t("voice.fx.sub")}>
            <Toggle label={t("voice.fx")} checked={c.voice_fx} disabled={c.voice_engine !== "jarvis"} onchange={(v) => set(() => (c.voice_fx = v))} />
          </SettingRow>
        </Group>
        {#if c.voice_engine === "windows"}
          <p class="note"><Icon name="info" size={14} /> {t("voice.windows.note")}</p>
        {/if}
      {:else if tab === "ai"}
        <Group title={t("ai.news")}>
          {#each [0, 1] as i (i)}
            <SettingRow icon="sparkles" label={t(`ai.gemini${i + 1}`)} desc={i === 0 ? t("ai.gemini.sub") : t("ai.gemini2.sub")} wide>
              <TextField secret label={t(`ai.gemini${i + 1}`)} placeholder="AIza…" value={c.online.gemini_keys[i] ?? ""} onchange={(v) => setKey(i, v)} />
            </SettingRow>
          {/each}
          <SettingRow icon="settings" label={t("ai.model")} desc={t("ai.model.sub")} wide>
            <TextField label={t("ai.model")} value={c.online.gemini_model} onchange={(v) => set(() => (c.online.gemini_model = v))} />
          </SettingRow>
        </Group>
        <Group title={t("ai.voice")}>
          <SettingRow icon="volume" label={t("ai.eleven")} desc={t("ai.eleven.sub")} wide>
            <TextField secret label={t("ai.eleven")} value={c.online.eleven_key} onchange={(v) => set(() => (c.online.eleven_key = v))} />
          </SettingRow>
          <SettingRow icon="person" label={t("ai.eleven.voice")} desc={t("ai.eleven.voice.sub")} wide>
            <TextField label={t("ai.eleven.voice")} value={c.online.eleven_voice} onchange={(v) => set(() => (c.online.eleven_voice = v))} />
          </SettingRow>
          <SettingRow icon="volume" label={t("ai.fish")} desc={t("ai.fish.sub")} wide>
            <TextField secret label={t("ai.fish")} value={c.online.fish_key} onchange={(v) => set(() => (c.online.fish_key = v))} />
          </SettingRow>
          <SettingRow icon="person" label={t("ai.fish.voice")} wide>
            <TextField label={t("ai.fish.voice")} value={c.online.fish_voice} onchange={(v) => set(() => (c.online.fish_voice = v))} />
          </SettingRow>
        </Group>
        <Group title={t("gc.title")}>
          <SettingRow icon="user" label={t("gc.id")} desc={t("gc.id.sub")} wide>
            <TextField label={t("gc.id")} placeholder="….apps.googleusercontent.com" value={c.online.classroom_id} onchange={(v) => set(() => (c.online.classroom_id = v))} />
          </SettingRow>
          <SettingRow icon="eye" label={t("gc.secret")} wide>
            <TextField secret label={t("gc.secret")} placeholder="GOCSPX-…" value={c.online.classroom_secret} onchange={(v) => set(() => (c.online.classroom_secret = v))} />
          </SettingRow>
          <SettingRow
            icon={c.online.classroom_token ? "check" : "globe"}
            label={c.online.classroom_token ? t("gc.on") : gc.busy ? t("gc.wait") : t("gc.off")}
            desc={gc.error || t("gc.sub")}>
            {#if c.online.classroom_token}
              <button type="button" class="btn" onclick={disconnectClassroom}>{t("gc.disconnect")}</button>
            {:else}
              <button
                type="button"
                class="btn primary"
                disabled={gc.busy || !c.online.classroom_id.trim() || !c.online.classroom_secret.trim()}
                onclick={connectClassroom}>{t("gc.connect")}</button>
            {/if}
          </SettingRow>
        </Group>
        <p class="note"><Icon name="info" size={14} /> {t("ai.note")}</p>
      {:else if tab === "hotkeys"}
        <Group title={t("set.hotkeys.sub")}>
          {#each ["push_to_talk", "toggle_window", "toggle_mic"] as const as k (k)}
            <SettingRow icon="keyboard" label={t(`hk.${k}`)} wide>
              <TextField label={t(`hk.${k}`)} value={c.hotkeys[k]} onchange={(v) => set(() => (c.hotkeys[k] = v))} />
            </SettingRow>
          {/each}
        </Group>
        <p class="note"><Icon name="info" size={14} /> {t("set.hotkeys.restart")}</p>
      {:else}
        <div class="card about">
          <Logo size={56} />
          <div>
            <p class="t-title">{t("app.name")}</p>
            <p class="t-caption num">{t("about.version")} 0.1.0 · {app.commandCount} {t("tile.commands")}</p>
          </div>
        </div>
        <Group title={t("about.thanks")}>
          {#each t("about.credits").split("\n") as line (line)}
            <SettingRow label={line}>{""}</SettingRow>
          {/each}
        </Group>
      {/if}
    </div>
  {/key}
</div>

<style>
  .settings {
    max-width: 820px;
    margin: 0;
    position: relative;
  }
  .panel {
    transition:
      opacity var(--t-base) var(--ease-out),
      translate var(--t-base) var(--ease-out);
    @starting-style {
      opacity: 0;
      translate: 0 4px;
    }
  }
  .swatches {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .sw {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 0;
    padding: 0;
    display: grid;
    place-items: center;
    background: var(--c);
    color: transparent;
    cursor: pointer;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.12);
    transition:
      transform var(--t-fast) var(--ease-out),
      color var(--t-fast) ease;
  }
  .sw:hover {
    transform: scale(1.1);
  }
  .sw:active {
    transform: scale(0.94);
  }
  .sw.on {
    color: #fff;
    box-shadow: 0 0 0 2px var(--bg-card), 0 0 0 3.5px var(--c);
  }
  .sw.on[style*="#f1f5f9"] {
    color: #111;
  }
  .sw.custom {
    position: relative;
    background: var(--fill);
    border: 1px dashed var(--stroke-strong);
    box-sizing: border-box;
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
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-bottom: 12px;
  }
  .engine {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    text-align: left;
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      border-color var(--t-base) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .engine:hover:not(:disabled) {
    background: var(--bg-card-hover);
  }
  .engine:active:not(:disabled) {
    transform: scale(0.985);
  }
  .engine.on {
    border-color: rgba(var(--accent-rgb), 0.6);
    background: color-mix(in srgb, var(--bg-card) 90%, var(--accent));
  }
  .engine:disabled {
    cursor: default;
  }
  .engine:disabled .eic,
  .engine:disabled .etx {
    opacity: 0.5;
  }
  .eic {
    width: 36px;
    height: 36px;
    display: grid;
    place-items: center;
    border-radius: var(--r-md);
    background: var(--fill);
    color: var(--text-2);
    flex: none;
  }
  .engine.on .eic {
    background: var(--accent);
    color: var(--on-accent);
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
    line-height: 16px;
    color: var(--text-2);
  }
  .soon {
    font-size: 10.5px;
    line-height: 16px;
    font-weight: 600;
    padding: 0 6px;
    border-radius: var(--r-xs);
    background: var(--fill);
    color: var(--text-2);
  }
  .radio {
    width: 18px;
    height: 18px;
    box-sizing: border-box;
    border-radius: 50%;
    border: 1.5px solid rgba(255, 255, 255, 0.45);
    flex: none;
    transition:
      border-width var(--t-base) var(--spring),
      border-color var(--t-base) ease;
  }
  .engine.on .radio {
    border: 5px solid var(--accent);
  }

  .preview {
    padding: 16px;
    margin-bottom: 24px;
  }
  .pv {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .pv p {
    margin: 0;
  }
  .wave {
    height: 48px;
    display: flex;
    align-items: center;
    gap: 3px;
    margin-bottom: 14px;
  }
  .wave span {
    flex: 1;
    height: 100%;
    border-radius: 2px;
    background: rgba(var(--accent-rgb), 0.35);
    transform: scaleY(calc(var(--h) * 0.4));
    transition:
      transform 400ms var(--ease-out),
      background-color var(--t-base) ease;
  }
  .wave.live span {
    animation: wave 700ms ease-in-out infinite alternate;
    animation-delay: calc(var(--i) * -37ms);
    background: var(--accent);
  }
  @keyframes wave {
    from {
      transform: scaleY(calc(var(--h) * 0.2));
    }
    to {
      transform: scaleY(var(--h));
    }
  }
  .note {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    margin: -12px 2px 24px;
    font-size: 12px;
    line-height: 16px;
    color: var(--text-2);
  }
  .note :global(svg) {
    flex: none;
    margin-top: 1px;
  }
  .about {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 20px;
    margin-bottom: 24px;
  }
  .about p {
    margin: 0;
  }
</style>
