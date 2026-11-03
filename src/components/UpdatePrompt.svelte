<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { lang, t } from "../lib/i18n.svelte";
  import { dialog } from "../lib/motion";
  import { toastError } from "../lib/state.svelte";
  import { install, later, RELEASES_URL, updates } from "../lib/updates.svelte";

  const info = $derived(updates.info);
  const shown = $derived(!!info && updates.phase === "idle" && !updates.waiting && updates.dismissed !== info.version && !(info.critical && info.installable));
  const notes = $derived(info ? info.notes[lang()] || info.notes.en : "");
</script>

{#if shown && info}
  <div class="prompt card" role="status" in:dialog out:dialog={{ duration: 100 }}>
    <b>{t(info.critical ? "update.availableCritical" : "update.available", { version: info.version })}</b>
    {#if notes}<p>{notes}</p>{/if}
    <div class="actions">
      <button class="btn ghost sm" onclick={later}>{t("update.later")}</button>
      {#if info.installable}
        <button class="btn primary sm" onclick={install}>{t("update.install")}</button>
      {:else}
        <button class="btn primary sm" onclick={() => openUrl(RELEASES_URL).catch(toastError)}>{t("update.openReleases")}</button>
      {/if}
    </div>
  </div>
{/if}

<style>
  .prompt {
    width: 320px;
    padding: 12px;
    box-shadow: var(--shadow);
  }
  b { display: block; font-size: var(--fs-13); font-weight: 600; }
  p {
    margin: 4px 0 0;
    max-height: 96px;
    overflow: auto;
    color: var(--text-2);
    font-size: var(--fs-12);
    white-space: pre-line;
  }
  .actions { display: flex; justify-content: flex-end; gap: 4px; margin-top: 8px; }
</style>
