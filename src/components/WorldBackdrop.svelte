<script lang="ts">
  import { onMount } from "svelte";
  import { api, mediaUrl } from "../lib/api";

  // The film of a real village behind the player (scripts/render-backdrop.py),
  // made from the player's own game; without it the wallpaper stays. It plays as
  // a run of JPEG frames because `<video>` brings down WebKitGTK when GStreamer's
  // good plugins are missing, and most frames only differ by a slow cloud.
  const FRAME_MS = 1000 / 12;
  let canvas: HTMLCanvasElement;
  let ready = $state(false);

  function load(frame: number) {
    const image = new Image();
    image.src = mediaUrl("backdrop", `${String(frame).padStart(5, "0")}.jpg`);
    return image.decode().then(() => image);
  }

  onMount(() => {
    let stopped = false;
    let visible = true;
    const seen = new IntersectionObserver(([e]) => (visible = e.isIntersecting));
    seen.observe(canvas);

    async function play(count: number) {
      const context = canvas.getContext("2d")!;
      let frame = 1;
      let next = load(frame);
      while (!stopped) {
        const started = performance.now();
        if (visible && !document.hidden) {
          const image = await next;
          if (stopped) return;
          if (canvas.width !== image.naturalWidth) {
            canvas.width = image.naturalWidth;
            canvas.height = image.naturalHeight;
          }
          context.drawImage(image, 0, 0);
          ready = true;
          frame = (frame % count) + 1;
          next = load(frame);
        }
        await new Promise((done) => setTimeout(done, Math.max(1, FRAME_MS - (performance.now() - started))));
      }
    }

    api
      .backdropFrames()
      .then((count) => {
        if (count && !stopped) void play(count).catch(() => { ready = false; });
      })
      .catch(() => {});
    return () => {
      stopped = true;
      seen.disconnect();
    };
  });
</script>

<canvas bind:this={canvas} class:ready></canvas>

<style>
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 0.5s var(--ease);
    pointer-events: none;
  }
  canvas.ready {
    opacity: 1;
  }
</style>
