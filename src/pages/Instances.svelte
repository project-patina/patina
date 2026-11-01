<script lang="ts">
  import { fly } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { Cog, Plus, Search } from "@lucide/svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import { bannerFor, iconFor } from "../lib/presets";
  import { loaderLabel, playTime, relativeTime } from "../lib/format";
  import { app, dur, navigate } from "../lib/state.svelte";

  let query = $state("");
  let filter = $state<"all" | "fabric" | "quilt" | "vanilla" | "rust">("all");

  const list = $derived(
    app.instances.filter((i) => (filter === "all" || i.loader === filter) && (!query || i.name.toLowerCase().includes(query.toLowerCase()) || i.mcVersion.includes(query))),
  );
  const count = (l: string) => app.instances.filter((i) => l === "all" || i.loader === l).length;
</script>

<div class="page">
  <header class="head">
    <h1>Instancje</h1>
    <div class="tools">
      <div class="search">
        <Search size={15} />
        <input class="input" placeholder="Szukaj…" bind:value={query} />
      </div>
      <button class="btn primary" onclick={() => (app.newInstanceOpen = true)}><Plus size={16} /> Nowa instancja</button>
    </div>
  </header>

  <div class="tabs">
    {#each [["all", "Wszystkie"], ["fabric", "Fabric"], ["quilt", "Quilt"], ["vanilla", "Vanilla"], ["rust", "Rust"]] as [k, label] (k)}
      <button class="tab" class:on={filter === k} onclick={() => (filter = k as typeof filter)}>{label} <span>{count(k)}</span></button>
    {/each}
  </div>

  <div class="grid">
    {#each list as i, n (i.id)}
      {@const Icon = iconFor(i.icon)}
      <div class="tile card" class:running={app.running[i.id] === "running"} animate:flip={{ duration: dur(300) }} in:fly={{ y: 16, duration: dur(400), delay: dur(Math.min(n, 12) * 40) }}>
        <button class="banner" style:background={bannerFor(i.banner)} onclick={() => navigate("instance", i.id)} aria-label="Otwórz {i.name}">
          <span class="big-ico"><Icon size={30} /></span>
          {#if app.running[i.id] === "running"}<span class="live chip">● W grze</span>{/if}
        </button>
        <div class="body">
          <button class="name" onclick={() => navigate("instance", i.id)}>{i.name}</button>
          <div class="meta">
            <span class="chip">{loaderLabel(i.loader)}</span>
            <span class="chip mono">{i.mcVersion}</span>
          </div>
          <div class="sub dim">{relativeTime(i.lastPlayed)} · {playTime(i.playTimeSecs)}</div>
          <div class="actions">
            <PlayButton instance={i} size="sm" />
            <button class="btn sm icon" aria-label="Ustawienia" onclick={() => navigate("instance", i.id, "settings")}><Cog size={15} /></button>
          </div>
        </div>
      </div>
    {/each}
    {#if list.length === 0}
      <p class="empty muted">{app.instances.length ? "Brak instancji pasujących do filtrów." : "Nie masz jeszcze instancji. Dodaj pierwszą przyciskiem powyżej."}</p>
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100%;
    overflow: auto;
    padding: 6px 22px 22px 6px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 4px 0 14px;
  }
  h1 {
    font-size: 1.65rem;
    font-weight: 700;
  }
  .tools {
    display: flex;
    gap: 8px;
  }
  .search {
    position: relative;
    display: flex;
    align-items: center;
    width: 240px;
  }
  .search :global(svg) {
    position: absolute;
    left: 11px;
    color: var(--text-3);
  }
  .search .input {
    padding-left: 34px;
    height: 36px;
  }
  .tabs {
    display: flex;
    gap: 4px;
    padding: 4px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 11px;
    width: fit-content;
    margin-bottom: 16px;
  }
  .tab {
    padding: 6px 12px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.88rem;
    color: var(--text-2);
    transition: background 0.2s, color 0.2s;
  }
  .tab span {
    color: var(--text-3);
    margin-left: 4px;
    font-size: 0.8rem;
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.on {
    background: var(--panel-3);
    color: var(--text);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 14px;
  }
  .tile {
    overflow: hidden;
    display: flex;
    flex-direction: column;
    transition: transform 0.3s var(--ease), border-color 0.2s, box-shadow 0.3s;
  }
  .tile:hover {
    transform: translateY(-3px);
    border-color: var(--border-2);
    box-shadow: var(--shadow);
  }
  .tile.running {
    border-color: color-mix(in srgb, var(--success) 55%, transparent);
  }
  .banner {
    height: 104px;
    position: relative;
    display: grid;
    place-items: center;
  }
  .big-ico {
    width: 56px;
    height: 56px;
    border-radius: 16px;
    display: grid;
    place-items: center;
    color: white;
    background: rgb(255 255 255 / 0.12);
    border: 1px solid rgb(255 255 255 / 0.15);
    backdrop-filter: blur(6px);
    transition: transform 0.4s var(--ease-spring);
  }
  .tile:hover .big-ico {
    transform: scale(1.08) rotate(-4deg);
  }
  .live {
    position: absolute;
    top: 10px;
    right: 10px;
    color: white;
    background: color-mix(in srgb, var(--success) 75%, black);
    border: 0;
  }
  .body {
    padding: 12px 14px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .name {
    font-weight: 650;
    font-size: 1rem;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    display: flex;
    gap: 6px;
  }
  .sub {
    font-size: 0.8rem;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 2px;
  }
  .actions :global(.play) {
    flex: 1;
  }
</style>
