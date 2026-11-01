<script lang="ts">
  import { fly } from "svelte/transition";
  import { Check, Download, Heart, LoaderCircle, Search } from "@lucide/svelte";
  import { api, errText } from "../lib/api";
  import { compact, kindLabel, loaderLabel, relativeTime } from "../lib/format";
  import { app, dur, toast, toastError } from "../lib/state.svelte";
  import type { ContentKind, SearchHit, SearchQuery } from "../lib/types";

  const CATEGORIES: Record<ContentKind, [string, string][]> = {
    mod: [["optimization", "Optymalizacja"], ["technology", "Technologia"], ["adventure", "Przygoda"], ["decoration", "Dekoracje"], ["magic", "Magia"], ["storage", "Przechowywanie"], ["utility", "Narzędzia"], ["worldgen", "Generowanie świata"], ["mobs", "Moby"], ["food", "Jedzenie"], ["library", "Biblioteki"]],
    shader: [["realistic", "Realistyczne"], ["semi-realistic", "Półrealistyczne"], ["cartoon", "Kreskówkowe"], ["fantasy", "Fantasy"], ["vanilla-like", "Jak vanilla"], ["potato", "Słabe PC"], ["low", "Niskie"], ["high", "Wysokie"]],
    resourcepack: [["16x", "16x"], ["32x", "32x"], ["64x", "64x+"], ["realistic", "Realistyczne"], ["simplistic", "Minimalistyczne"], ["themed", "Tematyczne"], ["tweaks", "Poprawki"], ["combat", "PvP"], ["gui", "GUI"], ["fonts", "Czcionki"]],
  };
  const SORTS = [["relevance", "Trafność"], ["downloads", "Pobrania"], ["follows", "Obserwujący"], ["newest", "Najnowsze"], ["updated", "Zaktualizowane"]] as const;

  let query = $state("");
  let sort = $state<NonNullable<SearchQuery["sort"]>>("relevance");
  let categories = $state<string[]>([]);
  let matchVersion = $state(true);
  let hits = $state<SearchHit[]>([]);
  let total = $state(0);
  let loading = $state(false);
  let error = $state("");
  let installing = $state<Record<string, boolean>>({});
  let installedIds = $state<Set<string>>(new Set());

  // Default target: the selected/last instance.
  $effect(() => {
    if (!app.discover.instanceId || !app.instances.some((i) => i.id === app.discover.instanceId)) {
      app.discover.instanceId = app.settings?.lastInstance && app.instances.some((i) => i.id === app.settings!.lastInstance) ? app.settings.lastInstance : (app.instances[0]?.id ?? null);
    }
  });
  const target = $derived(app.instances.find((i) => i.id === app.discover.instanceId) ?? null);

  let timer: ReturnType<typeof setTimeout> | undefined;
  let seq = 0;
  async function run(append = false) {
    const my = ++seq;
    loading = true;
    error = "";
    try {
      const r = await api.search({
        query,
        kind: app.discover.kind,
        gameVersion: matchVersion ? target?.mcVersion : null,
        loader: matchVersion && app.discover.kind === "mod" ? target?.loader : null,
        categories,
        sort,
        offset: append ? hits.length : 0,
      });
      if (my !== seq) return;
      hits = append ? [...hits, ...r.hits] : r.hits;
      total = r.totalHits;
    } catch (e) {
      if (my === seq) error = errText(e);
    } finally {
      if (my === seq) loading = false;
    }
  }

  // Re-run when any filter changes (debounced for typing).
  $effect(() => {
    void [query, sort, categories.join(), matchVersion, app.discover.kind, target?.id];
    clearTimeout(timer);
    timer = setTimeout(() => run(false), 280);
  });

  $effect(() => {
    void app.contentRev;
    const t = target;
    const kind = app.discover.kind;
    if (!t) return;
    api.listContent(t.id, kind).then((list) => (installedIds = new Set(list.map((c) => c.meta?.projectId).filter(Boolean) as string[]))).catch(() => {});
  });

  function setKind(k: ContentKind) {
    app.discover.kind = k;
    categories = [];
  }

  function toggleCat(c: string) {
    categories = categories.includes(c) ? categories.filter((x) => x !== c) : [...categories, c];
  }

  async function quickInstall(h: SearchHit) {
    if (!target) return toast("Najpierw utwórz instancję", "error");
    installing[h.projectId] = true;
    try {
      const done = await api.installContent(target.id, app.discover.kind, h.projectId);
      const deps = done.filter((d) => d.dependency).map((d) => d.title);
      toast(`Zainstalowano ${h.title}${deps.length ? ` + ${deps.join(", ")}` : ""}`, "success");
      app.contentRev++;
    } catch (e) {
      toastError(e);
    } finally {
      delete installing[h.projectId];
    }
  }

  const vanillaMods = $derived(app.discover.kind === "mod" && (target?.loader === "vanilla" || target?.loader === "rust"));
</script>

