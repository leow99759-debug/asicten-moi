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
  /* Fluent slider: 4 px track, 20 px thumb with an accent core that grows on hover */
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
    height: 4px;
    border-radius: 2px;
    background: linear-gradient(to right, var(--accent) var(--p), rgba(255, 255, 255, 0.16) var(--p));
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 20px;
    height: 20px;
    margin-top: -8px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--accent) 0 5px, #454a55 5.5px);
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.08), 0 1px 3px rgba(0, 0, 0, 0.4);
    transition: background var(--t-fast) ease;
  }
  input:hover::-webkit-slider-thumb {
    background: radial-gradient(circle, var(--accent) 0 6px, #454a55 6.5px);
  }
  input:active::-webkit-slider-thumb {
    background: radial-gradient(circle, var(--accent) 0 4px, #454a55 4.5px);
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
