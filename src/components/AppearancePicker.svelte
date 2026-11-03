<script lang="ts">
  import { ICONS } from "../lib/presets";

  let { icon = $bindable("grass") }: { icon?: string } = $props();
</script>

<div class="icons">
  {#each Object.entries(ICONS) as [key, Icon] (key)}
    <button class="ic" class:on={icon === key} onclick={() => (icon = key)} aria-label={key} aria-pressed={icon === key}><Icon size={16} /></button>
  {/each}
</div>

<style>
  .icons {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fill, 32px);
    gap: 4px;
  }
  .ic {
    height: 32px;
    border-radius: var(--radius-s);
    display: grid;
    place-items: center;
    background: var(--panel-2);
    border: 1px solid var(--border);
    color: var(--text-2);
    transition: color 0.12s, border-color 0.12s;
  }
  .ic:hover { color: var(--text); border-color: var(--border-2); }
  .ic.on { color: var(--text); border-color: var(--accent); }
</style>
