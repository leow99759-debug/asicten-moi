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
    background: rgba(5, 6, 9, 0.5);
    backdrop-filter: blur(8px);
    z-index: 100;
    transition: opacity 200ms var(--ease-out);
    @starting-style {
      opacity: 0;
    }
  }
  .dialog {
    width: min(420px, 90vw);
    padding: 28px 28px 22px;
    border-radius: var(--r-xl);
    background: rgba(28, 31, 38, 0.92);
    backdrop-filter: blur(var(--blur)) saturate(140%);
    border: 1px solid var(--line-2);
    box-shadow: var(--shadow-lg), inset 0 1px 0 var(--highlight);
    text-align: center;
    transition: transform 240ms var(--ease-out), opacity 240ms var(--ease-out);
    @starting-style {
      transform: scale(0.96);
      opacity: 0;
    }
  }
  .icon {
    width: 56px;
    height: 56px;
    margin: 0 auto;
    display: grid;
    place-items: center;
    border-radius: 18px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  h2 {
    margin: 14px 0 6px;
    font: 600 17px/1.3 var(--font-display);
  }
  #confirm-q {
    margin: 0;
    color: var(--text-2);
  }
  .buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin: 22px 0 14px;
  }
  button {
    height: 40px;
    border-radius: var(--r-md);
    border: 1px solid var(--line-2);
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    cursor: pointer;
    transition: transform var(--t-press) var(--ease-out), background-color var(--t-fast) ease;
  }
  button:hover {
    background: rgba(255, 255, 255, 0.09);
  }
  button:active {
    transform: scale(0.97);
  }
  button.primary {
    background: var(--accent);
    border-color: transparent;
    color: var(--on-accent);
  }
  button.primary:hover {
    background: color-mix(in srgb, var(--accent) 88%, white);
  }
  .hint {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--text-3);
  }
</style>
