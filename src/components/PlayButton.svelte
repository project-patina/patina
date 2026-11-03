<script lang="ts">
  import { Play, Square, X } from "@lucide/svelte";
  import { t } from "../lib/i18n.svelte";
  import { app, play, stopOrCancel } from "../lib/state.svelte";
  import type { Instance } from "../lib/types";

  let { instance, size = "lg" }: { instance: Instance; size?: "lg" | "sm" } = $props();

  const state = $derived(app.running[instance.id]);
  const prog = $derived(app.progress[instance.id]);
  const share = $derived(prog && prog.total > 0 ? Math.min(1, prog.done / prog.total) : 0);
  const pct = $derived(Math.round(share * 100));
  // "Downloading game assets (12/3911)" → title + counter on the second line.
  const title = $derived(prog?.message.replace(/\s*\(\d+\/\d+\)$/, "") ?? t("play.preparing"));
  const toggle = () => (state ? stopOrCancel(instance.id) : play(instance.id));
</script>

{#if size === "lg"}
  <button class="play lg" class:busy={state === "preparing"} class:live={state === "running"} onclick={toggle}>
    {#if state === "preparing"}<span class="fill" style:transform="scaleX({share})"></span>{/if}
    <span class="content">
      {#if state === "preparing"}
        <X size={16} />
        <span class="txt">
          <b>{title}</b>
          <small>{prog && prog.total > 1 ? `${pct}%` : t("play.clickToCancel")}</small>
        </span>
      {:else if state === "running"}
        <Square size={14} fill="currentColor" />
        <b>{t("play.stop")}</b>
      {:else}
        <Play size={16} fill="currentColor" />
        <b>{t("play.play")}</b>
      {/if}
    </span>
  </button>
{:else}
  <button class="play sm" class:busy={state === "preparing"} class:live={state === "running"} onclick={(e) => { e.stopPropagation(); toggle(); }}>
    {#if state === "preparing"}<span class="fill" style:transform="scaleX({share})"></span>{/if}
    <span class="content">
      {#if state === "preparing"}
        <X size={12} /> {prog && prog.total > 1 ? `${pct}%` : "…"}
      {:else if state === "running"}
        <Square size={11} fill="currentColor" /> {t("play.stopShort")}
      {:else}
        <Play size={12} fill="currentColor" /> {t("play.play")}
      {/if}
    </span>
  </button>
{/if}

<style>
  .play {
    position: relative;
    overflow: hidden;
    background: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
    transition: background 0.12s;
  }
  .play:hover { background: var(--accent-strong); }
  .play.busy, .play.live {
    background: var(--panel-3);
    color: var(--text);
  }
  .play.live:hover, .play.busy:hover { background: color-mix(in srgb, var(--panel-3) 80%, var(--text)); }
  .fill {
    position: absolute;
    inset: 0;
    transform-origin: left;
    background: var(--accent);
    opacity: 0.3;
    transition: transform 0.24s var(--ease);
  }
  .content {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
  .lg {
    height: 44px;
    min-width: 168px;
    max-width: 280px;
    padding: 0 20px;
    border-radius: var(--radius);
    font-size: var(--fs-14);
  }
  .txt {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    line-height: 1.2;
    min-width: 0;
  }
  .txt b {
    max-width: 200px;
    font-size: var(--fs-13);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .lg b { font-weight: 600; }
  .txt small { font-size: var(--fs-12); font-weight: 500; opacity: 0.7; }
  .sm {
    height: 28px;
    min-width: 72px;
    padding: 0 8px;
    border-radius: var(--radius-s);
    font-size: var(--fs-12);
  }
  .sm .content { gap: 4px; }
</style>
