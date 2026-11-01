<script lang="ts">
  import { Check } from "@lucide/svelte";
  import { app, updateSettings } from "../lib/state.svelte";
  import { THEMES } from "../lib/themes";
</script>

<div class="themes" role="group" aria-label="Paleta kolorów">
  {#each THEMES as theme (theme.id)}
    <button class="theme" class:selected={app.settings?.palette === theme.id}
      aria-pressed={app.settings?.palette === theme.id}
      onclick={() => updateSettings({ palette: theme.id, accent: theme.accent })}
      style:--preview-dark={theme.dark} style:--preview-light={theme.light} style:--preview-accent={theme.accent}>
      <span class="preview" aria-hidden="true">
        <span class="rail"></span>
        <span class="mock"><span class="line"></span><span class="landscape"></span><span class="line short"></span></span>
        <span class="swatch"></span>
      </span>
      <span class="caption">{theme.name}<span class="check">{#if app.settings?.palette === theme.id}<Check size={14} />{/if}</span></span>
    </button>
  {/each}
</div>

<style>
  .themes { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 12px; margin: 16px 0; }
  .theme { text-align: left; border: 1px solid var(--border-2); border-radius: var(--radius); padding: 5px; transition: border-color 0.2s, background 0.2s; }
  .theme:hover { background: var(--panel-2); }
  .theme.selected { border-color: var(--accent); background: var(--accent-soft); }
  .preview { height: 88px; border-radius: calc(var(--radius) - 4px); display: flex; padding: 10px; gap: 9px; position: relative; overflow: hidden; background: var(--preview-dark); }
  :global([data-theme="light"]) .preview { background: var(--preview-light); }
  .rail { width: 8px; border-radius: 3px; background: var(--preview-accent); opacity: 0.65; }
  .mock { flex: 1; display: flex; flex-direction: column; gap: 7px; }
  .line { height: 4px; width: 45%; border-radius: 3px; background: var(--preview-accent); opacity: 0.4; }
  .line.short { width: 65%; opacity: 0.2; }
  .landscape { flex: 1; border-radius: 4px; background: linear-gradient(125deg, color-mix(in srgb, var(--preview-accent) 40%, transparent), color-mix(in srgb, var(--preview-accent) 8%, transparent)); }
  .swatch { position: absolute; right: 17px; bottom: 25px; width: 23px; height: 8px; border-radius: 3px; background: var(--preview-accent); }
  .caption { display: flex; align-items: center; justify-content: space-between; padding: 9px 7px 5px; font-size: 0.88rem; font-weight: 550; }
  .check { width: 14px; height: 14px; color: var(--accent); }
  @media (max-width: 900px) { .preview { height: 72px; } }
</style>
