<script lang="ts">
  import { flip } from "svelte/animate";
  import { fly } from "svelte/transition";
  import { CircleAlert, CircleCheck, Info, X } from "@lucide/svelte";
  import { app, dismissToast, dur } from "../lib/state.svelte";
</script>

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast card {t.kind}" animate:flip={{ duration: dur(250) }} in:fly={{ y: 16, duration: dur(300) }} out:fly={{ x: 40, duration: dur(200) }}>
      {#if t.kind === "error"}<CircleAlert size={18} />{:else if t.kind === "success"}<CircleCheck size={18} />{:else}<Info size={18} />{/if}
      <span class="selectable">{t.text}</span>
      <button class="x" onclick={() => dismissToast(t.id)} aria-label="Zamknij"><X size={14} /></button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 18px;
    bottom: 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 80;
    max-width: 420px;
  }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    line-height: 1.4;
    user-select: text;
  }
  .toast :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
  }
  .toast.error :global(svg:first-child) {
    color: var(--danger);
  }
  .toast.success :global(svg:first-child) {
    color: var(--success);
  }
  .toast.info :global(svg:first-child) {
    color: var(--accent);
  }
  .x {
    margin-left: auto;
    color: var(--text-3);
    padding: 2px;
  }
  .x:hover {
    color: var(--text);
  }
</style>
