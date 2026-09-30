<script lang="ts">
  // Listening pill at the top of the screen (Luxify video 32): mic dot, «Джарвис · Слушаю…»,
  // then the live transcript, then ✓ what got done. The core shows/hides the window
  // (overlay.rs) and sends `pill` true/false for the in/out motion.
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import Icon from "../components/Icon.svelte";
  import { inTauri } from "../lib/window";
  import { t } from "../lib/i18n";
  import { connectLive, live } from "./live.svelte";

  let shown = $state(true);
  const WAKE = /^(\s*)(джарвис|jarvis)([,!.\s]*)/i;
  const text = $derived(live.transcript.replace(WAKE, "").trim());
  const mode = $derived(live.result ? "result" : text ? "text" : "idle");

  onMount(() => {
    const demo = new URLSearchParams(location.search).get("pill");
    connectLive().then(() => {
      if (demo === "idle") live.transcript = "";
      if (demo === "done") live.result = { ok: true, text: "Снимок сохранён" };
      if (demo === "error") live.result = { ok: false, text: "Не понял команду" };
    });
    if (inTauri()) listen<boolean>("pill", (e) => (shown = e.payload));
  });
</script>

<div class="wrap">
  <div class="pill" class:out={!shown} class:result={mode === "result"} style="--lv: {Math.min(1, live.level * 1.6).toFixed(3)}">
    {#if mode === "result" && live.result}
      <span class="dot" class:ok={live.result.ok} class:err={!live.result.ok}>
        <Icon name={live.result.ok ? "check" : "x"} size={15} stroke={2.4} />
      </span>
      <p class="line strong">{live.result.text}</p>
    {:else}
      <span class="dot mic"><Icon name="mic" size={15} stroke={2} /></span>
      {#if mode === "text"}
        <p class="line">{text}</p>
      {:else}
        <span class="tag">{t("pill.name")}</span>
        <p class="line hint">{live.state === "processing" ? t("pill.busy") : t("pill.listen")}</p>
      {/if}
    {/if}
  </div>
</div>

<style>
  .wrap {
    width: 100vw;
    height: 100vh;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 10px;
  }
  .pill {
    max-width: calc(100vw - 40px);
    min-width: 260px;
    height: 44px;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 20px 0 7px;
    border-radius: 999px;
    background: rgba(24, 27, 36, 0.94);
    box-shadow:
      0 10px 28px rgba(0, 0, 0, 0.45),
      0 0 0 1px rgba(255, 255, 255, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.06);
    color: var(--text);
    animation: pill-in 260ms var(--ease-out) both;
    transition:
      opacity 170ms ease,
      transform 170ms ease,
      box-shadow var(--t-base) ease;
  }
  .pill.out {
    opacity: 0;
    transform: translateY(-8px) scale(0.97);
  }
  @keyframes pill-in {
    from {
      opacity: 0;
      transform: translateY(-10px) scale(0.96);
    }
  }
  .dot {
    width: 30px;
    height: 30px;
    flex: none;
    display: grid;
    place-items: center;
    border-radius: 50%;
    color: #fff;
  }
  .dot.mic {
    background: var(--accent);
    /* voice-reactive glow, no loop: follows the mic level */
    box-shadow: 0 0 0 calc(var(--lv) * 7px) rgba(var(--accent-rgb), 0.22);
    transition: box-shadow 90ms linear;
  }
  .dot.ok {
    background: rgba(62, 207, 142, 0.16);
    color: var(--ok);
  }
  .dot.err {
    background: rgba(240, 97, 109, 0.16);
    color: var(--err);
  }
  .result .dot {
    animation: dot-pop 220ms var(--ease-out) both;
  }
  @keyframes dot-pop {
    from {
      transform: scale(0.6);
      opacity: 0;
    }
  }
  .tag {
    flex: none;
    padding: 3px 9px;
    border-radius: 7px;
    background: var(--accent);
    color: #fff;
    font-size: 13px;
    line-height: 18px;
    font-weight: 650;
  }
  .line {
    margin: 0;
    min-width: 0;
    font-size: 14.5px;
    line-height: 20px;
    font-weight: 500;
    letter-spacing: -0.006em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: ltr;
  }
  .line.strong {
    font-weight: 600;
  }
  .hint {
    color: var(--text-2);
    font-weight: 450;
  }
  @media (prefers-reduced-motion: reduce) {
    .pill,
    .dot {
      animation: none;
      transition: none;
    }
  }
</style>
