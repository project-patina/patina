<script lang="ts">
  import { LoaderCircle, Search } from "@lucide/svelte";
  import { app, loadVersions, toastError } from "../lib/state.svelte";

  let { value = $bindable("") }: { value?: string } = $props();

  let query = $state("");
  let snapshots = $state(app.settings?.showSnapshots ?? false);
  let old = $state(app.settings?.showOldVersions ?? false);
  let loading = $state(true);

  $effect(() => {
    loadVersions()
      .then((v) => {
        if (!value) value = v.latestRelease;
      })
      .catch(toastError)
      .finally(() => (loading = false));
  });

  const list = $derived(
    (app.versions?.versions ?? []).filter(
      (v) =>
        (v.kind === "release" || (snapshots && v.kind === "snapshot") || (old && v.kind.startsWith("old"))) &&
        (!query || v.id.toLowerCase().includes(query.toLowerCase())),
    ),
  );

  function kindLabel(k: string) {
    return k === "release" ? "" : k === "snapshot" ? "snapshot" : k === "old_beta" ? "beta" : "alpha";
  }
</script>

<div class="picker">
  <div class="top">
    <div class="search">
      <Search size={15} />
      <input class="input" placeholder="Szukaj wersji…" bind:value={query} />
    </div>
    <label class="chk"><input type="checkbox" bind:checked={snapshots} /> Snapshoty</label>
    <label class="chk"><input type="checkbox" bind:checked={old} /> Stare</label>
  </div>
  <div class="list">
    {#if loading}
      <div class="center"><LoaderCircle size={18} class="spin" /></div>
    {:else}
      {#each list as v (v.id)}
        <button class="ver" class:on={v.id === value} onclick={() => (value = v.id)}>
          <span class="mono">{v.id}</span>
          {#if v.id === app.versions?.latestRelease}<span class="chip accent">najnowsza</span>{/if}
          {#if kindLabel(v.kind)}<span class="chip">{kindLabel(v.kind)}</span>{/if}
          <span class="date">{new Date(v.releaseTime).toLocaleDateString("pl-PL")}</span>
        </button>
      {:else}
        <div class="center dim">Brak wyników</div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .picker {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--panel-2);
  }
  .top {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .search {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }
  .search :global(svg) {
    position: absolute;
    left: 11px;
    color: var(--text-3);
  }
  .search .input {
    padding-left: 34px;
    height: 34px;
    background: var(--panel);
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.85rem;
    color: var(--text-2);
    cursor: pointer;
    white-space: nowrap;
  }
  .chk input {
    accent-color: var(--accent);
  }
  .list {
    height: 210px;
    overflow: auto;
    padding: 4px;
  }
  .ver {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 7px 10px;
    border-radius: 7px;
    text-align: left;
    transition: background 0.12s;
  }
  .ver:hover {
    background: var(--panel-3);
  }
  .ver.on {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 1px var(--accent-line);
  }
  .date {
    margin-left: auto;
    font-size: 0.8rem;
    color: var(--text-3);
  }
  .center {
    display: grid;
    place-items: center;
    height: 100%;
  }
</style>
