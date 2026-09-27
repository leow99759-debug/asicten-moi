<script lang="ts">
  // Fluent toggle: 40×20, knob grows on hover and stretches while pressed.
  let {
    checked = false,
    label,
    disabled = false,
    onchange,
  }: { checked?: boolean; label: string; disabled?: boolean; onchange?: (v: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  class="toggle"
  class:on={checked}
  onclick={() => onchange?.(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 40px;
    height: 20px;
    flex: none;
    padding: 0;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.55);
    background: rgba(0, 0, 0, 0.1);
    cursor: pointer;
    transition:
      background-color var(--t-base) ease,
      border-color var(--t-base) ease;
  }
  .toggle:hover {
    background: rgba(255, 255, 255, 0.05);
  }
  .knob {
    position: absolute;
    top: 50%;
    left: 4px;
    width: 12px;
    height: 12px;
    margin-top: -6px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.8);
    transition:
      translate var(--t-base) var(--spring),
      width var(--t-fast) var(--ease-out),
      height var(--t-fast) var(--ease-out),
      margin var(--t-fast) var(--ease-out),
      background-color var(--t-base) ease;
  }
  .toggle:hover .knob {
    width: 14px;
    height: 14px;
    margin-top: -7px;
    margin-left: -1px;
  }
  .toggle:active .knob {
    width: 17px;
  }
  .toggle.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .toggle.on:hover {
    background: color-mix(in srgb, var(--accent) 88%, white);
  }
  .toggle.on .knob {
    translate: 20px 0;
    background: var(--on-accent);
  }
  .toggle.on:active .knob {
    translate: 17px 0;
  }
  .toggle:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
