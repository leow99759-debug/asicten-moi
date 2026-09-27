<script lang="ts">
  import Icon, { type IconName } from "../components/Icon.svelte";
  import { t } from "../lib/i18n";
  let { icon, milestone }: { icon: IconName; milestone: string } = $props();
</script>

<div class="ph">
  <span class="ic"><Icon name={icon} size={28} /></span>
  <p class="t">{t("soon.title")}</p>
  <p class="s">{t("soon.sub").replace("{m}", milestone)}</p>
</div>

<style>
  .ph {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    color: var(--text-2);
  }
  .ic {
    width: 64px;
    height: 64px;
    display: grid;
    place-items: center;
    border-radius: 20px;
    background: var(--accent-soft);
    color: var(--accent);
    margin-bottom: 8px;
  }
  .t {
    margin: 0;
    font-weight: 600;
    color: var(--text);
  }
  .s {
    margin: 0;
    font-size: 13px;
  }
</style>
