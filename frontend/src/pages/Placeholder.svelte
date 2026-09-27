<script lang="ts">
  import Icon, { type IconName } from "../components/Icon.svelte";
  import PageHeader from "../components/PageHeader.svelte";
  import type { Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n";
  let { page, icon, milestone }: { page: Page; icon: IconName; milestone: string } = $props();
</script>

<PageHeader title={t(`title.${page}`)} />
<div class="empty card">
  <span class="ic"><Icon name={icon} size={24} /></span>
  <p class="t-subtitle">{t("soon.title")}</p>
  <p class="t-caption">{t("soon.sub").replace("{m}", milestone)}</p>
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 56px 24px;
    text-align: center;
  }
  .ic {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
    border-radius: var(--r-lg);
    background: var(--accent-soft);
    color: var(--accent-text);
    margin-bottom: 10px;
  }
  p {
    margin: 0;
  }
</style>
