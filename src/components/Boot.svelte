<script lang="ts">
  import { fade } from "svelte/transition";
  import logo from "../assets/logo.svg?raw";
  import { APP_NAME } from "../lib/flavour";
  import { dur } from "../lib/state.svelte";

  // The launcher's opening: the logo's copper layer rises into place, the patina
  // layer settles onto it, and the name appears. Only transforms and opacity move: filters drop
  // WebKitGTK's frame rate. It waits for the app, and the app never waits past
  // the first run through.
  let { ready, ondone }: { ready: boolean; ondone: () => void } = $props();

  const MIN_MS = 1300;
  let shownLongEnough = $state(false);

  $effect(() => {
    const timer = setTimeout(() => (shownLongEnough = true), dur(MIN_MS));
    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (ready && shownLongEnough) ondone();
  });
</script>

<div class="boot" out:fade={{ duration: dur(180) }}>
  <div class="mark">
    <div class="logo" aria-hidden="true">{@html logo}</div>
    <div class="name">{APP_NAME}</div>
    <div class="track"><div class="fill"></div></div>
  </div>
</div>

<style>
  .boot {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: grid;
    place-items: center;
    background: #0d0f10;
  }
  .mark {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 22px;
  }
  .logo {
    width: 112px;
    height: 112px;
    opacity: 0;
    animation: appear 260ms ease-out forwards;
  }
  .logo :global(svg) {
    display: block;
    width: 100%;
    height: 100%;
    overflow: visible;
  }
  /* Faces only move: fading each one would let the layer behind show through. */
  .logo :global(polygon) {
    transform: var(--from);
    animation: land 560ms cubic-bezier(0.2, 0, 0, 1) forwards;
  }
  /* Moves are in the logo's own units: 512 across, drawn at 112 px. */
  .logo :global(.layer-0) {
    --from: translateY(150px);
  }
  .logo :global(.layer-1) {
    --from: translateY(-190px);
    animation-delay: 300ms;
  }
  .name {
    font-family: var(--font-display);
    font-size: var(--fs-20);
    font-weight: 600;
    color: #e6ecea;
    opacity: 0;
    transform: translateY(6px);
    animation: rise 360ms cubic-bezier(0.2, 0, 0, 1) 620ms forwards;
  }
  .track {
    width: 132px;
    height: 2px;
    overflow: hidden;
    border-radius: 2px;
    background: rgb(255 255 255 / 0.08);
  }
  .fill {
    width: 100%;
    height: 100%;
    background: #3fb8a0;
    transform: scaleX(0);
    transform-origin: left;
    animation: load 1100ms cubic-bezier(0.3, 0.1, 0.2, 1) 200ms forwards;
  }
  @keyframes appear {
    to {
      opacity: 1;
    }
  }
  @keyframes land {
    to {
      transform: none;
    }
  }
  @keyframes rise {
    to {
      opacity: 1;
      transform: none;
    }
  }
  @keyframes load {
    to {
      transform: scaleX(1);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .logo,
    .logo :global(polygon),
    .name,
    .fill {
      animation-duration: 1ms;
      animation-delay: 0ms;
    }
  }
</style>
