<script lang="ts">
  import { Cog, Plus } from "@lucide/svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import SearchInput from "../components/SearchInput.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { loaderLabel, playTime, relativeTime } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { reveal } from "../lib/motion";
  import { bannerFor, iconFor, LOADERS } from "../lib/presets";
  import { app, navigate } from "../lib/state.svelte";

  let query = $state("");
  let filter = $state<"all" | "fabric" | "rust">("all");

  const list = $derived(
    app.instances.filter((i) => (filter === "all" || i.loader === filter) && (!query || i.name.toLowerCase().includes(query.toLowerCase()) || i.mcVersion.includes(query))),
  );
  const count = (loader: string) => app.instances.filter((i) => i.loader === loader).length;
  const filters = $derived([
    { value: "all" as const, label: `${t("instances.all")} ${app.instances.length}` },
    ...LOADERS.map((l) => ({ value: l.id, label: `${l.label} ${count(l.id)}` })),
  ]);
</script>

<div class="view">
  <header class="head">
    <h1 class="page-title">{t("nav.instances")}</h1>
    <button class="btn primary" onclick={() => (app.newInstanceOpen = true)}><Plus size={14} /> {t("home.newInstance")}</button>
  </header>

  <div class="tools">
    <SearchInput bind:value={query} placeholder={t("common.search")} width="200px" />
    <Segmented label={t("instances.filter")} value={filter} options={filters} onchange={(v) => (filter = v)} />
  </div>

  <div class="grid">
    {#each list as i, n (i.id)}
      {@const Icon = iconFor(i.icon)}
      {@const live = app.running[i.id] === "running"}
      <div class="tile card" in:reveal|global={{ index: n }}>
        <button class="banner" style:background={bannerFor(i.banner)} onclick={() => navigate("instance", i.id)} aria-label={t("instances.open", { name: i.name })}>
          <span class="ico"><Icon size={16} /></span>
          {#if live}<span class="live">{t("instances.running")}</span>{/if}
        </button>
        <div class="body">
          <button class="name" onclick={() => navigate("instance", i.id)}>{i.name}</button>
          <p>{loaderLabel(i.loader)} {i.mcVersion}</p>
          <p class="dim">{i.lastPlayed ? `${relativeTime(i.lastPlayed)} · ${playTime(i.playTimeSecs)}` : t("instances.notPlayed")}</p>
          <div class="actions">
            <PlayButton instance={i} size="sm" />
            <button class="btn sm icon" title={t("nav.settings")} aria-label={t("nav.settings")} onclick={() => navigate("instance", i.id, "settings")}><Cog size={14} /></button>
          </div>
        </div>
      </div>
    {:else}
      <p class="empty">{t(app.instances.length ? "instances.noMatch" : "home.noInstances")}</p>
    {/each}
  </div>
</div>

<style>
  .view {
    height: 100%;
    overflow: auto;
    padding: var(--page-pad);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 16px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(208px, 1fr));
    gap: 12px;
  }
  .tile {
    min-width: 0;
    overflow: hidden;
    transition: border-color 0.12s;
  }
  .tile:hover { border-color: var(--border-2); }
  .banner {
    position: relative;
    display: block;
    width: 100%;
    height: 80px;
  }
  .ico {
    position: absolute;
    left: 8px;
    bottom: 8px;
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    color: #eef0ef;
    background: rgb(12 14 15 / 0.48);
  }
  .live {
    position: absolute;
    top: 8px;
    right: 8px;
    padding: 2px 6px;
    border-radius: 4px;
    color: #eef0ef;
    background: rgb(12 14 15 / 0.56);
    font-size: var(--fs-12);
    font-weight: 500;
  }
  .live::before {
    content: "";
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-right: 4px;
    border-radius: 50%;
    background: var(--success);
    vertical-align: 1px;
  }
  .body { padding: 8px 12px 12px; }
  .name {
    display: block;
    width: 100%;
    font-size: var(--fs-14);
    font-weight: 500;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  p {
    margin: 2px 0 0;
    color: var(--text-2);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  p.dim { color: var(--text-3); }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }
  .actions :global(.play) { flex: 1; }
  .empty {
    grid-column: 1 / -1;
    padding: 32px 0;
    color: var(--text-3);
    font-size: var(--fs-13);
    text-align: center;
  }
</style>
