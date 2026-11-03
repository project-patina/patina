<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { ArrowUpCircle, Compass, FilePlus, FolderOpen, Gauge, LoaderCircle, Package, RefreshCw, Replace, Trash2 } from "@lucide/svelte";
  import SearchInput from "./SearchInput.svelte";
  import Segmented from "./Segmented.svelte";
  import Toggle from "./Toggle.svelte";
  import { api } from "../lib/api";
  import { bytes, kindLabel } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import type { Key } from "../lib/locales/pl";
  import { ENGINE_MC } from "../lib/presets";
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
  // Put back at every launch (`patina_mod.rs`), so it has no switch and no delete button.
  const builtIn = (m: ContentItem) => kind === "mod" && m.file === "patina-client.jar" && inst.loader === "fabric" && inst.mcVersion === ENGINE_MC;
  const modsBlocked = $derived(kind === "mod" && (inst.loader === "vanilla" || inst.loader === "rust"));
  const kinds = $derived((["mod", "shader", "resourcepack"] as const).map((value) => ({ value, label: kindLabel(value) })));
  const EMPTY: Record<ContentKind, Key> = { mod: "content.emptyMods", shader: "content.emptyShaders", resourcepack: "content.emptyPacks" };

  async function checkUpdates() {
    checking = true;
    const k = kind;
    try {
      const list = await api.checkUpdates(inst.id, k);
      if (k !== kind) return;
      updates = Object.fromEntries(list.map((u) => [u.file, { ...u, kind: k }]));
      toast(list.length ? t("content.updatesFound", { n: list.length }) : t("content.upToDate"), list.length ? "info" : "success");
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
      return true;
    } catch (e) {
      toastError(e);
      return false;
    } finally {
      delete busy[u.file];
    }
  }

  async function updateAll() {
    let failed = 0;
    for (const u of Object.values(updates)) if (!(await update(u))) failed++;
    if (!failed) toast(t("content.updated"), "success");
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
      toast(t("content.removed", { name: m.meta?.title ?? m.file }));
    } catch (e) {
      toastError(e);
    }
  }

  async function fromFile() {
    const picked = await open({ multiple: true, title: t("content.addFiles", { kind: kindLabel(kind).toLowerCase() }), filters: [{ name: kindLabel(kind), extensions: [kind === "mod" ? "jar" : "zip"] }] });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    if (!paths.length) return;
    try {
      const names = await api.importContent(inst.id, kind, paths);
      toast(t("content.added", { names: names.join(", ") }), "success");
      await load(true);
    } catch (e) {
      toastError(e);
    }
  }

  async function perfMods() {
    perf = true;
    try {
      const names = await api.installPerformanceMods(inst.id);
      toast(names.length ? t("content.perfInstalled", { names: names.join(", ") }) : t("content.perfNothing"), "success", 6000);
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

<div class="bar">
  <Segmented label={t("content.kind")} value={kind} options={kinds} onchange={(k) => (kind = k)} />
  <span class="count">{t("content.enabledCount", { on: items.filter((m) => m.enabled).length, all: items.length })}</span>
  <button class="btn sm push" onclick={() => openDiscover(kind, inst.id)} disabled={modsBlocked}><Compass size={14} /> {t("content.browse")}</button>
</div>

<div class="bar">
  <SearchInput bind:value={query} placeholder={t("common.search")} width="200px" />
  <button class="btn sm icon push" onclick={() => load(true)} title={t("common.refresh")} aria-label={t("common.refresh")}><RefreshCw size={14} /></button>
  <button class="btn sm icon" title={t("content.openFolder")} aria-label={t("content.openFolder")} onclick={() => api.openFolder(inst.id, kind === "mod" ? "mods" : kind === "shader" ? "shaderpacks" : "resourcepacks").catch(toastError)}><FolderOpen size={14} /></button>
  <button class="btn sm" onclick={fromFile} disabled={modsBlocked}><FilePlus size={14} /> {t("content.fromFile")}</button>
  {#if kind === "mod" && !modsBlocked}
    <button class="btn sm" disabled={perf} onclick={perfMods}><Gauge size={14} /> {t("content.perfMods")}</button>
  {/if}
  <button class="btn sm" onclick={checkUpdates} disabled={checking || !items.length}>
    {#if checking}<LoaderCircle size={14} class="spin" />{:else}<ArrowUpCircle size={14} />{/if} {t("content.checkUpdates")}
  </button>
  {#if updateCount}
    <button class="btn sm primary" onclick={updateAll}>{t("content.updateAll", { n: updateCount })}</button>
  {/if}
</div>

{#if modsBlocked}
  <p class="empty">{t(inst.loader === "rust" ? "content.blockedRust" : "content.blockedVanilla")}</p>
{:else if loading && !items.length}
  <div class="empty"><LoaderCircle size={16} class="spin" /></div>
{:else if shown.length === 0}
  <div class="empty">
    <p>{items.length ? t("common.noResults") : t(EMPTY[kind])}</p>
    {#if !items.length}<button class="btn primary" onclick={() => openDiscover(kind, inst.id)}><Compass size={14} /> {t("content.browse")}</button>{/if}
  </div>
{:else}
  <div class="list">
    {#each shown as m (m.file)}
      {@const up = updates[m.file]}
      <div class="row" class:off={!m.enabled}>
        <Toggle label={m.meta?.title ?? m.file} checked={m.enabled} disabled={builtIn(m)} onchange={(v) => toggle(m, v)} />
        {#if m.meta?.iconUrl}<img src={m.meta.iconUrl} alt="" loading="lazy" />{:else}<span class="ph"><Package size={14} /></span>{/if}
        <button class="info" onclick={() => m.meta && changeVersion(m, "about")} disabled={!m.meta}>
          <b>{builtIn(m) ? "Patina Client" : (m.meta?.title ?? m.file)}</b>
          <small class="mono">{builtIn(m) ? t("content.builtIn") : m.meta ? m.meta.versionNumber : t("content.localFile")} · {bytes(m.size)}</small>
        </button>
        {#if up}
          <button class="btn sm" disabled={busy[m.file]} onclick={() => update(up)} title={t("content.updateTo", { version: up.versionNumber })}>
            {#if busy[m.file]}<LoaderCircle size={14} class="spin" />{:else}<ArrowUpCircle size={14} />{/if} <span class="to">{up.versionNumber}</span>
          </button>
        {/if}
        {#if m.meta}
          <button class="btn ghost sm icon" title={t("content.changeVersion")} aria-label={t("content.changeVersion")} onclick={() => changeVersion(m)}><Replace size={14} /></button>
        {/if}
        {#if !builtIn(m)}
          <button class="btn ghost sm icon danger" title={t("common.delete")} aria-label={t("common.delete")} onclick={() => remove(m)}><Trash2 size={14} /></button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .count {
    margin-left: 4px;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .push { margin-left: auto; }
  .list { border-top: 1px solid var(--border); margin-top: 4px; }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child { border-bottom: 0; }
  .row img,
  .ph {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    object-fit: cover;
    background: var(--panel-2);
    color: var(--text-3);
  }
  .row.off img,
  .row.off .info { opacity: 0.5; }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    text-align: left;
  }
  .info:disabled { cursor: default; }
  .info b {
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .info small {
    color: var(--text-3);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .to {
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 32px 16px;
    color: var(--text-3);
    font-size: var(--fs-13);
    line-height: 1.5;
    text-align: center;
  }
  .empty p { margin: 0; }
</style>
