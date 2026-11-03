<script lang="ts">
  import { onMount } from "svelte";
  import { blockTracks, css } from "../lib/blocks";
  import { dur } from "../lib/state.svelte";

  // Eight blocks roll in from around the scene and build one cube, hold, roll apart, and loop.
  // All geometry lives in lib/blocks.ts; here it only becomes matrix3d transforms.
  let { size = 180 }: { size?: number } = $props();

  const { duration, tracks } = blockTracks();
  const COLOURS = [
    { top: "#e9a06c", left: "#c47a47", right: "#7c4526", hidden: "#7c4526" },
    { top: "#6fdcc3", left: "#3fb8a0", right: "#2a8c7c", hidden: "#2a8c7c" },
  ];

  // svelte-ignore state_referenced_locally
  const unit = Math.round(size / 5.5);
  let blocks: HTMLElement[] = $state([]);

  onMount(() => {
    if (!dur(1)) return;
    const running = blocks.map((block, i) => block.animate(
      tracks[i].frames.map(({ offset, matrix }) => ({ offset, transform: css(matrix, unit) })),
      { duration, iterations: Infinity, easing: "linear" },
    ));
    return () => running.forEach((animation) => animation.cancel());
  });
</script>

<div class="stage" style:width="{size}px" style:height="{size}px" style:--unit="{unit}px" aria-hidden="true">
  <div class="scene">
    {#each tracks as track, i (i)}
      <div class="block" bind:this={blocks[i]} style:transform={css(track.home, unit)}>
        {#each track.faces as face (face.name)}
          <span class="face {face.name}" style:background={COLOURS[track.layer][face.shade]}></span>
        {/each}
      </div>
    {/each}
  </div>
</div>

<style>
  .stage { position: relative; }
  /* The floor spans cells -2..3; shifting by the cube's centre keeps it mid-stage. */
  .scene {
    position: absolute;
    left: 50%;
    top: 50%;
    width: 0;
    height: 0;
    transform-style: preserve-3d;
    transform: rotateX(-35.26deg) rotateY(45deg) translate3d(calc(var(--unit) * -1), var(--unit), calc(var(--unit) * -1));
  }
  .block {
    position: absolute;
    left: 0;
    top: 0;
    width: 0;
    height: 0;
    transform-origin: 0 0;
    transform-style: preserve-3d;
  }
  .face {
    position: absolute;
    left: calc(var(--unit) / -2);
    top: calc(var(--unit) / -2);
    width: var(--unit);
    height: var(--unit);
    backface-visibility: hidden;
  }
  .front { transform: translateZ(calc(var(--unit) / 2)); }
  .back { transform: rotateY(180deg) translateZ(calc(var(--unit) / 2)); }
  .right { transform: rotateY(90deg) translateZ(calc(var(--unit) / 2)); }
  .left { transform: rotateY(-90deg) translateZ(calc(var(--unit) / 2)); }
  .top { transform: rotateX(90deg) translateZ(calc(var(--unit) / 2)); }
  .bottom { transform: rotateX(-90deg) translateZ(calc(var(--unit) / 2)); }
</style>
