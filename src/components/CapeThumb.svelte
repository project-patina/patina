<script lang="ts">
  // Front face of a cape texture (64×32 layout, scaled for HD capes).
  let { src, width = 50 }: { src: string; width?: number } = $props();
  let canvas: HTMLCanvasElement | undefined = $state();

  $effect(() => {
    if (!canvas || !src) return;
    const target = canvas;
    const img = new Image();
    img.onload = () => {
      const k = img.width / 64;
      const ctx = target.getContext("2d");
      if (!ctx) return;
      ctx.imageSmoothingEnabled = false;
      ctx.clearRect(0, 0, 10, 16);
      ctx.drawImage(img, 1 * k, 1 * k, 10 * k, 16 * k, 0, 0, 10, 16);
    };
    img.src = src;
    return () => { img.onload = null; };
  });
</script>

<canvas bind:this={canvas} width="10" height="16" style:width="{width}px" style:height="{width * 1.6}px"></canvas>

<style>
  canvas {
    image-rendering: pixelated;
    display: block;
    border-radius: 4px;
  }
</style>
