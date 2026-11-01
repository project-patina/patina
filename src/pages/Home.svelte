<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { ArrowUpRight, ChevronDown, Clock, Ellipsis, ImageIcon, Newspaper, PersonStanding, Plus } from "@lucide/svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import Modal from "../components/Modal.svelte";
  import { refCss } from "../lib/images";
  import Head from "../components/Head.svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import Toggle from "../components/Toggle.svelte";
  import WorldBackdrop from "../components/WorldBackdrop.svelte";
  import { bannerFor, iconFor } from "../lib/presets";
  import { greeting, loaderLabel, relativeTime } from "../lib/format";
  import { parsePose } from "../lib/pose";
  import { activeAccount, app, dur, loadNews, navigate, updateSettings } from "../lib/state.svelte";

  const acc = $derived(activeAccount());
  const skin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);
  const selected = $derived(app.instances.find((i) => i.id === app.settings?.lastInstance) ?? app.instances[0] ?? null);
  const recent = $derived(app.instances.filter((i) => i.lastPlayed !== null).sort((a, b) => b.lastPlayed! - a.lastPlayed!).slice(0, 3));
  onMount(() => { void loadNews().catch(() => {}); });
  let picker = $state(false);
  const pose = $derived(parsePose(app.settings?.modelPose));
  let wallpaperOpen = $state(false);

  const s = $derived(app.settings!);
  const heroBg = $derived(s.heroWallpaper ? refCss(s.heroWallpaper, bannerFor(selected?.banner ?? "aurora")) : bannerFor(selected?.banner ?? "aurora"));

  let heroW = $state(800);
  let heroH = $state(400);
  // Fit between the bottom bar (~70px) and a little headroom for the nametag.
  const modelH = $derived(Math.round(Math.max(180, Math.min(heroH - 100, heroW * 0.5, 420))));
</script>

<svelte:window onclick={(e) => picker && !(e.target as HTMLElement).closest(".picker") && (picker = false)} />

