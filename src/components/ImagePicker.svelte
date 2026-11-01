<script lang="ts">
  import { Check, ImagePlus, Trash2, X } from "@lucide/svelte";
  import { api } from "../lib/api";
  import { BANNERS } from "../lib/presets";
  import { importWallpapers, refCss } from "../lib/images";
  import { app, toast, toastError } from "../lib/state.svelte";

  // Grid of presets + the user's own images. `value` is an image ref.
  let {
    value = $bindable(""),
    allowNone = true,
    noneLabel = "Brak",
    onchange,
  }: { value?: string; allowNone?: boolean; noneLabel?: string; onchange?: (v: string) => void } = $props();

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
      toast(ids.length > 1 ? `Dodano ${ids.length} obrazy` : "Dodano obraz", "success");
    } catch (e) {
      toastError(e);
    }
  }

  async function remove(id: string) {
    try {
      await api.deleteImage("wallpaper", id);
      app.wallpapers = (app.wallpapers ?? []).filter((w) => w.id !== id);
      if (value === `custom:${id}`) pick("");
    } catch (e) {
      toastError(e);
    }
  }
</script>

<div class="picker">
  {#if allowNone}
    <button class="tile none" class:on={!value} onclick={() => pick("")} title={noneLabel}>
      <X size={16} /><span>{noneLabel}</span>
    </button>
  {/if}
  <button class="tile add" onclick={add} title="Dodaj własny obraz">
    <ImagePlus size={18} /><span>Własny</span>
  </button>
  {#each app.wallpapers ?? [] as w (w.id)}
    {@const ref = `custom:${w.id}`}
    <div class="tile custom" class:on={value === ref} style:background={refCss(ref)}>
      <button class="hit" onclick={() => pick(ref)} title={w.name} aria-label={w.name}></button>
      {#if value === ref}<span class="ok"><Check size={13} /></span>{/if}
      <button class="del" onclick={() => remove(w.id)} aria-label="Usuń obraz"><Trash2 size={12} /></button>
    </div>
  {/each}
  {#each Object.keys(BANNERS) as k (k)}
    <button class="tile" class:on={value === `preset:${k}` || value === k} style:background={BANNERS[k]} onclick={() => pick(`preset:${k}`)} aria-label={k}>
      {#if value === `preset:${k}` || value === k}<span class="ok"><Check size={13} /></span>{/if}
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
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    align-items: flex-end;
    justify-content: flex-start;
    padding: 6px;
    transition: transform 0.2s var(--ease), border-color 0.15s;
  }
  .tile:hover {
    transform: translateY(-2px);
  }
  .tile.on {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .none,
  .add {
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: var(--panel-2);
    color: var(--text-2);
    font-size: 0.78rem;
    font-weight: 600;
  }
  .add:hover {
    color: var(--accent);
  }
  .ok {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: white;
  }
  .hit {
    position: absolute;
    inset: 0;
  }
  .del {
    position: absolute;
    bottom: 5px;
    right: 5px;
    width: 22px;
    height: 22px;
    border-radius: 6px;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.55);
    color: white;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .custom:hover .del {
    opacity: 1;
  }
  .del:hover {
    background: var(--danger);
  }
</style>
