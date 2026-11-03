<script lang="ts">
  import { tick } from "svelte";
  import { ArrowDownToLine, Copy, Trash2 } from "@lucide/svelte";
  import Option from "./Option.svelte";
  import SearchInput from "./SearchInput.svelte";
  import { api } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { app, toast, toastError } from "../lib/state.svelte";

  let { id }: { id: string } = $props();

  let box: HTMLDivElement | undefined = $state();
  let query = $state("");
  let errorsOnly = $state(false);
  let follow = $state(true);

  $effect(() => {
    const iid = id;
    if (!app.logs[iid]?.length) {
      api.gameLog(iid).then((l) => {
        if (l.length && !app.logs[iid]?.length) app.logs[iid] = l;
      }).catch(toastError);
    }
  });

  const lines = $derived((app.logs[id] ?? []).filter((l) => (!errorsOnly || l.error) && (!query || l.line.toLowerCase().includes(query.toLowerCase()))));
  const shown = $derived(lines.slice(-1500));

  $effect(() => {
    void shown.length;
    if (follow && box) tick().then(() => box && (box.scrollTop = box.scrollHeight));
  });

  function onScroll() {
    if (!box) return;
    follow = box.scrollHeight - box.scrollTop - box.clientHeight < 40;
  }

  function copy() {
    navigator.clipboard?.writeText(lines.map((l) => l.line).join("\n")).then(() => toast(t("console.copied"), "success")).catch(toastError);
  }
</script>

<div class="console">
  <div class="bar">
    <SearchInput bind:value={query} placeholder={t("console.filter")} width="220px" />
    <Option bind:checked={errorsOnly} label={t("console.errorsOnly")} />
    <span class="count">{t("console.lines", { n: lines.length })}</span>
    <button class="btn sm icon" title={t("common.copy")} aria-label={t("common.copy")} onclick={copy}><Copy size={14} /></button>
    <button class="btn sm icon" title={t("console.clear")} aria-label={t("console.clear")} onclick={() => (app.logs[id] = [])}><Trash2 size={14} /></button>
    <button class="btn sm icon" class:on={follow} title={t("console.follow")} aria-label={t("console.follow")} aria-pressed={follow} onclick={() => { follow = true; box && (box.scrollTop = box.scrollHeight); }}><ArrowDownToLine size={14} /></button>
  </div>
  <div class="log selectable mono" bind:this={box} onscroll={onScroll}>
    {#each shown as l, i (i)}
      <div class="ln" class:err={l.error} class:sys={l.line.startsWith("[Patina]")}>{l.line}</div>
    {:else}
      <div class="empty">{t("console.empty")}</div>
    {/each}
  </div>
</div>

<style>
  .console {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 240px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--panel);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px 12px;
    flex-wrap: wrap;
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .count {
    margin-left: auto;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .btn.on { color: var(--accent-strong); }
  .log {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 8px 12px;
    font-size: var(--fs-12);
    line-height: 1.5;
    user-select: text;
    background: var(--bg-2);
  }
  .ln {
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-2);
  }
  .ln.err { color: var(--danger); }
  .ln.sys { color: var(--accent-strong); }
  .empty {
    padding: 32px;
    color: var(--text-3);
    font-family: var(--font);
    font-size: var(--fs-13);
    text-align: center;
  }
</style>
