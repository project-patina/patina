<script lang="ts">
  import { onMount } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { X } from "@lucide/svelte";
  import Modal from "./Modal.svelte";
  import { longDate } from "../lib/format";
  import { lang, t } from "../lib/i18n.svelte";
  import { renderAnnouncementMarkdown } from "../lib/markdown";
  import { reveal } from "../lib/motion";
  import { app, loadAnnouncements, toastError } from "../lib/state.svelte";
  import type { Announcement } from "../lib/types";

  const FALLBACK_IMAGE = "/announcements/first-version.png";
  const storageKey = "patina.read-announcements.v1";
  let read = $state<string[]>([]);
  let selectedId = $state<string | null>(null);
  const items = $derived(app.announcements?.items ?? []);
  // By id, so an open article follows a language change.
  const selected = $derived(items.find((item) => item.id === selectedId) ?? null);
  const key = (item: Announcement) => `${item.id}:${item.date}`;

  onMount(() => {
    try {
      const saved: unknown = JSON.parse(localStorage.getItem(storageKey) ?? "[]");
      if (Array.isArray(saved)) read = saved.filter((id): id is string => typeof id === "string").slice(-100);
    } catch { /* Reading still works when browser storage is unavailable. */ }
  });

  $effect(() => {
    void loadAnnouncements(lang());
  });

  function show(item: Announcement) {
    selectedId = item.id;
    if (!read.includes(key(item))) {
      read = [...read, key(item)].slice(-100);
      try { localStorage.setItem(storageKey, JSON.stringify(read)); } catch { /* Session-only read state. */ }
    }
  }

  function fallback(event: Event) {
    const image = event.currentTarget as HTMLImageElement;
    if (!image.dataset.fallback) { image.dataset.fallback = "1"; image.src = FALLBACK_IMAGE; }
  }

  function openLink(event: MouseEvent) {
    const anchor = (event.target as Element).closest("a");
    if (!anchor) return;
    event.preventDefault();
    try {
      const url = new URL(anchor.getAttribute("href") ?? "");
      if (url.protocol === "https:" && !url.username && !url.password) void openUrl(url.href).catch(toastError);
    } catch { /* Relative and active links are deliberately inert. */ }
  }

  function articleLinks(node: HTMLDivElement) {
    node.addEventListener("click", openLink);
    return { destroy() { node.removeEventListener("click", openLink); } };
  }
</script>

{#if items.length}
  <section class="news">
    <h2>{t("news.title")}</h2>
    <div class="tiles">
      {#each items as item, n (item.id)}
        <button class="tile" onclick={() => show(item)} title={item.title} in:reveal|global={{ index: n }}>
          <img src={item.image} alt="" loading="lazy" onerror={fallback} />
          <span class="name">
            {#if !read.includes(key(item))}<span class="unread" role="img" aria-label={t("news.unread")}></span>{/if}
            <span class="text">{item.title}</span>
          </span>
        </button>
      {/each}
    </div>
  </section>
{/if}

{#if selected}
  <Modal onclose={() => (selectedId = null)} width={560} flush>
    <div class="cover">
      <img src={selected.image} alt="" onerror={fallback} />
      <button class="close" aria-label={t("common.close")} onclick={() => (selectedId = null)}><X size={16} /></button>
    </div>
    <div class="body">
      <time datetime={selected.date}>{longDate(selected.date)}</time>
      <div class="article selectable" use:articleLinks>{@html renderAnnouncementMarkdown(selected.markdown)}</div>
    </div>
  </Modal>
{/if}

<style>
  .news { min-width: 0; }
  h2 { color: var(--text-2); font-size: var(--fs-13); font-weight: 600; margin-bottom: 8px; }
  /* Only tiles that fit whole are kept: extra rows collapse to nothing. */
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, 200px);
    grid-template-rows: auto;
    grid-auto-rows: 0;
    column-gap: 12px;
    overflow: hidden;
  }
  .tile { display: flex; flex-direction: column; gap: 4px; min-width: 0; text-align: left; }
  .tile img {
    display: block;
    width: 200px;
    height: var(--tile-h, 96px);
    object-fit: cover;
    border-radius: var(--radius);
    background: var(--panel-2);
  }
  .name { display: flex; align-items: center; gap: 6px; height: 20px; min-width: 0; font-size: var(--fs-13); font-weight: 500; color: var(--text-2); transition: color 0.12s; }
  .tile:hover .name { color: var(--text); }
  .text { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .unread { flex-shrink: 0; width: 6px; height: 6px; border-radius: 50%; background: var(--accent); }

  .cover { position: relative; }
  .cover img { display: block; width: 100%; height: 220px; object-fit: cover; }
  .close {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 28px;
    height: 28px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    background: rgb(16 18 20 / 0.6);
    color: #eef0ef;
  }
  .close:hover { background: rgb(16 18 20 / 0.8); }
  .body { padding: 16px 24px 24px; }
  time { color: var(--text-3); font-size: var(--fs-12); }
  .article { user-select: text; overflow-wrap: anywhere; line-height: 1.6; color: var(--text-2); font-size: var(--fs-14); }
  .article :global(h1) { color: var(--text); font-family: var(--font-display); font-size: var(--fs-20); font-weight: 600; line-height: 1.25; margin: 4px 0 12px; }
  .article :global(h2), .article :global(h3) { color: var(--text); font-size: var(--fs-14); margin: 16px 0 4px; }
  .article :global(p) { margin: 8px 0; }
  .article :global(ul), .article :global(ol) { padding-left: 20px; margin: 8px 0; }
  .article :global(a) { color: var(--text); text-underline-offset: 3px; }
  .article :global(pre) { overflow-x: auto; background: var(--panel-2); padding: 12px; border-radius: var(--radius-s); }
  .article :global(blockquote) { border-left: 2px solid var(--border-2); padding-left: 12px; margin: 8px 0; }
</style>
