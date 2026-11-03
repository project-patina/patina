<script lang="ts">
  import { flip } from "svelte/animate";
  import { CircleAlert, CircleCheck, Info, X } from "@lucide/svelte";
  import UpdatePrompt from "./UpdatePrompt.svelte";
  import { t } from "../lib/i18n.svelte";
  import { dialog } from "../lib/motion";
  import { app, dismissToast, dur } from "../lib/state.svelte";
</script>

<div class="toasts">
  {#each app.toasts as toast (toast.id)}
    <div class="toast card {toast.kind}" animate:flip={{ duration: dur(160) }} in:dialog out:dialog={{ duration: 100 }}>
      {#if toast.kind === "error"}<CircleAlert size={16} />{:else if toast.kind === "success"}<CircleCheck size={16} />{:else}<Info size={16} />{/if}
      <span class="selectable">{toast.text}</span>
      <button class="x" onclick={() => dismissToast(toast.id)} aria-label={t("common.close")}><X size={14} /></button>
    </div>
  {/each}
  <UpdatePrompt />
</div>

<style>
  .toasts {
    position: fixed;
    right: 16px;
    bottom: 16px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 8px;
    z-index: 80;
    max-width: 360px;
    pointer-events: none;
  }
  .toasts > :global(*) { pointer-events: auto; }
  .toast {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 12px;
    box-shadow: var(--shadow);
    font-size: var(--fs-13);
    line-height: 1.4;
    user-select: text;
  }
  .toast :global(svg) { flex-shrink: 0; margin-top: 1px; }
  .toast.error :global(svg:first-child) { color: var(--danger); }
  .toast.success :global(svg:first-child) { color: var(--success); }
  .toast.info :global(svg:first-child) { color: var(--text-2); }
  .x {
    margin-left: auto;
    color: var(--text-3);
    padding: 2px;
  }
  .x:hover { color: var(--text); }
</style>
