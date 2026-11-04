<script lang="ts">
  import { onMount } from "svelte";
  import sheet from "../assets/patina-splash.png?url";
  import { dur } from "../lib/state.svelte";

  // The loading animation of both game clients: eight blocks roll in and build one cube, hold,
  // roll apart and loop. It plays the sheet scripts/render-splash.mjs renders from lib/blocks.ts,
  // the one the clients play, frame by frame: CSS 3D faces of the same blocks drew skewed
  // squares and dark seams between them in WebKitGTK.
  let { size = 180 }: { size?: number } = $props();

  const COLUMNS = 8;
  const FRAMES = 64;
  const LOOP_MS = 5120;
  /** A frame of the whole cube, shown still when motion is reduced. */
  const WHOLE = 32;

  const position = (frame: number) => {
    const x = (frame % COLUMNS) * size;
    const y = Math.floor(frame / COLUMNS) * size;
    return `-${x}px -${y}px`;
  };
  let stage: HTMLElement | undefined = $state();

  onMount(() => {
    if (!stage || !dur(1)) return;
    // Each frame holds until the next, as the clients step through the sheet.
    const frames = Array.from({ length: FRAMES + 1 }, (_, i) => ({
      offset: i / FRAMES,
      backgroundPosition: position(i % FRAMES),
      easing: "steps(1, end)",
    }));
    const running = stage.animate(frames, { duration: LOOP_MS, iterations: Infinity });
    return () => running.cancel();
  });
</script>

<div
  class="stage"
  bind:this={stage}
  style:width="{size}px"
  style:height="{size}px"
  style:background-image="url({sheet})"
  style:background-size="{size * COLUMNS}px {size * COLUMNS}px"
  style:background-position={position(dur(1) ? 0 : WHOLE)}
  aria-hidden="true"
></div>

<style>
  .stage {
    background-repeat: no-repeat;
  }
</style>
