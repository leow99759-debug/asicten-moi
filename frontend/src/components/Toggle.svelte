<script lang="ts">
  let {
    checked = false,
    label,
    onchange,
  }: { checked?: boolean; label: string; onchange?: (v: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="toggle"
  class:on={checked}
  onclick={() => onchange?.(!checked)}>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    width: 38px;
    height: 22px;
    padding: 0;
    border-radius: 11px;
    border: 1px solid var(--line-2);
    background: rgba(255, 255, 255, 0.08);
    position: relative;
    cursor: pointer;
    flex: none;
    transition: background-color var(--t-fast) ease, border-color var(--t-fast) ease;
  }
  .toggle.on {
    background: var(--accent);
    border-color: transparent;
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.35);
    transition: transform var(--t-med) var(--ease-out), width var(--t-press) var(--ease-out);
  }
  .toggle.on .knob {
    transform: translateX(16px);
  }
  /* press stretches the knob like iOS/Win11 */
  .toggle:active .knob {
    width: 20px;
  }
  .toggle.on:active .knob {
    transform: translateX(12px);
  }
</style>
