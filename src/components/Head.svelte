<script lang="ts">
  import { app } from "../lib/state.svelte";

  // Face of a Minecraft skin, drawn pixel-perfect (base layer + hat overlay).
  // `src` (a data/blob URL of a skin) wins over the account's `skinUrl`.
  let { skinUrl = null, src = null, name = "", size = 28, radius = 7 }: { skinUrl?: string | null; src?: string | null; name?: string; size?: number; radius?: number } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  const data = $derived(src ?? (skinUrl ? app.skins[skinUrl] : undefined));

  $effect(() => {
    if (!canvas || !data) return;
    const target = canvas;
    const img = new Image();
    img.onload = () => {
      const ctx = target.getContext("2d");
      if (!ctx) return;
      ctx.imageSmoothingEnabled = false;
      ctx.clearRect(0, 0, 8, 8);
      ctx.drawImage(img, 8, 8, 8, 8, 0, 0, 8, 8);
      // Old 64×32 skins often fill the hat area with opaque black; like the
      // game, only draw the hat when it actually has transparency.
      const probe = document.createElement("canvas").getContext("2d")!;
      probe.canvas.width = probe.canvas.height = 8;
      probe.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8);
      const alpha = probe.getImageData(0, 0, 8, 8).data.filter((_, i) => i % 4 === 3);
      if (alpha.some((a) => a < 255)) ctx.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8);
    };
    img.src = data;
    return () => { img.onload = null; };
  });
</script>

{#if data}
  <canvas bind:this={canvas} width="8" height="8" style:width="{size}px" style:height="{size}px" style:border-radius="{radius}px"></canvas>
{:else}
  <span class="fallback" style:width="{size}px" style:height="{size}px" style:border-radius="{radius}px" style:font-size="{size * 0.46}px">
    {name.slice(0, 1).toUpperCase() || "?"}
  </span>
{/if}

<style>
  canvas {
    image-rendering: pixelated;
    display: block;
    flex-shrink: 0;
  }
  .fallback {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
    font-weight: 700;
    font-family: var(--font-display);
  }
</style>
