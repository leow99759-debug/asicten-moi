<script lang="ts">
  // App icon tile for add-on cards (§9): brand glyph / monogram / UI icon on a solid tile.
  import Icon, { type IconName } from "./Icon.svelte";
  import { BRANDS } from "../lib/brands";
  // glyph = video30 look: dark glass tile, the brand mark in its own colour
  let {
    icon = "",
    color = "",
    name = "",
    size = 40,
    glyph = false,
  }: { icon?: string; color?: string; name?: string; size?: number; glyph?: boolean } = $props();
  const b = $derived(BRANDS[icon]);
  const ui = $derived(icon.startsWith("i:") ? (icon.slice(2) as IconName) : null);
  const text = $derived(b?.text ?? (b?.path || ui ? "" : name.trim().charAt(0).toUpperCase()));
  const solid = $derived(b?.bg ?? (color || (ui ? "var(--accent)" : "#3a3f4a")));
  const dark = $derived(glyph && !!b?.path);
  const bg = $derived(dark ? "#232429" : solid);
  const fg = $derived(dark ? solid : (b?.fg ?? "#fff"));
</script>

<span
  class="bi"
  style="--s: {size}px; --bg: {bg}; --fg: {fg}; --fs: {size * (text.length > 2 ? 0.3 : text.length > 1 ? 0.38 : 0.46)}px"
  aria-hidden="true">
  {#if b?.path}
    <svg viewBox="0 0 24 24" width={size * (dark ? 0.56 : 0.52)} height={size * (dark ? 0.56 : 0.52)}><path d={b.path} /></svg>
  {:else if ui}
    <Icon name={ui} size={size * 0.48} stroke={2} />
  {:else}
    {text}
  {/if}
</span>

<style>
  .bi {
    position: relative;
    flex: none;
    width: var(--s);
    height: var(--s);
    display: grid;
    place-items: center;
    border-radius: calc(var(--s) * 0.26);
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.14), rgba(0, 0, 0, 0.1)), var(--bg);
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.14),
      0 1px 2px rgba(0, 0, 0, 0.35);
    color: var(--fg);
    font-size: var(--fs);
    line-height: 1;
    font-weight: 750;
    letter-spacing: -0.02em;
  }
  svg {
    fill: currentColor;
  }
</style>
