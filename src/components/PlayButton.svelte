<script lang="ts">
  import { Play, Square, X } from "@lucide/svelte";
  import { app, play, stopOrCancel } from "../lib/state.svelte";
  import type { Instance } from "../lib/types";
  import { loaderLabel } from "../lib/format";

  let { instance, size = "lg" }: { instance: Instance; size?: "lg" | "sm" } = $props();

  const state = $derived(app.running[instance.id]);
  const prog = $derived(app.progress[instance.id]);
  const pct = $derived(prog && prog.total > 0 ? Math.min(100, (prog.done / prog.total) * 100) : 0);
  // "Pobieranie zasobów gry (12/3911)" → title + counter on the second line.
  const title = $derived(prog?.message.replace(/\s*\(\d+\/\d+\)$/, "") ?? "Przygotowywanie…");
</script>

{#if size === "lg"}
  <button class="play lg" class:busy={state === "preparing"} class:live={state === "running"} onclick={() => (state ? stopOrCancel(instance.id) : play(instance.id))}>
    <span class="fill" style:width="{state === 'preparing' ? pct : 0}%"></span>
    <span class="content">
      {#if state === "preparing"}
        <X size={18} />
        <span class="txt">
          <b>{title}</b>
          <small>{prog && prog.total > 1 ? `${Math.round(pct)}% · ${prog.done}/${prog.total}` : "kliknij, aby anulować"}</small>
        </span>
      {:else if state === "running"}
        <Square size={16} fill="currentColor" />
        <span class="txt"><b>ZATRZYMAJ GRĘ</b><small>gra jest uruchomiona</small></span>
      {:else}
        <Play size={19} fill="currentColor" />
        <span class="txt"><b>GRAJ</b><small>{loaderLabel(instance.loader)} {instance.mcVersion}</small></span>
      {/if}
    </span>
  </button>
{:else}
  <button class="play sm" class:busy={state === "preparing"} class:live={state === "running"} onclick={(e) => { e.stopPropagation(); state ? stopOrCancel(instance.id) : play(instance.id); }}>
    <span class="fill" style:width="{state === 'preparing' ? pct : 0}%"></span>
    <span class="content">
      {#if state === "preparing"}
        <X size={14} /> {prog && prog.total > 1 ? `${Math.round(pct)}%` : "…"}
      {:else if state === "running"}
        <Square size={12} fill="currentColor" /> Stop
      {:else}
        <Play size={14} fill="currentColor" /> Graj
      {/if}
    </span>
  </button>
{/if}

<style>
  .play {
    position: relative;
    overflow: hidden;
    background: var(--play-bg);
    color: var(--play-fg);
    font-weight: 700;
    transition: transform 0.25s var(--ease), box-shadow 0.3s var(--ease), background 0.3s;
  }
  .play:hover {
    transform: translateY(-2px);
    box-shadow: 0 14px 34px -12px color-mix(in srgb, var(--accent) 80%, transparent);
  }
  .play:active {
    transform: scale(0.97);
  }
  .fill {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    background: var(--accent);
    opacity: 0.9;
    transition: width 0.35s var(--ease);
  }
  .busy {
    background: color-mix(in srgb, var(--play-bg) 80%, var(--accent));
  }
  .busy .content {
    color: var(--play-fg);
    mix-blend-mode: normal;
  }
  .live {
    background: color-mix(in srgb, var(--success) 92%, black);
    color: white;
  }
  .content {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }
  .lg {
    height: 58px;
    min-width: 260px;
    max-width: 380px;
    padding: 0 26px;
    border-radius: 14px;
  }
  .lg .txt {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    line-height: 1.15;
    min-width: 0;
  }
  .lg b {
    font-size: 1rem;
    letter-spacing: 0.04em;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 270px;
  }
  .lg small {
    font-size: 0.75rem;
    opacity: 0.6;
    font-weight: 600;
  }
  .sm {
    height: 32px;
    padding: 0 14px;
    border-radius: 9px;
    font-size: 0.86rem;
    min-width: 84px;
  }
  .sm .content {
    gap: 7px;
  }
</style>
