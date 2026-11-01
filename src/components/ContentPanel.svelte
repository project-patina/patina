<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { ArrowUpCircle, Compass, FilePlus, FolderOpen, LoaderCircle, Package, RefreshCw, Replace, Sparkles, Trash2 } from "@lucide/svelte";
  import Toggle from "./Toggle.svelte";
  import { api } from "../lib/api";
  import { bytes, kindLabel } from "../lib/format";
  import { app, openDiscover, toast, toastError } from "../lib/state.svelte";
  import type { ContentItem, ContentKind, ContentUpdate, Instance } from "../lib/types";

  let { inst }: { inst: Instance } = $props();

  let kind = $state<ContentKind>("mod");
  let items = $state<ContentItem[]>([]);
  // Each update remembers its kind: the tab may change while updates run.
  let updates = $state<Record<string, ContentUpdate & { kind: ContentKind }>>({});
  let query = $state("");
  let loading = $state(false);
  let checking = $state(false);
  let busy = $state<Record<string, boolean>>({});
  let perf = $state(false);

  let seq = 0;
  async function load(identify = true) {
    const my = ++seq;
    loading = true;
    try {
      const list = await api.listContent(inst.id, kind, identify);
      // A slower, older request (another tab) must not overwrite this one.
      if (my === seq) items = list;
    } catch (e) {
      if (my === seq) toastError(e);
    } finally {
      if (my === seq) loading = false;
    }
  }

  $effect(() => {
    void [inst.id, kind, app.contentRev];
    updates = {};
    load(true);
  });
  // The first launch installs the performance mods.
  $effect(() => {
    if (app.running[inst.id] === "running") load(false);
  });

  const shown = $derived(items.filter((m) => !query || (m.meta?.title ?? m.file).toLowerCase().includes(query.toLowerCase())));
  const updateCount = $derived(Object.keys(updates).length);
  const modsBlocked = $derived(kind === "mod" && (inst.loader === "vanilla" || inst.loader === "rust"));

  async function checkUpdates() {
    checking = true;
    const k = kind;
    try {
      const list = await api.checkUpdates(inst.id, k);
      if (k !== kind) return;
      updates = Object.fromEntries(list.map((u) => [u.file, { ...u, kind: k }]));
      toast(list.length ? `Dostępne aktualizacje: ${list.length}` : "Wszystko aktualne", list.length ? "info" : "success");
    } catch (e) {
      toastError(e);
    } finally {
      checking = false;
    }
  }

  async function update(u: ContentUpdate & { kind: ContentKind }) {
    busy[u.file] = true;
    try {
      await api.installContent(inst.id, u.kind, u.projectId, u.versionId);
      delete updates[u.file];
      if (u.kind === kind) await load(false);
    } catch (e) {
      toastError(e);
    } finally {
      delete busy[u.file];
    }
  }

  async function updateAll() {
    for (const u of Object.values(updates)) await update(u);
    toast("Zaktualizowano", "success");
  }

  async function toggle(m: ContentItem, on: boolean) {
    try {
      await api.setContentEnabled(inst.id, kind, m.file, on);
      m.enabled = on;
    } catch (e) {
      toastError(e);
      await load(false);
    }
  }

  async function remove(m: ContentItem) {
    try {
      await api.deleteContent(inst.id, kind, m.file);
      items = items.filter((x) => x.file !== m.file);
      toast(`Usunięto ${m.meta?.title ?? m.file} (do kosza)`);
    } catch (e) {
      toastError(e);
    }
  }

  async function fromFile() {
    const picked = await open({ multiple: true, title: `Dodaj: ${kindLabel(kind)}`, filters: [{ name: kindLabel(kind), extensions: [kind === "mod" ? "jar" : "zip"] }] });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (!paths.length) return;
    try {
      const names = await api.importContent(inst.id, kind, paths);
      toast(`Dodano: ${names.join(", ")}`, "success");
      await load(true);
    } catch (e) {
      toastError(e);
    }
  }

  async function perfMods() {
    perf = true;
    try {
      const names = await api.installPerformanceMods(inst.id);
      toast(names.length ? `Zainstalowano: ${names.join(", ")}` : "Wszystkie mody już są albo nie ma ich dla tej wersji", "success", 6000);
      await load(true);
    } catch (e) {
      toastError(e);
    } finally {
      perf = false;
      delete app.progress[inst.id];
    }
  }

  // The project page installs into app.discover.instanceId: point it here.
  function changeVersion(m: ContentItem, tab: "about" | "versions" = "versions") {
    if (!m.meta) return;
    app.discover.instanceId = inst.id;
    app.project = { id: m.meta.projectId, kind, tab };
  }
</script>

