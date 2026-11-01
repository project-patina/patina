<script lang="ts">
  import { tick } from "svelte";
  import { ArrowDownToLine, Copy, Search, Trash2 } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { app, toast } from "../lib/state.svelte";

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
      });
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
    navigator.clipboard?.writeText(lines.map((l) => l.line).join("\n")).then(() => toast("Skopiowano log", "success"));
  }
</script>

<div class="console">
  <div class="bar">
    <div class="search">
      <Search size={14} />
      <input class="input" placeholder="Filtruj…" bind:value={query} />
    </div>
    <label class="chk"><input type="checkbox" bind:checked={errorsOnly} /> Tylko błędy</label>
    <span class="dim count">{lines.length} linii</span>
    <button class="btn sm icon" title="Kopiuj" onclick={copy}><Copy size={14} /></button>
    <button class="btn sm icon" title="Wyczyść" onclick={() => (app.logs[id] = [])}><Trash2 size={14} /></button>
    <button class="btn sm icon" class:on={follow} title="Przewijaj na bieżąco" onclick={() => { follow = true; box && (box.scrollTop = box.scrollHeight); }}><ArrowDownToLine size={14} /></button>
  </div>
  <div class="log selectable mono" bind:this={box} onscroll={onScroll}>
    {#each shown as l, i (i)}
      <div class="ln" class:err={l.error} class:sys={l.line.startsWith("[Patina]")}>{l.line}</div>
    {:else}
      <div class="empty dim">Brak logów. Uruchom grę, aby zobaczyć jej wyjście.</div>
    {/each}
  </div>
</div>

<style>
  .console {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--panel);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    flex: 1;
    max-width: 320px;
  }
  .search :global(svg) {
    position: absolute;
    left: 10px;
    color: var(--text-3);
  }
  .search .input {
    height: 30px;
    padding-left: 30px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    color: var(--text-2);
  }
  .chk input {
    accent-color: var(--accent);
  }
  .count {
    margin-left: auto;
    font-size: 0.8rem;
  }
  .btn.on {
    color: var(--accent);
  }
  .log {
    flex: 1;
    overflow: auto;
    padding: 10px 12px;
    font-size: 0.78rem;
    line-height: 1.55;
    user-select: text;
    background: var(--bg-2);
  }
  .ln {
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--text-2);
  }
  .ln.err {
    color: #f87171;
  }
  .ln.sys {
    color: var(--accent);
  }
  .empty {
    padding: 30px;
    text-align: center;
    font-family: var(--font);
  }
</style>
