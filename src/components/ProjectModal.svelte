<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { fade } from "svelte/transition";
  import { Bug, Check, Code, Download, ExternalLink, Heart, LoaderCircle, MessageCircle, BookOpen, TriangleAlert } from "@lucide/svelte";
  import Modal from "./Modal.svelte";
  import { api, errText } from "../lib/api";
  import { compact, kindLabel, loaderLabel } from "../lib/format";
  import { renderMarkdown } from "../lib/markdown";
  import { app, dur, toast, toastError } from "../lib/state.svelte";
  import type { ContentItem, ModVersion, Project } from "../lib/types";

  const ctx = $derived(app.project!);
  let project = $state<Project | null>(null);
  let versions = $state<ModVersion[] | null>(null);
  let installed = $state<ContentItem[]>([]);
  let tab = $state<"about" | "gallery" | "versions">(app.project?.tab ?? "about");
  let allVersions = $state(false);
  let busy = $state<string | null>(null);
  let error = $state("");
  let versionsError = $state("");
  let lightbox = $state<string | null>(null);

  const targetId = $derived(app.discover.instanceId);
  const target = $derived(app.instances.find((i) => i.id === targetId) ?? null);
  const current = $derived(installed.find((c) => c.meta?.projectId === project?.id) ?? null);

  $effect(() => {
    const id = ctx.id;
    project = null;
    error = "";
    let stale = false;
    api.project(id).then((p) => !stale && (project = p)).catch((e) => !stale && (error = errText(e)));
    return () => (stale = true);
  });

  $effect(() => {
    const id = ctx.id;
    const kind = ctx.kind;
    const inst = targetId;
    const all = allVersions;
    versions = null;
    versionsError = "";
    let stale = false;
    api
      .versions(id, kind, inst, all)
      .then((v) => !stale && (versions = v))
      .catch((e) => {
        if (stale) return;
        versionsError = errText(e);
        versions = [];
      });
    return () => (stale = true);
  });

  async function loadInstalled() {
    if (targetId) installed = await api.listContent(targetId, ctx.kind).catch(() => []);
  }
  $effect(() => {
    void targetId;
    void app.contentRev;
    loadInstalled();
  });

  async function install(versionId: string | null) {
    if (!target) {
      toast("Wybierz instancję, do której chcesz zainstalować", "error");
      return;
    }
    busy = versionId ?? "best";
    try {
      const done = await api.installContent(target.id, ctx.kind, ctx.id, versionId);
      const deps = done.filter((d) => d.dependency).map((d) => d.title);
      toast(`Zainstalowano ${project?.title ?? ""} w „${target.name}”${deps.length ? ` (+ ${deps.join(", ")})` : ""}`, "success", 6000);
      app.contentRev++;
    } catch (e) {
      toastError(e);
    } finally {
      busy = null;
    }
  }

  function compatible(v: ModVersion) {
    return !target || v.gameVersions.includes(target.mcVersion);
  }

  // Links in the description open in the system browser.
  function onBodyClick(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest("a");
    if (a?.href?.startsWith("http")) {
      e.preventDefault();
      openUrl(a.href);
    }
  }

  const links = $derived(
    project
      ? ([
          [project.sourceUrl, "Kod", Code],
          [project.issuesUrl, "Zgłoszenia", Bug],
          [project.wikiUrl, "Wiki", BookOpen],
          [project.discordUrl, "Discord", MessageCircle],
          [`https://modrinth.com/${project.projectType}/${project.slug}`, "Modrinth", ExternalLink],
        ].filter(([u]) => u) as [string, string, typeof Code][])
      : [],
  );
</script>

