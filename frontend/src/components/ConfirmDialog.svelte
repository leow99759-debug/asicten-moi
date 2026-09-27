<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { on } from "../lib/ipc";
  import { t } from "../lib/i18n";
  import { confirmAnswer } from "../lib/commands";
  import { inTauri } from "../lib/window";

  let question = $state<string | null>(null);
  let left = $state(0);
  let timer: ReturnType<typeof setInterval> | undefined;
  const unsubs: Array<() => void> = [];

  function close() {
    question = null;
    clearInterval(timer);
  }

  onMount(async () => {
    if (!inTauri()) return;
    unsubs.push(
      await on("confirm", (req) => {
        question = req.question;
        left = req.timeout_sec;
        clearInterval(timer);
        timer = setInterval(() => (left = Math.max(0, left - 1)), 1000);
      }),
      await on("confirm_closed", close),
    );
  });
  onDestroy(() => {
    unsubs.forEach((u) => u());
    clearInterval(timer);
  });

  function answer(yes: boolean) {
    confirmAnswer(yes);
    close();
  }
</script>

{#if question}
  <div class="backdrop" role="presentation">
    <div class="dialog" role="alertdialog" aria-labelledby="confirm-title" aria-describedby="confirm-q">
      <div class="icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="28" height="28"><path fill="currentColor" d="M12 2 4 5v6c0 5 3.4 9.7 8 11 4.6-1.3 8-6 8-11V5l-8-3z" /></svg>
      </div>
      <h2 id="confirm-title">{t("confirm.title")}</h2>
      <p id="confirm-q">{question}</p>
      <div class="buttons">
        <button type="button" class="primary" onclick={() => answer(true)}>{t("confirm.yes")}</button>
        <button type="button" onclick={() => answer(false)}>{t("confirm.no")}</button>
      </div>
      <p class="hint">{t("confirm.autocancel").replace("{n}", String(left))}</p>
      <p class="hint">{t("confirm.voice")}</p>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    display: grid;
    place-items: center;
    background: rgba(0, 0, 0, 0.45);
    backdrop-filter: blur(6px);
    z-index: 100;
  }
  .dialog {
    width: min(420px, 90vw);
    padding: 24px;
    border-radius: 16px;
    background: rgba(30, 33, 40, 0.92);
    border: 1px solid rgba(255, 255, 255, 0.06);
    text-align: center;
  }
  .icon {
    color: var(--accent, #3b82f6);
  }
  h2 {
    margin: 8px 0;
    font-size: 16px;
  }
  .buttons {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin: 16px 0 8px;
  }
  button {
    padding: 8px 18px;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: transparent;
    color: inherit;
    cursor: pointer;
  }
  button.primary {
    background: var(--accent, #3b82f6);
    border-color: transparent;
  }
  .hint {
    margin: 4px 0 0;
    font-size: 12px;
    opacity: 0.6;
  }
</style>
