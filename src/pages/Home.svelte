<script lang="ts">
  import { ChevronUp, ImageIcon, Plus } from "@lucide/svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import Modal from "../components/Modal.svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import AnnouncementCards from "../components/AnnouncementCards.svelte";
  import { loaderLabel, playTime, relativeTime } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { refCss } from "../lib/images";
  import { dialog, reveal } from "../lib/motion";
  import { bannerFor, iconFor } from "../lib/presets";
  import { activeAccount, app, navigate, updateSettings } from "../lib/state.svelte";

  const acc = $derived(activeAccount());
  const skin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);
  const selected = $derived(app.instances.find((i) => i.id === app.settings?.lastInstance) ?? app.instances[0] ?? null);
  const recent = $derived(app.instances.filter((i) => i.lastPlayed !== null).sort((a, b) => b.lastPlayed! - a.lastPlayed!).slice(0, 3));
  const newsEmpty = $derived(app.announcements !== null && app.announcements.items.length === 0);
  let picker = $state(false);
  let wallpaperOpen = $state(false);

  const s = $derived(app.settings!);
  const banner = $derived(bannerFor(selected?.banner ?? "aurora"));
  const stageBg = $derived(s.heroWallpaper ? refCss(s.heroWallpaper, banner) : banner);
  const meta = $derived(selected ? [
    `${loaderLabel(selected.loader)} ${selected.mcVersion}`,
    selected.playTimeSecs > 0 ? playTime(selected.playTimeSecs) : "",
    selected.lastPlayed ? relativeTime(selected.lastPlayed) : "",
  ].filter(Boolean).join(" · ") : "");

  let stageW = $state(800);
  let stageH = $state(400);
  // The canvas stays inside the stage on both sides of the model's centre, wide enough for the sword.
  const modelX = $derived(Math.round(stageW * 0.68));
  const modelH = $derived(Math.round(Math.max(160, Math.min(stageH, (stageW - modelX) * 2 / 1.25, 640))));
  const modelW = $derived(Math.round(modelH * 1.25));
</script>

<svelte:window
  onclick={(e) => picker && !(e.target as HTMLElement).closest(".picker") && (picker = false)}
  onkeydown={(e) => e.key === "Escape" && (picker = false)}
/>

