<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { fade } from "svelte/transition";
  import { Bug, Check, Code, Download, ExternalLink, Heart, LoaderCircle, MessageCircle, BookOpen, TriangleAlert } from "@lucide/svelte";
  import Modal from "./Modal.svelte";
  import Option from "./Option.svelte";
  import Segmented from "./Segmented.svelte";
  import { api, errText } from "../lib/api";
  import { categoryLabel } from "../lib/categories";
  import { compact, kindLabel, loaderLabel } from "../lib/format";
  import { locale, t } from "../lib/i18n.svelte";
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
      toast(t("project.pickInstance"), "error");
      return;
    }
    busy = versionId ?? "best";
    try {
      const done = await api.installContent(target.id, ctx.kind, ctx.id, versionId);
      const deps = done.filter((d) => d.dependency).map((d) => d.title);
      const params = { title: project?.title ?? "", name: target.name, deps: deps.join(", ") };
      toast(t(deps.length ? "project.installedWith" : "project.installed", params), "success", 6000);
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
      openUrl(a.href).catch(toastError);
    }
  }

  const links = $derived(
    project
      ? ([
          [project.sourceUrl, t("project.source"), Code],
          [project.issuesUrl, t("project.issues"), Bug],
          [project.wikiUrl, "Wiki", BookOpen],
          [project.discordUrl, "Discord", MessageCircle],
          [`https://modrinth.com/${project.projectType}/${project.slug}`, "Modrinth", ExternalLink],
        ].filter(([u]) => u) as [string, string, typeof Code][])
      : [],
  );

  const tabs = $derived([
    { value: "about" as const, label: t("project.about") },
    ...(project?.gallery.length ? [{ value: "gallery" as const, label: t("project.gallery", { n: project.gallery.length }) }] : []),
    { value: "versions" as const, label: t("project.versions") },
  ]);
  const channel = (type: string) => (type === "beta" ? "Beta · " : type === "alpha" ? "Alpha · " : "");

  // Runs before the dialog's own handler, so Escape leaves the picture, not the whole page.
  function closeLightbox(e: KeyboardEvent) {
    if (lightbox && e.key === "Escape") {
      e.stopPropagation();
      lightbox = null;
    }
  }
</script>

<svelte:window onkeydowncapture={closeLightbox} />

