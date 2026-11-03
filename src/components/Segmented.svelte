<script lang="ts" generics="T extends string">
  import { indicator } from "../lib/motion";

  let { value, options, label, onchange }: {
    value: T;
    options: readonly { value: T; label: string }[];
    label: string;
    onchange: (value: T) => void;
  } = $props();
</script>

<div class="seg" role="group" aria-label={label} use:indicator={value}>
  <span class="nav-indicator" data-indicator></span>
  {#each options as option (option.value)}
    <button aria-current={value === option.value ? "true" : undefined} aria-pressed={value === option.value} onclick={() => onchange(option.value)}>{option.label}</button>
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: inline-grid;
    grid-auto-flow: column;
    grid-auto-columns: 1fr;
    padding: 2px;
    border-radius: var(--radius-s);
    border: 1px solid var(--border);
    background: var(--bg);
  }
  .seg .nav-indicator { background: var(--panel-3); border-radius: 4px; }
  .seg .nav-indicator::before { display: none; }
  button {
    position: relative;
    height: 26px;
    padding: 0 12px;
    border-radius: 4px;
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
    transition: color 0.12s;
  }
  button:hover, button[aria-current] { color: var(--text); }
</style>