{#if wallpaperOpen}
  <Modal onclose={() => (wallpaperOpen = false)} width={560}>
    <h3 class="modal-title">{t("home.wallpaper")}</h3>
    <ImagePicker value={s.heroWallpaper} noneLabel={t("look.instanceBanner")} onchange={(v) => updateSettings({ heroWallpaper: v })} />
  </Modal>
{/if}

<div class="home">
  <section class="stage" bind:clientWidth={stageW} bind:clientHeight={stageH} style:background={stageBg}>
    <div class="scrim"></div>
    {#if skin && acc}
      <div class="model" style:left="{modelX - modelW / 2}px">
        {#if s.showNametag}<div class="nametag">{acc.name}</div>{/if}
        <SkinViewer {skin} slim={acc.skinSlim} armed followCursor={s.followCursor} width={modelW} height={modelH} interactive={false} zoom={0.8} />
      </div>
    {/if}

    <button class="tool" title={t("home.wallpaper")} aria-label={t("home.wallpaper")} onclick={() => (wallpaperOpen = true)}><ImageIcon size={16} /></button>

    <div class="info">
      {#if selected}
        <h1>{selected.name}</h1>
        <p>{meta}</p>
        <div class="actions">
          <PlayButton instance={selected} />
          <div class="picker">
            <button class="switch" aria-expanded={picker} onclick={() => (picker = !picker)}>{t("home.switchInstance")} <ChevronUp size={14} /></button>
            {#if picker}
              <div class="menu card" in:dialog out:dialog={{ duration: 100 }}>
                {#each app.instances as i (i.id)}
                  {@const Icon = iconFor(i.icon)}
                  <button class="item" class:on={i.id === selected.id} onclick={() => { updateSettings({ lastInstance: i.id }); picker = false; }}>
                    <Icon size={14} />
                    <span class="item-text"><b>{i.name}</b><small>{loaderLabel(i.loader)} {i.mcVersion}</small></span>
                  </button>
                {/each}
                <button class="item add" onclick={() => { picker = false; app.newInstanceOpen = true; }}><Plus size={14} /> {t("home.newInstance")}</button>
              </div>
            {/if}
          </div>
        </div>
      {:else}
        <h1>{t("home.noInstances")}</h1>
        <div class="actions"><button class="btn primary create" onclick={() => (app.newInstanceOpen = true)}><Plus size={16} /> {t("home.createFirst")}</button></div>
      {/if}
    </div>
  </section>

  <div class="below" class:solo={newsEmpty}>
    <AnnouncementCards />
    <section class="recent">
      <div class="head">
        <h2>{t("home.recent")}</h2>
        <button class="all" onclick={() => navigate("instances")}>{t("home.all")}</button>
      </div>
      {#each recent as i, n (i.id)}
        {@const Icon = iconFor(i.icon)}
        <div class="line" in:reveal|global={{ index: n }}>
          <button class="line-main" onclick={() => navigate("instance", i.id)}>
            <span class="ico" style:background={bannerFor(i.banner)}><Icon size={14} /></span>
            <span class="item-text"><b>{i.name}</b><small>{relativeTime(i.lastPlayed)}</small></span>
          </button>
          <PlayButton instance={i} size="sm" />
        </div>
      {:else}
        <p class="empty">{t("home.recentEmpty")}</p>
      {/each}
    </section>
  </div>
</div>

<style>
  .modal-title { font-size: var(--fs-16); margin-bottom: 16px; }
  .home {
    --tile-h: 96px;
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    overflow: hidden;
  }
  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
    border-radius: var(--radius-l);
    overflow: hidden;
  }
  /* Only behind the text; the wallpaper stays as chosen elsewhere. */
  .scrim {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, rgb(12 14 15 / 0.72), transparent 56%);
  }
  .model {
    position: absolute;
    bottom: 0;
  }
  .nametag {
    position: absolute;
    top: 4%;
    left: 50%;
    transform: translateX(-50%);
    z-index: 1;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgb(12 14 15 / 0.56);
    color: #eef0ef;
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
  }
  .tool {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    color: #eef0ef;
    background: rgb(12 14 15 / 0.48);
    transition: background 0.12s;
  }
  .tool:hover { background: rgb(12 14 15 / 0.72); }
  .info {
    position: absolute;
    left: 24px;
    bottom: 24px;
    max-width: min(52%, 460px);
    color: #eef0ef;
  }
  h1 {
    font-family: var(--font-display);
    font-size: var(--fs-28);
    font-weight: 600;
    line-height: 1.15;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .info p {
    margin: 4px 0 0;
    color: #c6cbc9;
    font-size: var(--fs-13);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
  }
  .create { height: 44px; padding: 0 16px; border-radius: var(--radius); }
  .picker { position: relative; }
  .switch {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 44px;
    padding: 0 12px;
    border-radius: var(--radius);
    background: rgb(12 14 15 / 0.56);
    color: #eef0ef;
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
    transition: background 0.12s;
  }
  .switch:hover, .switch[aria-expanded="true"] { background: rgb(12 14 15 / 0.8); }
  .menu {
    position: absolute;
    left: 0;
    bottom: 52px;
    width: 256px;
    max-height: 280px;
    overflow: auto;
    padding: 4px;
    box-shadow: var(--shadow);
    color: var(--text);
  }
  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 40px;
    padding: 4px 8px;
    border-radius: var(--radius-s);
    color: var(--text-2);
    font-size: var(--fs-13);
    text-align: left;
  }
  .item:hover { background: var(--panel-2); color: var(--text); }
  .item.on { background: var(--panel-2); color: var(--text); }
  .item.add { min-height: 32px; font-weight: 500; }
  .item-text { display: flex; flex-direction: column; min-width: 0; line-height: 1.25; }
  .item-text b { font-size: var(--fs-13); font-weight: 500; color: var(--text); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .item-text small { font-size: var(--fs-12); color: var(--text-3); }

  .below {
    flex-shrink: 0;
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(240px, 320px);
    gap: 16px;
  }
  .below.solo { grid-template-columns: minmax(0, 1fr); }
  .head { display: flex; align-items: center; justify-content: space-between; height: 20px; margin-bottom: 8px; }
  h2 { color: var(--text-2); font-size: var(--fs-13); font-weight: 600; }
  .all { color: var(--text-3); font-size: var(--fs-12); transition: color 0.12s; }
  .all:hover { color: var(--text); }
  .recent { min-width: 0; }
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 40px;
  }
  .line-main {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 100%;
    border-radius: var(--radius-s);
    text-align: left;
  }
  .ico {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    color: #eef0ef;
  }
  .empty { margin: 0; color: var(--text-3); font-size: var(--fs-13); }

  @media (max-height: 640px) {
    .home { --tile-h: 64px; padding: 12px; }
    .stage { border-radius: var(--radius); }
    .info { left: 16px; bottom: 16px; }
    .actions { margin-top: 8px; }
    h1 { font-size: var(--fs-20); }
    .line:nth-of-type(n + 4) { display: none; }
  }
</style>