<Modal onclose={() => (app.project = null)} width={860}>
  <div class="project">
    {#if error && !project}
      <p class="error">{error}</p>
    {:else if !project}
      <div class="loading"><LoaderCircle size={20} class="spin" /></div>
    {:else}
      <header class="head">
        {#if project.iconUrl}<img class="icon" src={project.iconUrl} alt="" />{:else}<div class="icon"></div>{/if}
        <div class="title">
          <h3>{project.title}</h3>
          <p>{project.description}</p>
          <div class="stats">
            <span><Download size={12} /> {compact(project.downloads)}</span>
            <span><Heart size={12} /> {compact(project.followers)}</span>
            <span>{[kindLabel(ctx.kind, false), ...project.categories.slice(0, 4).map(categoryLabel)].join(" · ")}</span>
          </div>
        </div>
        <div class="cta">
          <button class="btn primary" disabled={!!busy || !target} onclick={() => install(null)}>
            {#if busy === "best"}<LoaderCircle size={14} class="spin" />{:else}<Download size={14} />{/if}
            {t(current ? "project.update" : "common.install")}
          </button>
          {#if current}<span class="small"><Check size={12} /> {current.meta?.versionNumber}</span>{/if}
          <span class="small">{target ? t("project.into", { name: target.name }) : t("project.noInstance")}</span>
        </div>
      </header>

      <nav class="tabs">
        <Segmented label={t("project.sections")} value={tab} options={tabs} onchange={(v) => (tab = v)} />
        <div class="links">
          {#each links as [url, label, Icon] (label)}
            <button class="btn ghost sm" onclick={() => openUrl(url).catch(toastError)}><Icon size={14} /> {label}</button>
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
          <Option bind:checked={allVersions} label={t("project.allVersions")} />
          {#if target}<span class="small">{loaderLabel(target.loader)} {target.mcVersion}</span>{/if}
        </div>
        {#if versionsError}
          <p class="error">{versionsError}</p>
        {:else if !versions}
          <div class="loading"><LoaderCircle size={16} class="spin" /></div>
        {:else if versions.length === 0}
          <p class="none">{t("project.noVersions")}</p>
        {:else}
          <div class="versions">
            {#each versions as v (v.id)}
              {@const isCurrent = current?.meta?.versionId === v.id}
              <div class="ver">
                <div class="vtext">
                  <b>{v.name}</b>
                  <small>{channel(v.versionType)}{v.versionNumber} · {v.gameVersions.slice(-3).join(", ")}{v.gameVersions.length > 3 ? "…" : ""} · {new Date(v.datePublished).toLocaleDateString(locale())} · {compact(v.downloads)}</small>
                </div>
                {#if !compatible(v)}<span class="warn" title={t("project.otherVersion")}><TriangleAlert size={14} /></span>{/if}
                {#if isCurrent}
                  <span class="small">{t("project.isInstalled")}</span>
                {:else}
                  <button class="btn sm" disabled={!!busy || !target} onclick={() => install(v.id)}>
                    {#if busy === v.id}<LoaderCircle size={14} class="spin" />{/if}
                    {t(current ? "project.switchTo" : "common.install")}
                  </button>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      {/if}
    {/if}
  </div>
</Modal>

{#if lightbox}
  <button class="lightbox" aria-label={t("common.close")} onclick={() => (lightbox = null)} transition:fade={{ duration: dur(120) }}><img src={lightbox} alt="" /></button>
{/if}

<style>
  /* Tabs differ in height: a steady floor keeps the dialog from jumping between them. */
  .project { min-height: min(480px, calc(100vh - 96px)); }
  .loading {
    display: grid;
    place-items: center;
    padding: 48px;
    color: var(--text-3);
  }
  .error { margin: 0; color: var(--danger); font-size: var(--fs-13); }
  .none { margin: 0; color: var(--text-3); font-size: var(--fs-13); }
  .head {
    display: flex;
    align-items: flex-start;
    gap: 16px;
  }
  .icon {
    width: 64px;
    height: 64px;
    flex-shrink: 0;
    border-radius: var(--radius);
    object-fit: cover;
    background: var(--panel-2);
  }
  .title { flex: 1; min-width: 0; }
  .title h3 { font-family: var(--font-display); font-size: var(--fs-20); }
  .title p {
    margin: 2px 0 8px;
    color: var(--text-2);
    font-size: var(--fs-13);
    line-height: 1.4;
  }
  .stats {
    display: flex;
    align-items: center;
    gap: 4px 12px;
    flex-wrap: wrap;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .stats span { display: inline-flex; align-items: center; gap: 4px; }
  .cta {
    flex-shrink: 0;
    max-width: 200px;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
  }
  .small {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    max-width: 100%;
    color: var(--text-3);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tabs {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
    flex-wrap: wrap;
    margin: 16px 0 12px;
  }
  .links { display: flex; flex-wrap: wrap; gap: 4px; }
  .body {
    color: var(--text-2);
    font-size: var(--fs-13);
    line-height: 1.6;
    user-select: text;
    overflow-wrap: anywhere;
  }
  .body :global(img) { max-width: 100%; height: auto; border-radius: var(--radius-s); }
  .body :global(h1),
  .body :global(h2),
  .body :global(h3) { margin: 16px 0 8px; color: var(--text); font-size: var(--fs-14); }
  .body :global(a) { color: var(--accent-strong); }
  .body :global(code) {
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--panel-2);
    font-family: var(--font-mono);
    font-size: 0.9em;
  }
  .gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 8px;
  }
  .shot img {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    border-radius: var(--radius-s);
  }
  .vbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 8px;
  }
  .versions { border-top: 1px solid var(--border); }
  .ver {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 48px;
    border-bottom: 1px solid var(--border);
  }
  .ver:last-child { border-bottom: 0; }
  .vtext {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .vtext b {
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .vtext small {
    color: var(--text-3);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .warn { display: grid; color: var(--warning); }
  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 70;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.85);
    cursor: zoom-out;
  }
  .lightbox img {
    max-width: 92vw;
    max-height: 88vh;
    border-radius: var(--radius-s);
  }
</style>
