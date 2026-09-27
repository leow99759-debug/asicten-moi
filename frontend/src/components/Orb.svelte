<script lang="ts">
  // The orb (SPEC §3.1): concentric soft discs in the accent color. Breathes when idle,
  // swells with mic/TTS level. Canvas 2D radial gradients (no per-frame blur filter = cheap);
  // stops rendering while the window is hidden.
  import { onMount } from "svelte";

  let { level = 0, active = false }: { level?: number; active?: boolean } = $props();
  let canvas: HTMLCanvasElement;

  onMount(() => {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let raf = 0;
    let smooth = 0;
    let glow = 0;
    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

    const accent = () =>
      getComputedStyle(document.documentElement).getPropertyValue("--accent-rgb").trim() || "59, 130, 246";

    const resize = () => {
      const r = canvas.getBoundingClientRect();
      const dpr = Math.min(devicePixelRatio || 1, 2);
      canvas.width = Math.max(1, Math.round(r.width * dpr));
      canvas.height = Math.max(1, Math.round(r.height * dpr));
    };
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);
    resize();

    const draw = (ms: number) => {
      const { width: w, height: h } = canvas;
      const t = reduced ? 0 : ms / 1000;
      // ease toward the live level: fast attack, slower release (reads as voice, not noise)
      const k = level > smooth ? 0.35 : 0.08;
      smooth += (level - smooth) * k;
      glow += ((active ? 1 : 0) - glow) * 0.06;
      const rgb = accent();
      ctx.clearRect(0, 0, w, h);
      const cx = w / 2;
      const cy = h / 2;
      const base = Math.min(w, h) * 0.46;
      const breathe = Math.sin(t * 1.25) * 0.5 + 0.5; // ~5 s cycle
      // discrete discs like the video: dim wide halo → bright core, each with a soft rim
      const rings = [
        { r: 1.0, a: 0.07 + glow * 0.03, lag: 0.25 },
        { r: 0.8, a: 0.12 + glow * 0.04, lag: 0.5 },
        { r: 0.62, a: 0.22 + glow * 0.05, lag: 0.75 },
        { r: 0.45, a: 0.42 + glow * 0.08, lag: 1 },
      ];
      for (const ring of rings) {
        const swell = 1 + breathe * 0.025 + smooth * 0.16 * ring.lag;
        const r = base * ring.r * swell;
        const g = ctx.createRadialGradient(cx, cy, r * 0.8, cx, cy, r);
        g.addColorStop(0, `rgba(${rgb}, ${ring.a})`);
        g.addColorStop(0.7, `rgba(${rgb}, ${ring.a * 0.85})`);
        g.addColorStop(1, `rgba(${rgb}, 0)`);
        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.arc(cx, cy, r, 0, Math.PI * 2);
        ctx.fill();
      }
      // faint top-left sheen on the core
      const cr = base * 0.45 * (1 + smooth * 0.16);
      const core = ctx.createRadialGradient(cx - cr * 0.3, cy - cr * 0.35, 0, cx, cy, cr);
      core.addColorStop(0, `rgba(255, 255, 255, ${0.09 + glow * 0.05})`);
      core.addColorStop(0.6, "rgba(255, 255, 255, 0.02)");
      core.addColorStop(1, "rgba(255, 255, 255, 0)");
      ctx.fillStyle = core;
      ctx.beginPath();
      ctx.arc(cx, cy, cr, 0, Math.PI * 2);
      ctx.fill();
      raf = document.hidden ? 0 : requestAnimationFrame(draw);
    };

    const onVis = () => {
      if (!document.hidden && !raf) raf = requestAnimationFrame(draw);
    };
    document.addEventListener("visibilitychange", onVis);
    raf = requestAnimationFrame(draw);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      document.removeEventListener("visibilitychange", onVis);
    };
  });
</script>

<canvas bind:this={canvas} class="orb" aria-hidden="true"></canvas>

<style>
  .orb {
    width: 100%;
    height: 100%;
    display: block;
    pointer-events: none;
  }
</style>
