<script lang="ts">
  import { Check, ImagePlus, Trash2, X } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { BANNERS } from "../lib/presets";
  import { importWallpapers, refCss } from "../lib/images";
  import { app, toast, toastError, updateSettings } from "../lib/state.svelte";

  // Grid of presets + the user's own images. `value` is an image ref.
  let {
    value = $bindable(""),
    allowNone = true,
    noneLabel = "",
    onchange,
  }: { value?: string; allowNone?: boolean; noneLabel?: string; onchange?: (v: string) => void } = $props();

  const isPreset = (key: string) => value === `preset:${key}` || value === key;

  function pick(v: string) {
    value = v;
    onchange?.(v);
  }

  $effect(() => {
    if (!app.wallpapers) void api.listImages("wallpaper").then((l) => { app.wallpapers = l; }).catch(() => { app.wallpapers = []; });
  });

  async function add() {
    try {
      const ids = await importWallpapers();
      if (!ids.length) return;
      app.wallpapers = await api.listImages("wallpaper");
      pick(`custom:${ids[0]}`);
      toast(t("images.added", { n: ids.length }), "success");
    } catch (e) {
      toastError(e);
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteImage("wallpaper", id);
      app.wallpapers = (app.wallpapers ?? []).filter((w) => w.id !== id);
      const ref = `custom:${id}`;
      if (value === ref) pick("");
      // Both wallpapers share the library: clear the other one's reference too.
      if (app.settings?.heroWallpaper === ref) updateSettings({ heroWallpaper: "" });
      if (app.settings?.appWallpaper === ref) updateSettings({ appWallpaper: "" });
    } catch (e) {
      toastError(e);
    }
  }
</script>

<div class="picker">
  {#if allowNone}
    <button class="tile none" class:on={!value} aria-pressed={!value} onclick={() => pick("")} title={noneLabel || t("images.none")}>
      <X size={16} /><span>{noneLabel || t("images.none")}</span>
    </button>
  {/if}
  <button class="tile add" onclick={add} title={t("images.addTitle")}>
    <ImagePlus size={16} /><span>{t("images.add")}</span>
  </button>
  {#each app.wallpapers ?? [] as w (w.id)}
    {@const ref = `custom:${w.id}`}
    <div class="tile custom" class:on={value === ref} style:background={refCss(ref)}>
      <button class="hit" aria-pressed={value === ref} onclick={() => pick(ref)} title={w.name} aria-label={w.name}></button>
      {#if value === ref}<span class="ok"><Check size={12} /></span>{/if}
      <button class="del" onclick={() => remove(w.id)} aria-label={t("images.remove")}><Trash2 size={12} /></button>
    </div>
  {/each}
  {#each Object.keys(BANNERS) as k (k)}
    <button class="tile" class:on={isPreset(k)} aria-pressed={isPreset(k)} style:background={BANNERS[k]} onclick={() => pick(`preset:${k}`)} aria-label={k}>
      {#if isPreset(k)}<span class="ok"><Check size={12} /></span>{/if}
    </button>
  {/each}
</div>

<style>
  .picker {
    width: 100%;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
    gap: 8px;
  }
  .tile {
    position: relative;
    aspect-ratio: 16 / 10;
    border-radius: var(--radius-s);
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    justify-content: flex-start;
    padding: 4px;
    border: 1px solid var(--border);
  }
  .tile.on { outline: 2px solid var(--accent); outline-offset: 1px; }
  .none,
  .add {
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: var(--panel-2);
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
    transition: color 0.12s;
  }
  .none:hover, .add:hover { color: var(--text); }
  .ok {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: var(--accent-fg);
  }
  .hit {
    position: absolute;
    inset: 0;
  }
  .del {
    position: absolute;
    bottom: 4px;
    right: 4px;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    display: grid;
    place-items: center;
    background: rgb(16 18 20 / 0.6);
    color: #eef0ef;
    opacity: 0;
    transition: opacity 0.12s;
  }
  .custom:hover .del, .del:focus-visible { opacity: 1; }
  .del:hover { background: var(--danger); }
</style>
