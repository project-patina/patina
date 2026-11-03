<script lang="ts">
  import { app } from "../lib/state.svelte";

  let { value = $bindable(4096), onchange }: { value?: number; onchange?: (v: number) => void } = $props();

  const total = $derived(app.info?.totalMemoryMb ?? 16384);
  const limit = $derived(Math.max(2048, Math.floor((total * 0.85) / 512) * 512));
  const warn = $derived(value > total * 0.6);
</script>

<div class="mem">
  <input type="range" min="512" max={Math.max(limit, value)} step="512" bind:value oninput={() => onchange?.(value)} />
  <span class="val mono" class:warn>{(value / 1024).toFixed(1)} GB</span>
</div>

<style>
  .mem {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 240px;
  }
  input { flex: 1; min-width: 0; }
  .val {
    width: 60px;
    font-size: var(--fs-12);
    text-align: right;
  }
  .warn { color: var(--warning); }
</style>
