<script lang="ts">
  import { onDestroy } from "svelte";
  import { Check, Download, Heart, LoaderCircle } from "@lucide/svelte";
  import Option from "../components/Option.svelte";
  import SearchInput from "../components/SearchInput.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Select from "../components/Select.svelte";
  import { api, errText } from "../lib/api";
  import { CATEGORIES, categoryLabel } from "../lib/categories";
  import { compact, kindLabel, loaderLabel, relativeTime } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { reveal } from "../lib/motion";
  import { app, toast, toastError } from "../lib/state.svelte";
  import type { ContentKind, SearchHit, SearchQuery } from "../lib/types";

  type Sort = NonNullable<SearchQuery["sort"]>;
  const sorts: { value: Sort; label: string }[] = $derived([
    { value: "relevance", label: t("discover.sortRelevance") },
    { value: "downloads", label: t("discover.sortDownloads") },
    { value: "follows", label: t("discover.sortFollows") },
    { value: "newest", label: t("discover.sortNewest") },
    { value: "updated", label: t("discover.sortUpdated") },
  ]);
  const kinds = $derived((["mod", "shader", "resourcepack"] as const).map((value) => ({ value, label: kindLabel(value) })));
  const targets = $derived(app.instances.map((i) => ({ value: i.id, label: `${i.name} · ${loaderLabel(i.loader)} ${i.mcVersion}` })));

  let query = $state("");
  let sort = $state<Sort>("relevance");
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
  onDestroy(() => { clearTimeout(timer); seq++; });
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
    const instance = target;
    const kind = app.discover.kind;
    if (!instance) return;
    let stale = false;
    api.listContent(instance.id, kind)
      .then((list) => { if (!stale) installedIds = new Set(list.map((c) => c.meta?.projectId).filter(Boolean) as string[]); })
      .catch(() => {});
    return () => { stale = true; };
  });

  function setKind(k: ContentKind) {
    app.discover.kind = k;
    categories = [];
  }

  function toggleCat(c: string) {
    categories = categories.includes(c) ? categories.filter((x) => x !== c) : [...categories, c];
  }

  async function quickInstall(h: SearchHit) {
    if (!target) return toast(t("discover.createFirst"), "error");
    installing[h.projectId] = true;
    try {
      const done = await api.installContent(target.id, app.discover.kind, h.projectId);
      const deps = done.filter((d) => d.dependency).map((d) => d.title);
      toast(deps.length ? t("discover.installedWith", { title: h.title, deps: deps.join(", ") }) : t("discover.installed", { title: h.title }), "success");
      app.contentRev++;
    } catch (e) {
      toastError(e);
    } finally {
      delete installing[h.projectId];
    }
  }

  const vanillaMods = $derived(app.discover.kind === "mod" && (target?.loader === "vanilla" || target?.loader === "rust"));
  const onlyLabel = $derived(target ? t("discover.compatibleWith", { version: target.mcVersion }) : t("discover.compatible"));
</script>

