<script lang="ts">
  import { app } from "../lib/state.svelte";

  let { value = $bindable(4096), onchange }: { value?: number; onchange?: (v: number) => void } = $props();

  const total = $derived(app.info?.totalMemoryMb ?? 16384);
  const max = $derived(Math.max(2048, Math.floor((total * 0.85) / 512) * 512));
  const pct = $derived(((value - 512) / (max - 512)) * 100);
  const warn = $derived(value > total * 0.6);
</script>

<div class="mem">
  <input
    type="range"
    min="512"
    {max}
    step="512"
    bind:value
    style:--pct="{pct}%"
    oninput={() => onchange?.(value)}
  />
  <span class="val mono" class:warn>{(value / 1024).toFixed(1)} GB</span>
</div>

<style>
  .mem {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 320px;
  }
  input[type="range"] {
    flex: 1;
    appearance: none;
    height: 6px;
    border-radius: 6px;
    background: linear-gradient(to right, var(--accent) var(--pct), var(--panel-3) var(--pct));
    outline: none;
  }
  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    border: 3px solid var(--accent);
    box-shadow: 0 2px 8px rgb(0 0 0 / 0.3);
    cursor: pointer;
    transition: transform 0.2s var(--ease-spring);
  }
  input[type="range"]::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }
  .val {
    width: 64px;
    text-align: right;
    font-weight: 600;
  }
  .warn {
    color: var(--warning);
  }
</style>
