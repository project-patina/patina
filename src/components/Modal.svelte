<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, scale } from "svelte/transition";
  import { dur } from "../lib/state.svelte";

  let { onclose, width = 520, children }: { onclose: () => void; width?: number; children: Snippet } = $props();
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && onclose()} />

<div class="backdrop" transition:fade={{ duration: dur(180) }} onclick={(e) => e.target === e.currentTarget && onclose()} role="presentation">
  <div class="modal card" style:width="{width}px" transition:scale={{ duration: dur(260), start: 0.94, opacity: 0 }} role="dialog" aria-modal="true">
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
    max-height: calc(100vh - 80px);
    overflow: auto;
    box-shadow: var(--shadow);
    padding: 24px;
  }
</style>
