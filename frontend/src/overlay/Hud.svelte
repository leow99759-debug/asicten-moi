<script lang="ts">
  // HUD skin (SPEC §3.7): Iron Man style rings over the screen while Jarvis listens and
  // answers. Pure SVG + CSS animation; the core shows/hides the window (overlay.rs) and
  // sends `hud` true/false so it can play the in/out motion.
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { inTauri } from "../lib/window";
  import { t } from "../lib/i18n";
  import { connectLive, live } from "./live.svelte";

  const C = 230;
  const still = new URLSearchParams(location.search).has("still");
  let shown = $state(true);
  let pct = $state(still ? 100 : 0);
  let run = $state(0); // restarts the draw-in animations

  const label = $derived(
    live.state === "processing" ? t("hud.busy") : live.state === "speaking" ? t("hud.speak") : t("hud.listen"),
  );
  const text = $derived(live.transcript.length > 30 ? "…" + live.transcript.slice(-29) : live.transcript);

  function countUp() {
    if (still) return;
    const t0 = performance.now();
    const step = (now: number) => {
      const k = Math.min(1, (now - t0) / 1100);
      pct = Math.round(100 * (1 - (1 - k) ** 3));
      if (k < 1 && shown) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  }

  onMount(() => {
    connectLive();
    countUp();
    if (inTauri())
      listen<boolean>("hud", (e) => {
        if (e.payload && !shown) {
          run++;
          countUp();
        }
        shown = e.payload;
      });
  });

  const ticks = Array.from({ length: 120 }, (_, i) => i);
  const arc = (r: number, from: number, to: number) => {
    const p = (a: number) => [C + r * Math.cos(((a - 90) * Math.PI) / 180), C + r * Math.sin(((a - 90) * Math.PI) / 180)];
    const [x1, y1] = p(from);
    const [x2, y2] = p(to);
    return `M${x1.toFixed(1)} ${y1.toFixed(1)}A${r} ${r} 0 ${to - from > 180 ? 1 : 0} 1 ${x2.toFixed(1)} ${y2.toFixed(1)}`;
  };
  const GAUGE = 2 * Math.PI * 138;
</script>

{#key run}
  <div class="hud" class:out={!shown} style="--lv: {live.level.toFixed(3)}">
    <svg viewBox="0 0 460 460" aria-hidden="true">
      <defs>
        <radialGradient id="core">
          <stop offset="0" stop-color="rgb(var(--accent-rgb))" stop-opacity="0.55" />
          <stop offset="0.6" stop-color="rgb(var(--accent-rgb))" stop-opacity="0.16" />
          <stop offset="1" stop-color="rgb(var(--accent-rgb))" stop-opacity="0" />
        </radialGradient>
      </defs>

      <!-- outer tick ring -->
      <g class="rot slow">
        {#each ticks as i (i)}
          <line
            x1={C}
            y1={i % 10 === 0 ? 6 : 12}
            x2={C}
            y2="20"
            transform="rotate({i * 3} {C} {C})"
            class:major={i % 10 === 0} />
        {/each}
      </g>

      <!-- blue segments -->
      <g class="rot cw">
        <circle cx={C} cy={C} r="200" class="thin" />
        <path d={arc(200, 10, 70)} class="seg draw" />
        <path d={arc(200, 130, 160)} class="seg draw d1" />
        <path d={arc(200, 220, 310)} class="seg draw d2" />
      </g>

      <!-- red counter-rotating arcs -->
      <g class="rot ccw">
        <path d={arc(182, 300, 350)} class="red draw d1" />
        <path d={arc(182, 110, 150)} class="red draw d2" />
        <path d={arc(176, 20, 40)} class="red thinred" />
        <path d={arc(176, 200, 215)} class="red thinred" />
      </g>

      <!-- voice-reactive dashed ring -->
      <g class="level">
        <circle cx={C} cy={C} r="160" class="dash" />
      </g>

      <!-- boot gauge -->
      <circle cx={C} cy={C} r="138" class="track" />
      <circle
        cx={C}
        cy={C}
        r="138"
        class="gauge"
        stroke-dasharray={GAUGE}
        stroke-dashoffset={GAUGE * (1 - pct / 100)}
        transform="rotate(-90 {C} {C})" />

      <!-- core -->
      <circle cx={C} cy={C} r="112" fill="url(#core)" class="core" />
      <circle cx={C} cy={C} r="84" class="inner" />

      <text x={C} y="212" class="name">J.A.R.V.I.S.</text>
      <text x={C} y="246" class="pct">{pct}<tspan class="unit">%</tspan></text>
      <text x={C} y="272" class="state">{label}</text>

      <!-- side readouts -->
      <text x="18" y="232" class="side left">{t("hud.power")}</text>
      <text x="18" y="248" class="side left val">{pct}%</text>
      <text x="442" y="232" class="side right">{t("hud.systems")}</text>
      <text x="442" y="248" class="side right val">{pct === 100 ? t("hud.ok") : "…"}</text>
    </svg>
    {#if text}<div class="said">{text}</div>{/if}
  </div>
{/key}

<style>
  .hud {
    position: relative;
    width: 100vw;
    height: 100vh;
    color: rgb(var(--accent-rgb));
    animation: hud-in 520ms var(--ease-out) both;
    transition:
      opacity 500ms var(--ease-out),
      scale 500ms var(--ease-out);
  }
  .hud.out {
    opacity: 0;
    scale: 1.06;
  }
  @keyframes hud-in {
    from {
      opacity: 0;
      scale: 0.82;
    }
  }
  svg {
    width: 100%;
    height: 100%;
    display: block;
    overflow: visible;
    filter: drop-shadow(0 0 6px rgba(var(--accent-rgb), 0.55));
  }
  line {
    stroke: rgba(var(--accent-rgb), 0.45);
    stroke-width: 1.2;
  }
  line.major {
    stroke: rgba(var(--accent-rgb), 0.95);
    stroke-width: 2;
  }
  .rot {
    transform-origin: 230px 230px;
  }
  .slow {
    animation: spin 60s linear infinite;
  }
  .cw {
    animation: spin 14s linear infinite;
  }
  .ccw {
    animation: spin 10s linear infinite reverse;
  }
  @keyframes spin {
    to {
      rotate: 360deg;
    }
  }
  circle,
  path {
    fill: none;
  }
  .thin {
    stroke: rgba(var(--accent-rgb), 0.22);
    stroke-width: 1;
  }
  .seg {
    stroke: rgb(var(--accent-rgb));
    stroke-width: 6;
    stroke-linecap: butt;
  }
  .red {
    stroke: #ff4545;
    stroke-width: 4;
    filter: drop-shadow(0 0 5px rgba(255, 69, 69, 0.7));
  }
  .thinred {
    stroke-width: 1.5;
  }
  .draw {
    stroke-dasharray: 400;
    stroke-dashoffset: 400;
    animation: draw 900ms var(--ease-out) forwards;
  }
  .d1 {
    animation-delay: 120ms;
  }
  .d2 {
    animation-delay: 240ms;
  }
  @keyframes draw {
    to {
      stroke-dashoffset: 0;
    }
  }
  .level {
    transform-origin: 230px 230px;
    scale: calc(1 + var(--lv) * 0.09);
    transition: scale 90ms linear;
  }
  .dash {
    stroke: rgba(var(--accent-rgb), 0.7);
    stroke-width: 3;
    stroke-dasharray: 2 7;
  }
  .track {
    stroke: rgba(var(--accent-rgb), 0.14);
    stroke-width: 8;
  }
  .gauge {
    stroke: rgb(var(--accent-rgb));
    stroke-width: 8;
    stroke-linecap: round;
  }
  .core {
    transform-origin: 230px 230px;
    scale: calc(0.94 + var(--lv) * 0.14);
    transition: scale 90ms linear;
  }
  .inner {
    stroke: rgba(255, 255, 255, 0.35);
    stroke-width: 1;
    fill: rgba(8, 12, 20, 0.35);
  }
  text {
    text-anchor: middle;
    fill: #eaf4ff;
    font-family: var(--font-display);
  }
  .name {
    font-size: 13px;
    letter-spacing: 0.32em;
    font-weight: 600;
    fill: rgba(234, 244, 255, 0.75);
  }
  .pct {
    font-size: 34px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .unit {
    font-size: 16px;
    fill: rgba(234, 244, 255, 0.7);
  }
  .state {
    font-size: 11px;
    letter-spacing: 0.24em;
    font-weight: 600;
    fill: rgb(var(--accent-rgb));
  }
  .side {
    font-size: 10px;
    letter-spacing: 0.18em;
    font-weight: 600;
    fill: rgba(234, 244, 255, 0.6);
  }
  .side.val {
    font-size: 13px;
    fill: #eaf4ff;
    font-variant-numeric: tabular-nums;
  }
  .left {
    text-anchor: start;
  }
  .right {
    text-anchor: end;
  }
  .said {
    position: absolute;
    left: 50%;
    bottom: 64px;
    translate: -50% 0;
    max-width: 80%;
    padding: 4px 12px;
    border-radius: 999px;
    background: rgba(8, 12, 20, 0.55);
    color: #eaf4ff;
    font-size: 13px;
    white-space: nowrap;
    text-shadow: 0 0 6px rgba(var(--accent-rgb), 0.6);
  }
</style>
