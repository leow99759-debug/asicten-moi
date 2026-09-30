<script lang="ts">
  let {
    value = 0,
    label,
    min = 0,
    max = 100,
    suffix = "%",
    oninput,
  }: {
    value?: number;
    label: string;
    min?: number;
    max?: number;
    suffix?: string;
    oninput?: (v: number) => void;
  } = $props();
  const pct = $derived(((value - min) / (max - min)) * 100);
</script>

<div class="slider">
  <input
    type="range"
    {min}
    {max}
    {value}
    aria-label={label}
    style="--p: {pct}%"
    oninput={(e) => oninput?.(Number(e.currentTarget.value))} />
  <span class="val">{value}{suffix}</span>
</div>

<style>
  /* Luxify slider: a thick 14 px bar filled with the theme colour; the whole bar is the handle */
  .slider {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
  }
  input {
    flex: 1;
    min-width: 0;
    appearance: none;
    height: 20px;
    margin: 0;
    background: transparent;
    cursor: pointer;
  }
  input::-webkit-slider-runnable-track {
    height: 14px;
    border-radius: 6px;
    box-shadow: inset 0 0 0 1px var(--stroke);
    background: linear-gradient(to right, var(--accent) var(--p), rgba(255, 255, 255, 0.06) var(--p));
    transition: filter var(--t-fast) ease;
  }
  input:hover::-webkit-slider-runnable-track {
    filter: brightness(1.12);
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 4px;
    height: 14px;
    border-radius: 2px;
    background: transparent;
  }
  .val {
    min-width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
    font-size: 13px;
    font-weight: 500;
  }
</style>
