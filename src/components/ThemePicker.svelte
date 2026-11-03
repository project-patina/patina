<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { app, updateSettings } from "../lib/state.svelte";
  import { THEMES } from "../lib/themes";
</script>

<div class="themes" role="group" aria-label={t("look.palette")}>
  {#each THEMES as theme (theme.id)}
    {@const on = app.settings?.palette === theme.id}
    <button class="theme" class:on aria-pressed={on}
      onclick={() => updateSettings({ palette: theme.id, accent: theme.accent })}
      style:--preview-dark={theme.dark} style:--preview-light={theme.light} style:--preview-accent={theme.accent}>
      <span class="preview" aria-hidden="true"><span class="bar"></span><span class="line"></span></span>
      <span class="caption">{theme.name}</span>
    </button>
  {/each}
</div>

<style>
  .themes { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; padding: 12px 0; border-bottom: 1px solid var(--border); }
  .theme { display: block; padding: 4px; text-align: left; border: 1px solid var(--border); border-radius: var(--radius); transition: border-color 0.12s; }
  .theme:hover { border-color: var(--border-2); }
  .theme.on { border-color: var(--accent); }
  .preview { display: flex; align-items: flex-end; gap: 4px; height: 48px; padding: 8px; border-radius: var(--radius-s); background: var(--preview-dark); }
  :global([data-theme="light"]) .preview { background: var(--preview-light); box-shadow: inset 0 0 0 1px var(--border); }
  .bar { width: 24px; height: 8px; border-radius: 2px; background: var(--preview-accent); }
  .line { width: 40px; height: 8px; border-radius: 2px; background: var(--preview-accent); opacity: 0.3; }
  .caption { display: block; padding: 4px; font-size: var(--fs-13); font-weight: 500; }
</style>
