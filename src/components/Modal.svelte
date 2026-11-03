<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";
  import { dialog } from "../lib/motion";
  import { dur } from "../lib/state.svelte";

  let { onclose, width = 520, flush = false, children }: { onclose: () => void; width?: number; flush?: boolean; children: Snippet } = $props();

  function modalFocus(node: HTMLDivElement) {
    const previous = document.activeElement;
    const focusable = () => Array.from(node.querySelectorAll<HTMLElement>('button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), summary, [tabindex="0"]'))
      .filter((element) => element.getClientRects().length > 0);
    queueMicrotask(() => { if (node.isConnected) (focusable()[0] ?? node).focus({ preventScroll: true }); });
    function keydown(event: KeyboardEvent) {
      const dialogs = document.querySelectorAll('[data-modal]');
      if (dialogs[dialogs.length - 1] !== node) return;
      if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); onclose(); }
      if (event.key !== "Tab") return;
      const controls = focusable();
      if (!controls.length) { event.preventDefault(); node.focus(); return; }
      const first = controls[0], last = controls[controls.length - 1];
      if (event.shiftKey && (document.activeElement === first || document.activeElement === node)) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && (document.activeElement === last || document.activeElement === node)) { event.preventDefault(); first.focus(); }
    }
    document.addEventListener("keydown", keydown, true);
    return { destroy() {
      document.removeEventListener("keydown", keydown, true);
      if (previous instanceof HTMLElement && previous.isConnected) previous.focus({ preventScroll: true });
    } };
  }
</script>

<div class="backdrop" transition:fade={{ duration: dur(120) }} onclick={(e) => e.target === e.currentTarget && onclose()} role="presentation">
  <div class="modal" class:flush data-modal tabindex="-1" use:modalFocus style:width="{width}px" in:dialog out:dialog={{ duration: 100 }} role="dialog" aria-modal="true">
    {@render children()}
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: var(--overlay);
    display: grid;
    place-items: center;
    z-index: 50;
  }
  .modal {
    max-width: calc(100vw - 48px);
    max-height: calc(100vh - 48px);
    overflow: auto;
    padding: 24px;
    background: var(--panel);
    border: 1px solid var(--border-2);
    border-radius: var(--radius-l);
    box-shadow: var(--shadow);
  }
  .modal.flush { padding: 0; }
</style>