<Modal onclose={() => (app.project = null)} width={860}>
  {#if error && !project}
    <p class="error">{error}</p>
  {:else if !project}
    <div class="loading"><LoaderCircle size={24} class="spin" /></div>
  {:else}
    <div in:fade={{ duration: dur(200) }}>
      <header class="head">
        {#if project.iconUrl}<img class="icon" src={project.iconUrl} alt="" />{:else}<div class="icon ph"></div>{/if}
        <div class="title">
          <h2>{project.title}</h2>
          <p class="muted">{project.description}</p>
          <div class="stats">
            <span><Download size={13} /> {compact(project.downloads)}</span>
            <span><Heart size={13} /> {compact(project.followers)}</span>
            <span class="chip">{kindLabel(ctx.kind, false)}</span>
            {#each project.categories.slice(0, 4) as c (c)}<span class="chip">{c}</span>{/each}
          </div>
        </div>
        <div class="cta">
          {#if current}
            <span class="chip accent"><Check size={13} /> {current.meta?.versionNumber}</span>
          {/if}
          <button class="btn primary" disabled={!!busy || !target} onclick={() => install(null)}>
            {#if busy === "best"}<LoaderCircle size={15} class="spin" />{:else}<Download size={15} />{/if}
            {current ? "Aktualizuj" : "Zainstaluj"}
          </button>
          <span class="dim small">{target ? `do: ${target.name}` : "brak instancji"}</span>
        </div>
      </header>

      <nav class="tabs">
        <button class:on={tab === "about"} onclick={() => (tab = "about")}>Opis</button>
        {#if project.gallery.length}<button class:on={tab === "gallery"} onclick={() => (tab = "gallery")}>Galeria ({project.gallery.length})</button>{/if}
        <button class:on={tab === "versions"} onclick={() => (tab = "versions")}>Wersje</button>
        <div class="links">
          {#each links as [url, label, Icon] (label)}
            <button class="btn ghost sm" onclick={() => openUrl(url)}><Icon size={14} /> {label}</button>
          {/each}
        </div>
      </nav>

      {#if tab === "about"}
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
        <article class="body selectable" onclick={onBodyClick}>{@html renderMarkdown(project.body)}</article>
      {:else if tab === "gallery"}
        <div class="gallery">
          {#each project.gallery as g (g.url)}
            <button class="shot" onclick={() => (lightbox = g.url)}><img src={g.url} alt={g.title ?? ""} loading="lazy" /></button>
          {/each}
        </div>
      {:else}
        <div class="vbar">
          <label class="chk"><input type="checkbox" bind:checked={allVersions} /> Pokaż wersje dla innych wersji gry</label>
          {#if target}<span class="dim small">{loaderLabel(target.loader)} {target.mcVersion}</span>{/if}
        </div>
        {#if versionsError}
          <p class="error">{versionsError}</p>
        {:else if !versions}
          <div class="loading"><LoaderCircle size={20} class="spin" /></div>
        {:else if versions.length === 0}
          <p class="dim">Brak wersji dla tej instancji. Zaznacz „inne wersje gry”, aby zobaczyć wszystkie.</p>
        {:else}
          <div class="versions">
            {#each versions as v (v.id)}
              {@const isCurrent = current?.meta?.versionId === v.id}
              <div class="ver" class:current={isCurrent}>
                <span class="type {v.versionType}">{v.versionType === "release" ? "R" : v.versionType === "beta" ? "B" : "A"}</span>
                <div class="vtext">
                  <b>{v.name}</b>
                  <small class="dim">{v.versionNumber} · {v.gameVersions.slice(-3).join(", ")}{v.gameVersions.length > 3 ? "…" : ""} · {new Date(v.datePublished).toLocaleDateString("pl-PL")} · {compact(v.downloads)}</small>
                </div>
                {#if !compatible(v)}<span class="warn" title="Inna wersja gry niż instancja"><TriangleAlert size={15} /></span>{/if}
                {#if isCurrent}
                  <span class="chip accent">Zainstalowana</span>
                {:else}
                  <button class="btn sm" disabled={!!busy || !target} onclick={() => install(v.id)}>
                    {#if busy === v.id}<LoaderCircle size={13} class="spin" />{/if}
                    {current ? "Zmień na tę" : "Zainstaluj"}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</Modal>

{#if lightbox}
  <button class="lightbox" onclick={() => (lightbox = null)} transition:fade={{ duration: dur(150) }}><img src={lightbox} alt="" /></button>
{/if}

<style>
  .loading {
    display: grid;
    place-items: center;
    padding: 60px;
  }
  .error {
    color: var(--danger);
  }
  .head {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }
  .icon {
    width: 76px;
    height: 76px;
    border-radius: 16px;
    object-fit: cover;
    background: var(--panel-2);
    flex-shrink: 0;
  }
  .title {
    flex: 1;
    min-width: 0;
  }
  .title h2 {
    font-size: 1.45rem;
  }
  .title p {
    margin: 4px 0 8px;
    line-height: 1.45;
  }
  .stats {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    color: var(--text-2);
    font-size: 0.85rem;
  }
  .stats > span:not(.chip) {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .cta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
  }
  .small {
    font-size: 0.78rem;
  }
  .tabs {
    display: flex;
    align-items: center;
    gap: 2px;
    margin: 18px 0 14px;
    border-bottom: 1px solid var(--border);
  }
  .tabs > button {
    padding: 8px 12px;
    font-weight: 600;
    color: var(--text-3);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
  }
  .tabs > button.on {
    color: var(--text);
    border-color: var(--accent);
  }
  .links {
    margin-left: auto;
    display: flex;
    gap: 2px;
  }
  .body {
    line-height: 1.6;
    color: var(--text-2);
    user-select: text;
    max-height: 56vh;
    overflow: auto;
    overflow-wrap: anywhere;
  }
  .body :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: 8px;
  }
  .body :global(h1),
  .body :global(h2),
  .body :global(h3) {
    color: var(--text);
    margin: 18px 0 8px;
  }
  .body :global(a) {
    color: var(--accent);
  }
  .body :global(code) {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--panel-2);
    padding: 1px 5px;
    border-radius: 4px;
  }
  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 10px;
    max-height: 56vh;
    overflow: auto;
  }
  .shot img {
    width: 100%;
    aspect-ratio: 16/9;
    object-fit: cover;
    border-radius: 10px;
    display: block;
    transition: transform 0.25s var(--ease);
  }
  .shot:hover img {
    transform: scale(1.02);
  }
  .vbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 0.88rem;
    color: var(--text-2);
  }
  .chk input {
    accent-color: var(--accent);
  }
  .versions {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 48vh;
    overflow: auto;
  }
  .ver {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 10px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid transparent;
  }
  .ver.current {
    border-color: var(--accent-line);
  }
  .type {
    width: 26px;
    height: 26px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    font-weight: 800;
    font-size: 0.75rem;
    flex-shrink: 0;
  }
  .type.release {
    background: color-mix(in srgb, var(--success) 18%, transparent);
    color: var(--success);
  }
  .type.beta {
    background: color-mix(in srgb, var(--warning) 18%, transparent);
    color: var(--warning);
  }
  .type.alpha {
    background: color-mix(in srgb, var(--danger) 18%, transparent);
    color: var(--danger);
  }
  .vtext {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .vtext b {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .vtext small {
    font-size: 0.78rem;
    margin-top: 2px;
  }
  .warn {
    color: var(--warning);
    display: grid;
  }
  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 70;
    background: rgb(0 0 0 / 0.85);
    display: grid;
    place-items: center;
    cursor: zoom-out;
  }
  .lightbox img {
    max-width: 92vw;
    max-height: 88vh;
    border-radius: 10px;
  }
</style>
