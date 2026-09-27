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
    height: 4px;
    border-radius: 2px;
    background: linear-gradient(to right, var(--accent) var(--p), rgba(255, 255, 255, 0.1) var(--p));
    cursor: pointer;
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
    transition: transform var(--t-press) var(--ease-out);
  }
  input:active::-webkit-slider-thumb {
    transform: scale(1.15);
  }
  .val {
    width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
    color: var(--text-2);
    font-size: 13px;
  }
</style>
