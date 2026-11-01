<script lang="ts">
  import { onMount } from "svelte";
  import { Newspaper, RefreshCw } from "@lucide/svelte";
  import { app, loadNews } from "../lib/state.svelte";
  import { errText } from "../lib/api";

  let error = $state("");
  let loading = $state(true);
  async function load() {
    error = "";
    loading = true;
    try { await loadNews(); }
    catch (e) { error = errText(e); }
    finally { loading = false; }
  }
  onMount(() => { void load(); });
  const date = (value: string) => new Intl.DateTimeFormat("pl-PL", { day: "numeric", month: "long", year: "numeric" }).format(new Date(`${value}T12:00:00`));
</script>

<div class="news-page">
  <div class="column">
    <header>
      <span class="eyebrow"><Newspaper size={14} /> Z ŻYCIA PATINY</span>
      <h1>NEWS<span class="headline-dot">.</span></h1>
      <p class="muted">Co nowego w Twoim launcherze.</p>
    </header>

    {#if error}
      <div class="empty" role="alert"><p>Nie udało się wczytać aktualności.</p><p class="muted selectable">{error}</p><button class="btn" onclick={load}><RefreshCw size={15} /> Spróbuj ponownie</button></div>
    {:else if loading}
      <p class="muted" role="status">Wczytywanie aktualności…</p>
    {:else}
      {#each app.news ?? [] as note, index (note.version)}
        <article>
          <div class="release-meta"><span class="chip accent">Patina {note.version}</span>{#if index === 0}<span class="latest">Najnowszy wpis</span>{/if}<time datetime={note.date}>{date(note.date)}</time></div>
          <h2>{note.title}</h2>
          <p class="summary">{note.summary}</p>
          <div class="changes">
            {#each note.sections as section (section.title)}
              <section><h3>{section.title}</h3><ul>{#each section.items as item (item)}<li>{item}</li>{/each}</ul></section>
            {/each}
          </div>
        </article>
      {:else}
        <div class="empty"><Newspaper size={28} /><h2>Tu pojawią się nowości</h2><p class="muted">Informacje o zmianach będą dostępne wraz z kolejną wersją Patiny.</p></div>
      {/each}
    {/if}
    <footer><span>Twoja wersja: <b>{app.info?.version}</b></span><span>Informacje dołączone do launchera · dostępne offline</span></footer>
  </div>
</div>

<style>
  .news-page { height: 100%; overflow: auto; padding: 24px 32px 36px; }
  .column { max-width: 760px; margin: 0 auto; }
  header { padding: 8px 0 30px; border-bottom: 1px solid var(--border); }
  .eyebrow { display: flex; align-items: center; gap: 8px; font-size: 0.7rem; font-weight: 650; letter-spacing: 0.14em; color: var(--text-2); }
  h1 { font-size: 3.4rem; font-weight: 650; letter-spacing: -0.06em; margin: 12px 0 6px; }
  .headline-dot { color: var(--accent); }
  header p { margin: 0; }
  article { padding: 30px 0; border-bottom: 1px solid var(--border); }
  .release-meta { display: flex; align-items: center; flex-wrap: wrap; gap: 10px; margin-bottom: 18px; }
  .latest { color: var(--text-2); font-size: 0.78rem; }
  time { margin-left: auto; color: var(--text-3); font-size: 0.82rem; }
  h2 { font-size: clamp(1.5rem, 2.4vw, 2.1rem); font-weight: 580; letter-spacing: -0.035em; line-height: 1.2; max-width: 600px; }
  .summary { color: var(--text-2); line-height: 1.7; margin: 14px 0 28px; max-width: 600px; }
  .changes { display: flex; flex-direction: column; gap: 22px; }
  h3 { font-family: var(--font); font-size: 0.88rem; font-weight: 650; }
  ul { margin: 9px 0 0; padding-left: 18px; color: var(--text-2); line-height: 1.7; font-size: 0.92rem; }
  li { padding-left: 4px; margin: 5px 0; }
  li::marker { color: var(--accent); }
  footer { display: flex; justify-content: space-between; flex-wrap: wrap; gap: 10px; color: var(--text-3); padding-top: 22px; font-size: 0.76rem; }
  footer b { font-weight: 500; color: var(--text-2); }
  .empty { padding: 40px 0; display: flex; flex-direction: column; align-items: flex-start; gap: 14px; }
  .empty p { margin: 0; }
</style>
