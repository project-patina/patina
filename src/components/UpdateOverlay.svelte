<script lang="ts">
  import { fade } from "svelte/transition";
  import BlocksLoader from "./BlocksLoader.svelte";
  import { bytes } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { dur } from "../lib/state.svelte";
  import { continueWithoutUpdate, install, updates } from "../lib/updates.svelte";

  const active = $derived(updates.phase !== "idle");
  const progress = $derived(updates.progress);
  const share = $derived(progress && progress.total > 0 ? Math.min(1, progress.done / progress.total) : 0);
  const stage = $derived.by(() => {
    if (updates.phase === "restarting") return t("update.stageRestart");
    if (progress?.stage === "verify") return t("update.stageVerify");
    if (progress?.stage === "install") return t("update.stageInstall");
    if (progress && progress.total > 0) return t("update.stageDownloadOf", { done: bytes(progress.done), total: bytes(progress.total) });
    return t("update.stageDownload");
  });
</script>

{#if active}
  <div class="overlay" role="alertdialog" aria-modal="true" aria-label={t("update.installing")} transition:fade={{ duration: dur(160) }}>
    <BlocksLoader />
    <h2>{updates.phase === "failed" ? t("update.failed") : t("update.installingVersion", { version: updates.info?.version ?? "" })}</h2>
    {#if updates.phase === "failed"}
      <p class="error selectable">{updates.error}</p>
      <div class="actions">
        <button class="btn" onclick={continueWithoutUpdate}>{t("update.continue")}</button>
        <button class="btn primary" onclick={install}>{t("update.retry")}</button>
      </div>
    {:else}
      <p aria-live="polite">{stage}</p>
      <div class="track"><div class="fill" style:transform="scaleX({updates.phase === 'restarting' ? 1 : share})"></div></div>
    {/if}
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 900;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }
  h2 {
    margin-top: 24px;
    font-family: var(--font-display);
    font-size: var(--fs-20);
    font-weight: 600;
  }
  p {
    margin: 4px 0 0;
    min-height: 20px;
    color: var(--text-2);
    font-size: var(--fs-13);
  }
  .error {
    max-width: 420px;
    color: var(--danger);
    text-align: center;
    user-select: text;
  }
  .track {
    width: 240px;
    height: 2px;
    margin-top: 16px;
    overflow: hidden;
    border-radius: 2px;
    background: var(--panel-3);
  }
  .fill {
    height: 100%;
    transform-origin: left;
    background: var(--accent);
    transition: transform 0.24s var(--ease);
  }
  .actions { display: flex; gap: 8px; margin-top: 16px; }
</style>
