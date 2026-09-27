<script lang="ts">
  // Reply (SPEC §5.3): voice pack clips by category (random pick inside) and/or text for the
  // neural voice; a recording wins, the text is the fallback. «▶» plays it as Jarvis would.
  import Icon from "./Icon.svelte";
  import type { Reply } from "../lib/bindings/Reply";
  import { sayReply } from "../lib/commands";
  import { t } from "../lib/i18n";
  import { REPLY_CLIPS } from "../lib/phrases";

  let { reply, onchange }: { reply: Reply; onchange: (next: Reply) => void } = $props();

  // pack categories this command already uses but the short list doesn't offer
  const cats = $derived([...REPLY_CLIPS, ...reply.clips.filter((c) => !(REPLY_CLIPS as readonly string[]).includes(c))]);
  const empty = $derived(!reply.clips.length && !reply.text);

  function toggle(c: string) {
    const clips = reply.clips.includes(c) ? reply.clips.filter((x) => x !== c) : [...reply.clips, c];
    onchange({ ...reply, clips });
  }
</script>

<div class="clips" role="group" aria-label={t("editor.sec.reply")}>
  {#each cats as c (c)}
    <button type="button" class="clip" aria-pressed={reply.clips.includes(c)} onclick={() => toggle(c)}>
      <Icon name={reply.clips.includes(c) ? "check" : "wave"} size={12} stroke={2} />
      {t(`reply.${c}`) === `reply.${c}` ? c : t(`reply.${c}`)}
    </button>
  {/each}
</div>
<div class="text-row">
  <input
    value={reply.text ?? ""}
    placeholder={t("card.reply_text")}
    aria-label={t("card.reply_text")}
    onchange={(e) => onchange({ ...reply, text: e.currentTarget.value.trim() || null })} />
  <button type="button" class="btn" disabled={empty} onclick={() => sayReply($state.snapshot(reply))}>
    <Icon name="play" size={14} />
    {t("card.listen")}
  </button>
</div>
<p class="t-caption hint">{t("card.reply_hint")}</p>

<style>
  .clips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .clip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 11px;
    border-radius: 14px;
    border: 1px solid var(--stroke);
    background: var(--fill);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-2);
    cursor: pointer;
    transition:
      background-color var(--t-fast) ease,
      color var(--t-fast) ease,
      transform var(--t-fast) var(--ease-out);
  }
  .clip :global(svg) {
    color: var(--text-3);
  }
  .clip:hover {
    background: var(--fill-hover);
  }
  .clip:active {
    transform: scale(0.97);
  }
  .clip[aria-pressed="true"] {
    background: var(--accent-soft);
    border-color: transparent;
    color: var(--accent-text);
  }
  .clip[aria-pressed="true"] :global(svg) {
    color: currentColor;
  }
  .text-row {
    display: flex;
    gap: 8px;
    margin-top: 10px;
  }
  .text-row input {
    flex: 1;
    min-width: 0;
    height: 32px;
    padding: 0 12px;
    border-radius: var(--r-sm);
    border: 1px solid var(--stroke);
    border-bottom-color: rgba(255, 255, 255, 0.16);
    background: var(--fill);
    color: var(--text);
    font-size: 13px;
    user-select: text;
    outline: none;
  }
  .text-row input:focus {
    background: rgba(0, 0, 0, 0.25);
    border-bottom-color: var(--accent);
    box-shadow: inset 0 -1px 0 var(--accent);
  }
  .hint {
    margin: 6px 0 0;
  }
</style>
