<script lang="ts">
  import { BANNERS, ICONS } from "../lib/presets";

  let { icon = $bindable("grass"), banner = $bindable("aurora") }: { icon?: string; banner?: string } = $props();
</script>

<div class="wrap">
  <div>
    <div class="label">Ikona</div>
    <div class="icons">
      {#each Object.entries(ICONS) as [key, Icon] (key)}
        <button class="ic" class:on={icon === key} onclick={() => (icon = key)} aria-label={key}><Icon size={17} /></button>
      {/each}
    </div>
  </div>
  <div>
    <div class="label">Tło</div>
    <div class="banners">
      {#each Object.entries(BANNERS) as [key, bg] (key)}
        <button class="bn" class:on={banner === key} style:background={bg} onclick={() => (banner = key)} aria-label={key}></button>
      {/each}
    </div>
  </div>
</div>

<style>
  .wrap {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }
  .label {
    margin-bottom: 8px;
  }
  .icons {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 5px;
  }
  .ic {
    aspect-ratio: 1;
    border-radius: 8px;
    display: grid;
    place-items: center;
    background: var(--panel-2);
    border: 1px solid var(--border);
    color: var(--text-2);
    transition: all 0.15s;
  }
  .ic:hover {
    color: var(--text);
    border-color: var(--border-2);
  }
  .ic.on {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .banners {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .bn {
    height: 34px;
    border-radius: 8px;
    border: 2px solid transparent;
    transition: transform 0.2s var(--ease), border-color 0.15s;
  }
  .bn:hover {
    transform: scale(1.05);
  }
  .bn.on {
    border-color: var(--text);
  }
</style>
