<script lang="ts">
  // Arc-reactor mark: accent disc, bright ring, triangle core.
  let { size = 36 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 40 40" aria-hidden="true">
  <defs>
    <radialGradient id="lg-core" cx="50%" cy="45%" r="60%">
      <stop offset="0" stop-color="rgb(var(--accent-rgb))" stop-opacity="1" />
      <stop offset="1" stop-color="rgb(var(--accent-rgb))" stop-opacity="0.72" />
    </radialGradient>
  </defs>
  <circle cx="20" cy="20" r="19" fill="url(#lg-core)" />
  <circle cx="20" cy="20" r="12.5" fill="none" stroke="#fff" stroke-opacity="0.9" stroke-width="2" />
  <path d="M20 12.8 26.3 23.8H13.7Z" fill="#fff" />
</svg>
