<script lang="ts">
  // Desktop avatar (SPEC §3.6): the orb in a screen corner, reacting to the voice.
  // Click-through; in placement mode it becomes draggable with a «Готово» button.
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Orb from "../components/Orb.svelte";
  import Icon from "../components/Icon.svelte";
  import { avatarEdit } from "../lib/commands";
  import { inTauri } from "../lib/window";
  import { t } from "../lib/i18n";
  import { connectLive, isActive, live } from "./live.svelte";

  let editing = $state(new URLSearchParams(location.search).has("edit"));
  const active = $derived(isActive(live.state));

  onMount(() => {
    connectLive();
    if (inTauri()) listen<boolean>("avatar-edit", (e) => (editing = e.payload));
  });
</script>

<div class="avatar" class:off={live.state === "mic_off"} class:editing data-tauri-drag-region>
  <div class="ring" class:spin={active} aria-hidden="true"></div>
  <div class="orb"><Orb level={live.level} {active} /></div>
  {#if editing}
    <div class="hint" data-tauri-drag-region><Icon name="move" size={14} /> {t("avatar.drag")}</div>
    <button type="button" class="done pressable" aria-label={t("avatar.done")} title={t("avatar.done")} onclick={() => avatarEdit(false)}>
      <Icon name="check" size={16} />
    </button>
  {/if}
</div>

<style>
  .avatar {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: grid;
    place-items: center;
    transition: opacity 400ms var(--ease-out);
  }
  .avatar.off {
    opacity: 0.35;
  }
  .orb {
    position: absolute;
    inset: 6%;
    pointer-events: none;
  }
  /* arc-reactor ring: thin accent arcs that turn while Jarvis is busy */
  .ring {
    position: absolute;
    inset: 16%;
    border-radius: 50%;
    border: 1.5px solid rgba(var(--accent-rgb), 0.18);
    border-top-color: rgba(var(--accent-rgb), 0.75);
    border-bottom-color: rgba(var(--accent-rgb), 0.45);
    opacity: 0;
    scale: 0.92;
    transition:
      opacity 300ms var(--ease-out),
      scale 300ms var(--ease-out);
    pointer-events: none;
  }
  .ring.spin {
    opacity: 1;
    scale: 1;
    animation: spin 2.4s linear infinite;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  .editing {
    cursor: grab;
  }
  .editing::after {
    content: "";
    position: absolute;
    inset: 4px;
    border-radius: 50%;
    border: 1.5px dashed rgba(255, 255, 255, 0.45);
    pointer-events: none;
  }
  .hint {
    position: absolute;
    top: 10px;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 3px 9px;
    border-radius: 999px;
    background: rgba(14, 16, 20, 0.8);
    color: var(--text);
    font-size: 11px;
    font-weight: 600;
  }
  .done {
    position: absolute;
    bottom: 12px;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 0;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--on-accent);
    cursor: pointer;
    box-shadow: 0 6px 16px -4px rgba(var(--accent-rgb), 0.7);
  }
</style>