<div class="page">
  <header class="head">
    <h1>Przeglądaj</h1>
    <div class="target">
      <span class="dim">Instaluj do</span>
      <select class="input" bind:value={app.discover.instanceId}>
        {#each app.instances as i (i.id)}
          <option value={i.id}>{i.name} · {loaderLabel(i.loader)} {i.mcVersion}</option>
        {/each}
      </select>
    </div>
  </header>

  <div class="kinds">
    {#each ["mod", "shader", "resourcepack"] as const as k (k)}
      <button class="kind" class:on={app.discover.kind === k} onclick={() => setKind(k)}>{kindLabel(k)}</button>
    {/each}
  </div>

  <div class="bar">
    <div class="search">
      <Search size={16} />
      <input class="input" placeholder="Szukaj na Modrinth… (np. sodium, minimap, faithful)" bind:value={query} />
    </div>
    <select class="input sort" bind:value={sort}>
      {#each SORTS as [k, l] (k)}<option value={k}>{l}</option>{/each}
    </select>
    <label class="chk" title="Pokazuj tylko to, co działa z wybraną instancją">
      <input type="checkbox" bind:checked={matchVersion} /> Tylko zgodne {#if target}({target.mcVersion}){/if}
    </label>
  </div>

  <div class="cats">
    {#each CATEGORIES[app.discover.kind] as [k, l] (k)}
      <button class="chip cat" class:accent={categories.includes(k)} onclick={() => toggleCat(k)}>{l}</button>
    {/each}
  </div>

  {#if vanillaMods}
    <div class="warnbox card">Wybrana instancja nie obsługuje modów — wymagają Fabric albo Quilt. Zmień loader w ustawieniach instancji albo wybierz inną.</div>
  {/if}

  {#if error}
    <div class="warnbox card">{error}</div>
  {/if}

  <div class="results">
    {#each hits as h, n (h.projectId)}
      <div class="hit card" in:fly={{ y: 10, duration: dur(300), delay: dur(Math.min(n % 20, 10) * 25) }}>
        <button class="open" onclick={() => (app.project = { id: h.projectId, kind: app.discover.kind })}>
          {#if h.iconUrl}<img src={h.iconUrl} alt="" loading="lazy" />{:else}<span class="ph"></span>{/if}
          <span class="txt">
            <span class="t"><b>{h.title}</b> <small class="dim">od {h.author}</small></span>
            <span class="d">{h.description}</span>
            <span class="meta">
              <span><Download size={12} /> {compact(h.downloads)}</span>
              <span><Heart size={12} /> {compact(h.follows)}</span>
              <span>{relativeTime(Date.parse(h.dateModified) / 1000)}</span>
              {#each h.displayCategories.slice(0, 3) as c (c)}<span class="chip mini">{c}</span>{/each}
            </span>
          </span>
        </button>
        {#if installedIds.has(h.projectId)}
          <span class="chip accent inst"><Check size={13} /> Zainstalowany</span>
        {:else}
          <button class="btn primary inst" disabled={!target || installing[h.projectId] || vanillaMods} onclick={() => quickInstall(h)}>
            {#if installing[h.projectId]}<LoaderCircle size={15} class="spin" />{:else}<Download size={15} />{/if} Zainstaluj
          </button>
        {/if}
      </div>
    {:else}
      {#if !loading}<div class="empty dim">Nic nie znaleziono.</div>{/if}
    {/each}
  </div>

  <div class="more">
    {#if loading}
      <LoaderCircle size={20} class="spin" />
    {:else if hits.length < total}
      <button class="btn" onclick={() => run(true)}>Załaduj więcej ({total - hits.length})</button>
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
    gap: 12px;
    margin: 4px 0 14px;
  }
  h1 {
    font-size: 1.65rem;
    font-weight: 700;
  }
  .target {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 0.88rem;
  }
  .target select {
    width: 280px;
    height: 36px;
  }
  .kinds {
    display: flex;
    gap: 4px;
    padding: 4px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 11px;
    width: fit-content;
    margin-bottom: 12px;
  }
  .kind {
    padding: 7px 14px;
    border-radius: 8px;
    font-weight: 600;
    color: var(--text-2);
    transition: background 0.2s, color 0.2s;
  }
  .kind.on {
    background: var(--panel-3);
    color: var(--text);
  }
  .bar {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .search {
    flex: 1;
    position: relative;
    display: flex;
    align-items: center;
  }
  .search :global(svg) {
    position: absolute;
    left: 12px;
    color: var(--text-3);
  }
  .search .input {
    padding-left: 38px;
    height: 40px;
  }
  .sort {
    width: 160px;
    height: 40px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 0.86rem;
    color: var(--text-2);
    white-space: nowrap;
  }
  .chk input {
    accent-color: var(--accent);
  }
  .cats {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    margin: 12px 0 14px;
  }
  .cat {
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
  }
  .cat:hover {
    border-color: var(--border-2);
  }
  .warnbox {
    padding: 12px 16px;
    margin-bottom: 12px;
    color: var(--warning);
    border-color: color-mix(in srgb, var(--warning) 35%, var(--border));
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .hit {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px 10px 10px;
    border-radius: var(--radius);
    transition: border-color 0.2s, transform 0.2s var(--ease);
  }
  .hit:hover {
    border-color: var(--border-2);
  }
  .open {
    flex: 1;
    display: flex;
    gap: 14px;
    align-items: center;
    text-align: left;
    min-width: 0;
  }
  .open img,
  .ph {
    width: 58px;
    height: 58px;
    border-radius: 12px;
    object-fit: cover;
    background: var(--panel-2);
    flex-shrink: 0;
  }
  .txt {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .t b {
    font-weight: 650;
  }
  .t small {
    font-size: 0.8rem;
  }
  .d {
    color: var(--text-2);
    font-size: 0.86rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    display: flex;
    gap: 12px;
    align-items: center;
    color: var(--text-3);
    font-size: 0.78rem;
  }
  .meta > span:not(.chip) {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .mini {
    height: 19px;
    font-size: 0.7rem;
    padding: 0 7px;
  }
  .inst {
    flex-shrink: 0;
  }
  .empty {
    padding: 40px;
    text-align: center;
  }
  .more {
    display: grid;
    place-items: center;
    padding: 16px;
  }
</style>
