<script lang="ts">
  // The orb (SPEC §3.1, video26): translucent accent discs, the inner ones wobble. Breathes when idle,
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
      const base = Math.min(w, h) * 0.42;
      const breathe = Math.sin(t * 1.25) * 0.5 + 0.5; // ~5 s cycle
      // video26: flat translucent discs (wide dim halo → dense core); inner ones wobble like
      // liquid. Each disc = one path + one fill, soft edge from a thin radial fade.
      const rings = [
        { r: 1.0, a: 0.13 + glow * 0.04, lag: 0.2, wob: 0.006 },
        { r: 0.8, a: 0.22 + glow * 0.05, lag: 0.45, wob: 0.012 },
        { r: 0.61, a: 0.4 + glow * 0.07, lag: 0.7, wob: 0.022 },
        { r: 0.44, a: 0.78 + glow * 0.12, lag: 1, wob: 0.03 },
      ];
      for (const [i, ring] of rings.entries()) {
        const swell = 1 + breathe * 0.02 + smooth * 0.18 * ring.lag;
        const r = base * ring.r * swell;
        const g = ctx.createRadialGradient(cx, cy, r * 0.94, cx, cy, r * 1.02);
        g.addColorStop(0, `rgba(${rgb}, ${Math.min(1, ring.a)})`);
        g.addColorStop(1, `rgba(${rgb}, 0)`);
        ctx.fillStyle = g;
        ctx.beginPath();
        const wob = ring.wob * (1 + smooth * 2.5);
        for (let k = 0; k <= 64; k++) {
          const th = (k / 64) * Math.PI * 2;
          const rr =
            r *
            (1 +
              wob * Math.sin(3 * th + t * 0.9 + i) +
              wob * 0.7 * Math.sin(5 * th - t * 1.3 + i * 2) +
              wob * 0.4 * Math.sin(2 * th + t * 0.5));
          const x = cx + Math.cos(th) * rr;
          const y = cy + Math.sin(th) * rr;
          if (k === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.closePath();
        ctx.fill();
      }
      // soft light inside the core (liquid glass sheen)
      const cr = base * 0.44 * (1 + smooth * 0.18);
      const core = ctx.createRadialGradient(cx - cr * 0.25, cy - cr * 0.3, 0, cx, cy, cr);
      core.addColorStop(0, `rgba(255, 255, 255, ${0.1 + glow * 0.06})`);
      core.addColorStop(0.55, "rgba(255, 255, 255, 0.025)");
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