<div class="kinds">
  {#each ["mod", "shader", "resourcepack"] as const as k (k)}
    <button class="kind" class:on={kind === k} onclick={() => (kind = k)}>{kindLabel(k)}</button>
  {/each}
</div>

<div class="bar">
  <input class="input" placeholder="Szukaj…" bind:value={query} />
  <span class="dim count">{items.filter((m) => m.enabled).length}/{items.length} włączonych</span>
  <div class="spacer"></div>
  <button class="btn sm" onclick={() => load(true)} title="Odśwież"><RefreshCw size={14} /></button>
  <button class="btn sm" onclick={checkUpdates} disabled={checking || !items.length}>
    {#if checking}<LoaderCircle size={14} class="spin" />{:else}<ArrowUpCircle size={14} />{/if} Aktualizacje
  </button>
  {#if updateCount}
    <button class="btn sm primary" onclick={updateAll}>Aktualizuj wszystko ({updateCount})</button>
  {/if}
  <button class="btn sm" onclick={fromFile} disabled={modsBlocked}><FilePlus size={14} /> Z pliku</button>
  <button class="btn sm" onclick={() => api.openFolder(inst.id, kind === "mod" ? "mods" : kind === "shader" ? "shaderpacks" : "resourcepacks").catch(toastError)}><FolderOpen size={14} /></button>
  {#if kind === "mod" && !modsBlocked}
    <button class="btn sm" disabled={perf} onclick={perfMods}><Sparkles size={14} /> Mody na FPS</button>
  {/if}
  <button class="btn sm primary" onclick={() => openDiscover(kind, inst.id)} disabled={modsBlocked}><Compass size={14} /> Przeglądaj</button>
</div>

{#if modsBlocked}
  <div class="empty card">{inst.loader === "rust" ? "Silnik Rust nie uruchamia modów Fabric." : "Vanilla nie obsługuje modów."} Zmień loader na Fabric w ustawieniach instancji, aby je dodać. Paczki zasobów działają.</div>
{:else if loading && !items.length}
  <div class="empty card"><LoaderCircle size={22} class="spin" /></div>
{:else if shown.length === 0}
  <div class="empty card">
    <Package size={28} />
    <p>{items.length ? "Brak wyników." : `Brak: ${kindLabel(kind).toLowerCase()}. Znajdź coś w przeglądarce albo dodaj plik.`}</p>
    <button class="btn primary" onclick={() => openDiscover(kind, inst.id)}><Compass size={15} /> Przeglądaj {kindLabel(kind).toLowerCase()}</button>
  </div>
{:else}
  <div class="list card">
    {#each shown as m (m.file)}
      {@const up = updates[m.file]}
      <div class="row" class:off={!m.enabled}>
        <Toggle checked={m.enabled} onchange={(v) => toggle(m, v)} />
        {#if m.meta?.iconUrl}<img src={m.meta.iconUrl} alt="" loading="lazy" />{:else}<span class="ph"><Package size={16} /></span>{/if}
        <button class="info" onclick={() => m.meta && changeVersion(m, "about")} disabled={!m.meta}>
          <b>{m.meta?.title ?? m.file}</b>
          <small class="dim mono">{m.meta ? m.meta.versionNumber : "plik lokalny"} · {bytes(m.size)}</small>
        </button>
        {#if up}
          <button class="btn sm primary" disabled={busy[m.file]} onclick={() => update(up)} title="Aktualizuj do {up.versionNumber}">
            {#if busy[m.file]}<LoaderCircle size={13} class="spin" />{:else}<ArrowUpCircle size={13} />{/if} {up.versionNumber}
          </button>
        {/if}
        {#if m.meta}
          <button class="btn ghost sm icon" title="Zmień wersję" onclick={() => changeVersion(m)}><Replace size={14} /></button>
        {/if}
        <button class="btn ghost sm icon danger" title="Usuń" onclick={() => remove(m)}><Trash2 size={14} /></button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .kinds {
    display: flex;
    gap: 4px;
    margin-bottom: 10px;
  }
  .kind {
    padding: 6px 12px;
    border-radius: 8px;
    font-weight: 600;
    font-size: 0.88rem;
    color: var(--text-2);
    border: 1px solid var(--border);
    background: var(--panel);
  }
  .kind.on {
    color: var(--text);
    border-color: var(--accent-line);
    background: var(--accent-soft);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .bar .input {
    height: 32px;
    max-width: 240px;
  }
  .count {
    font-size: 0.85rem;
  }
  .spacer {
    flex: 1;
  }
  .list {
    padding: 4px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 10px;
    border-radius: 9px;
    transition: background 0.15s;
  }
  .row:hover {
    background: var(--panel-2);
  }
  .row img,
  .ph {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    object-fit: cover;
    flex-shrink: 0;
    background: var(--panel-3);
    display: grid;
    place-items: center;
    color: var(--text-3);
  }
  .row.off img,
  .row.off .info {
    opacity: 0.45;
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    text-align: left;
  }
  .info b {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .info small {
    font-size: 0.76rem;
    margin-top: 1px;
  }
  .empty {
    padding: 34px;
    text-align: center;
    color: var(--text-2);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .empty p {
    margin: 0;
    max-width: 460px;
    line-height: 1.5;
  }
</style>