{#if wallpaperOpen}
  <Modal onclose={() => (wallpaperOpen = false)} width={640}>
    <h3>Tapeta karty startowej</h3>
    <p class="dim" style="margin: 4px 0 14px">„Brak” = baner wybranej instancji. Własne obrazy PNG/JPG/WebP/GIF.</p>
    <div class="dimrow">
      <span>Kinowe tło</span>
      <Toggle checked={s.heroScene} onchange={(v) => updateSettings({ heroScene: v })} />
    </div>
    <ImagePicker value={s.heroWallpaper} noneLabel="Baner instancji" onchange={(v) => updateSettings({ heroWallpaper: v, heroScene: false })} />
    <div class="dimrow">
      <span>Przyciemnienie</span>
      <input type="range" min="0" max="80" value={s.heroDim} oninput={(e) => updateSettings({ heroDim: +e.currentTarget.value })} />
      <span class="mono">{s.heroDim}%</span>
    </div>
  </Modal>
{/if}

<div class="home">
  <header class="greet" in:fly={{ y: 10, duration: dur(400) }}>
    <h1>
      {greeting()},
      {#if acc}<Head skinUrl={acc.skinUrl} name={acc.name} size={30} radius={7} /><span>{acc.name}!</span>{/if}
    </h1>
    <button class="news-link" onclick={() => navigate("news")}><Newspaper size={15} /><span>NEWS</span><span class="news-version">{app.news?.[0]?.version ?? app.info?.version}</span><ArrowUpRight size={14} /></button>
  </header>

  <div class="grid">
    <div class="main">
      <section class="hero pos-{s.modelPosition}" bind:clientWidth={heroW} bind:clientHeight={heroH} style:background={heroBg}>
        {#if s.heroScene}<WorldBackdrop />{/if}
        <div class="hero-shade" style:--dim={s.heroDim / 100}></div>
        <details class="hero-tools">
          <summary aria-label="Dostosuj ekran startowy" title="Dostosuj ekran startowy"><Ellipsis size={19} /></summary>
          <div class="tools-menu">
            <button onclick={() => (wallpaperOpen = true)}><ImageIcon size={15} /> Zmień tło</button>
            <button onclick={() => navigate("pose")}><PersonStanding size={15} /> Poza postaci</button>
          </div>
        </details>
        {#if skin && acc}
          <div class="floor"></div>
          <div class="model">
            {#if s.showNametag}<div class="nametag">{acc.name}</div>{/if}
            <SkinViewer {skin} slim={acc.skinSlim} armed pose={() => pose} followCursor={s.followCursor} width={Math.round(modelH * 1.5)} height={modelH} interactive={false} zoom={0.8} />
          </div>
        {/if}

        <div class="hero-bottom">
          {#if selected}
            {@const Icon = iconFor(selected.icon)}
            <div class="picker">
              <button class="inst-pick" aria-label="Wybierz instancję" aria-expanded={picker} onclick={() => (picker = !picker)}>
                <span class="ico"><Icon size={16} /></span>
                <span class="pick-text"><b>{selected.name}</b><small>{loaderLabel(selected.loader)} {selected.mcVersion}</small></span>
                <ChevronDown size={15} />
              </button>
              {#if picker}
                <div class="pick-menu card glass" transition:fly={{ y: 8, duration: dur(180) }}>
                  {#each app.instances as i (i.id)}
                    {@const I = iconFor(i.icon)}
                    <button class="pick-item" class:on={i.id === selected.id} onclick={() => { updateSettings({ lastInstance: i.id }); picker = false; }}>
                      <span class="ico sm"><I size={14} /></span>
                      <span class="pick-text"><b>{i.name}</b><small>{loaderLabel(i.loader)} {i.mcVersion}</small></span>
                    </button>
                  {/each}
                  <button class="pick-item new" onclick={() => { picker = false; app.newInstanceOpen = true; }}><Plus size={15} /> Nowa instancja</button>
                </div>
              {/if}
            </div>
            <PlayButton instance={selected} />

          {:else}
            <button class="btn primary create" onclick={() => (app.newInstanceOpen = true)}><Plus size={17} /> Utwórz pierwszą instancję</button>
          {/if}
        </div>
      </section>

      <section class="recent">
        <div class="section-head">
          <span class="label"><Clock size={13} /> Ostatnio grane</span>
          <button class="btn ghost sm" onclick={() => navigate("instances")}>Wszystkie</button>
        </div>
        {#each recent as i, n (i.id)}
          {@const Icon = iconFor(i.icon)}
          <div class="recent-row card" class:running={app.running[i.id] === "running"} in:fly={{ y: 12, duration: dur(380), delay: dur(60 * n) }}>
            <button class="row-main" onclick={() => navigate("instance", i.id)}>
              <span class="ico" style:background={bannerFor(i.banner)}><Icon size={17} /></span>
              <span class="row-text">
                <b>{i.name}</b>
                <small>{loaderLabel(i.loader)} {i.mcVersion} · {relativeTime(i.lastPlayed)}</small>
              </span>
            </button>
            <PlayButton instance={i} size="sm" />
          </div>
        {:else}
          <div class="empty">{app.instances.length ? "Twoje ostatnie światy pojawią się tutaj po pierwszej grze." : "Zacznij od utworzenia instancji powyżej."}</div>
        {/each}
      </section>
    </div>


  </div>
</div>

<style>
  .home {
    height: 100%;
    overflow: auto;
    padding: 10px 28px 28px 10px;
  }
  .greet { display: flex; align-items: center; justify-content: space-between; gap: 18px; max-width: 1100px; margin: 0 auto 20px; }
  .news-link { display: flex; align-items: center; gap: 8px; color: var(--text-2); font-size: 0.78rem; font-weight: 600; padding: 8px 0 8px 10px; }
  .news-link:hover { color: var(--accent); }
  .news-version { color: var(--text-3); font-weight: 400; }
  .greet h1 {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 1.65rem;
    font-weight: 700;
    margin: 4px 0;
  }
  .grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    max-width: 1100px;
    margin: 0 auto;
    gap: 16px;
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }
  .hero {
    /* The hero sits on an image in both themes: keep the light Play button. */
    --play-bg: #fafafa;
    --play-fg: #09090b;
    position: relative;
    height: clamp(340px, 52vh, 470px);
    border-radius: var(--radius-l);
    overflow: hidden;
    border: 1px solid var(--border);
    transition: background 0.6s var(--ease);
  }
  .hero-shade {
    position: absolute;
    inset: 0;
    background:
      linear-gradient(to top, rgb(0 0 0 / 0.55), transparent 45%),
      linear-gradient(rgb(0 0 0 / var(--dim, 0.25)), rgb(0 0 0 / var(--dim, 0.25)));
  }
  .hero-tools { position: absolute; top: 14px; right: 14px; z-index: 5; color: white; }
  .hero-tools summary { width: 32px; height: 32px; display: grid; place-items: center; list-style: none; border-radius: 8px; cursor: pointer; background: rgb(0 0 0 / 0.3); border: 1px solid rgb(255 255 255 / 0.15); }
  .hero-tools summary::-webkit-details-marker { display: none; }
  .tools-menu { position: absolute; right: 0; top: 40px; min-width: 180px; padding: 5px; border-radius: var(--radius); background: var(--panel); color: var(--text); border: 1px solid var(--border-2); box-shadow: var(--shadow); }
  .tools-menu button { display: flex; align-items: center; gap: 9px; width: 100%; padding: 10px; text-align: left; border-radius: var(--radius-s); font-size: 0.88rem; }
  .tools-menu button:hover { background: var(--panel-2); }
  .pos-left .model,
  .pos-left .floor {
    left: 24%;
  }
  .pos-right .model,
  .pos-right .floor {
    left: 76%;
  }
  .dimrow {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 16px;
    font-size: 0.9rem;
  }
  .dimrow input {
    flex: 1;
    accent-color: var(--accent);
  }
  .hero::after {
    content: "";
    position: absolute;
    inset: 0;
    background-image:
      linear-gradient(to bottom, transparent, rgb(0 0 0 / 0.25) 85%),
      linear-gradient(rgb(255 255 255 / 0.03) 1px, transparent 1px),
      linear-gradient(90deg, rgb(255 255 255 / 0.03) 1px, transparent 1px);
    background-size: 100% 100%, 32px 32px, 32px 32px;
    pointer-events: none;
  }
  .model {
    position: absolute;
    left: 50%;
    bottom: 64px;
    transform: translateX(-50%);
    z-index: 1;
  }
  .floor {
    position: absolute;
    left: 50%;
    bottom: 60px;
    width: 220px;
    height: 40px;
    transform: translateX(-50%);
    border-radius: 50%;
    background: radial-gradient(closest-side, rgb(0 0 0 / 0.45), transparent);
    z-index: 1;
  }
  .nametag {
    position: absolute;
    top: -8px;
    left: 50%;
    transform: translateX(-50%);
    white-space: nowrap;
    z-index: 2;
    padding: 5px 12px;
    border-radius: 6px;
    background: rgb(0 0 0 / 0.45);
    color: white;
    font-family: var(--font-display);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .hero-bottom {
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: 16px;
    z-index: 3;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
  }
  .hero-bottom > :global(.play) {
    grid-column: 2;
  }
  .create {
    grid-column: 2;
    height: 48px;
    padding: 0 20px;
  }

  .picker {
    position: relative;
    justify-self: start;
    min-width: 0;
  }
  .inst-pick {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 12px 7px 7px;
    border-radius: 12px;
    color: white;
    background: rgb(0 0 0 / 0.35);
    border: 1px solid rgb(255 255 255 / 0.12);
    max-width: 240px;
    transition: background 0.2s;
  }
  .inst-pick:hover {
    background: rgb(0 0 0 / 0.5);
  }
  .ico {
    width: 32px;
    height: 32px;
    border-radius: 9px;
    display: grid;
    place-items: center;
    background: rgb(255 255 255 / 0.12);
    color: white;
    flex-shrink: 0;
  }
  .ico.sm {
    width: 28px;
    height: 28px;
    background: var(--panel-3);
    color: var(--text);
  }
  .pick-text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    line-height: 1.2;
    min-width: 0;
  }
  .pick-text b {
    font-size: 0.88rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 150px;
  }
  .pick-text small {
    font-size: 0.75rem;
    opacity: 0.65;
  }
  .pick-menu {
    position: absolute;
    bottom: 54px;
    left: 0;
    width: 260px;
    max-height: 300px;
    overflow: auto;
    padding: 6px;
    border-radius: var(--radius);
    box-shadow: var(--shadow);
  }
  .pick-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px;
    border-radius: 8px;
    text-align: left;
  }
  .pick-item:hover,
  .pick-item.on {
    background: var(--panel-2);
  }
  .pick-item.new {
    color: var(--text-2);
    font-weight: 600;
    padding: 10px;
  }
  .section-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .recent {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .recent-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px 8px 8px;
    border-radius: var(--radius);
    transition: border-color 0.2s, transform 0.2s var(--ease);
  }
  .recent-row:hover {
    border-color: var(--border-2);
  }
  .recent-row.running {
    border-color: color-mix(in srgb, var(--success) 50%, transparent);
  }
  .row-main {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 12px;
    text-align: left;
    min-width: 0;
  }
  .row-main .ico {
    width: 38px;
    height: 38px;
    border-radius: 10px;
  }
  .row-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .row-text b {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-text small {
    color: var(--text-3);
    font-size: 0.82rem;
    margin-top: 2px;
  }
  .empty {
    padding: 22px;
    color: var(--text-2);
    text-align: center;
  }

</style>