<div class="view">
  <header class="head">
    <h1 class="page-title">{t("nav.discover")}</h1>
    <div class="target">
      <span class="dim">{t("discover.installTo")}</span>
      <Select label={t("discover.installTo")} width={260} value={app.discover.instanceId ?? ""} options={targets} onchange={(id) => (app.discover.instanceId = id)} />
    </div>
  </header>

  <div class="filters">
    <Segmented label={t("discover.kind")} value={app.discover.kind} options={kinds} onchange={setKind} />
    <Option bind:checked={matchVersion} label={onlyLabel} title={t("discover.compatibleHint")} />
  </div>

  <div class="filters">
    <div class="grow"><SearchInput bind:value={query} placeholder={t("discover.search")} width="100%" /></div>
    <Select label={t("discover.sort")} width={160} value={sort} options={sorts} onchange={(v) => (sort = v)} />
  </div>

  <div class="cats" role="group" aria-label={t("discover.categories")}>
    {#each CATEGORIES[app.discover.kind] as tag (tag)}
      <button class="cat" aria-pressed={categories.includes(tag)} onclick={() => toggleCat(tag)}>{categoryLabel(tag)}</button>
    {/each}
  </div>

  {#if vanillaMods}
    <p class="notice">{t("discover.noMods")}</p>
  {/if}

  {#if error}
    <p class="notice">{error}</p>
  {/if}

  <div class="results">
    {#each hits as h, n (h.projectId)}
      <div class="hit" in:reveal|global={{ index: n }}>
        <button class="open" onclick={() => (app.project = { id: h.projectId, kind: app.discover.kind })}>
          {#if h.iconUrl}<img src={h.iconUrl} alt="" loading="lazy" />{:else}<span class="ph"></span>{/if}
          <span class="txt">
            <span class="name"><b>{h.title}</b> <small>{t("discover.by", { author: h.author })}</small></span>
            <span class="desc">{h.description}</span>
            <span class="meta">
              <span><Download size={12} /> {compact(h.downloads)}</span>
              <span><Heart size={12} /> {compact(h.follows)}</span>
              <span>{relativeTime(Date.parse(h.dateModified) / 1000)}</span>
              {#if h.displayCategories.length}<span class="tags">{h.displayCategories.slice(0, 3).map(categoryLabel).join(" · ")}</span>{/if}
            </span>
          </span>
        </button>
        {#if installedIds.has(h.projectId)}
          <span class="done"><Check size={14} /> {t("discover.isInstalled")}</span>
        {:else}
          <button class="btn inst" disabled={!target || installing[h.projectId] || vanillaMods} onclick={() => quickInstall(h)}>
            {#if installing[h.projectId]}<LoaderCircle size={14} class="spin" />{:else}<Download size={14} />{/if} {t("common.install")}
          </button>
        {/if}
      </div>
    {:else}
      {#if !loading}<p class="empty">{t("discover.nothing")}</p>{/if}
    {/each}
  </div>

  <div class="more">
    {#if loading}
      <LoaderCircle size={16} class="spin" />
    {:else if hits.length < total}
      <button class="btn" onclick={() => run(true)}>{t("discover.more", { n: total - hits.length })}</button>
    {/if}
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
    flex-wrap: wrap;
    margin-bottom: 16px;
  }
  .target {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    font-size: var(--fs-13);
    white-space: nowrap;
  }
  .filters {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .grow { flex: 1; min-width: 200px; }
  .cats {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin: 12px 0 16px;
  }
  .cat {
    height: 24px;
    padding: 0 8px;
    border-radius: var(--radius-s);
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
    transition: background 0.12s, color 0.12s;
  }
  .cat:hover { background: var(--panel-2); color: var(--text); }
  .cat[aria-pressed="true"] { background: var(--accent-soft); color: var(--text); }
  .notice {
    margin: 0 0 8px;
    color: var(--warning);
    font-size: var(--fs-13);
    line-height: 1.4;
  }
  .results { border-top: 1px solid var(--border); }
  .hit {
    display: flex;
    align-items: center;
    gap: 16px;
    border-bottom: 1px solid var(--border);
  }
  .open {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 0;
    text-align: left;
  }
  .open img,
  .ph {
    width: 48px;
    height: 48px;
    flex-shrink: 0;
    border-radius: var(--radius);
    object-fit: cover;
    background: var(--panel-2);
  }
  .txt {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name, .desc, .meta { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name b { font-size: var(--fs-14); font-weight: 500; }
  .name small { color: var(--text-3); font-size: var(--fs-12); }
  .open:hover b { color: var(--accent-strong); }
  .desc { color: var(--text-2); font-size: var(--fs-13); }
  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .meta > span { display: inline-flex; align-items: center; gap: 4px; flex-shrink: 0; }
  .meta .tags { display: block; flex-shrink: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .inst { flex-shrink: 0; }
  .done {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-2);
    font-size: var(--fs-13);
  }
  .empty {
    margin: 0;
    padding: 32px 0;
    color: var(--text-3);
    font-size: var(--fs-13);
    text-align: center;
  }
  .more {
    display: grid;
    place-items: center;
    min-height: 32px;
    padding-top: 16px;
    color: var(--text-3);
  }
</style>
