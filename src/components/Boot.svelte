<script lang="ts">
  import { fade } from "svelte/transition";
  import { APP_NAME } from "../lib/flavour";
  import { dur } from "../lib/state.svelte";

  // The launcher's opening: the copper block assembles and oxidises from the top
  // down while the name appears. Only transforms and opacity move: filters drop
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
    <svg viewBox="104 82 304 348" width="104" height="119" aria-hidden="true">
      <defs>
        <linearGradient id="boot-copper-left" x1="0" y1="0" x2="0.5" y2="1">
          <stop offset="0" stop-color="#e9a06c" />
          <stop offset="1" stop-color="#b06a3c" />
        </linearGradient>
        <linearGradient id="boot-copper-right" x1="1" y1="0" x2="0.5" y2="1">
          <stop offset="0" stop-color="#c47a47" />
          <stop offset="1" stop-color="#7c4526" />
        </linearGradient>
        <linearGradient id="boot-patina-top" x1="0.1" y1="0" x2="0.9" y2="1">
          <stop offset="0" stop-color="#8ff0d6" />
          <stop offset="1" stop-color="#3fb8a0" />
        </linearGradient>
        <linearGradient id="boot-patina-left" x1="0" y1="0" x2="0.6" y2="1">
          <stop offset="0" stop-color="#3fb8a0" />
          <stop offset="0.58" stop-color="#b06a3c" />
          <stop offset="1" stop-color="#e08a4f" />
        </linearGradient>
        <linearGradient id="boot-patina-right" x1="1" y1="0" x2="0.4" y2="1">
          <stop offset="0" stop-color="#2a8c7c" />
          <stop offset="0.5" stop-color="#7c4526" />
          <stop offset="1" stop-color="#a65f33" />
        </linearGradient>
      </defs>
      <g class="face top">
        <path d="M256 92 398 174 256 256 114 174Z" fill="#e9a06c" />
        <path class="oxide" d="M256 92 398 174 256 256 114 174Z" fill="url(#boot-patina-top)" />
      </g>
      <g class="face left">
        <path d="M114 192 247 269V420L114 343Z" fill="url(#boot-copper-left)" />
        <path class="oxide late" d="M114 192 247 269V420L114 343Z" fill="url(#boot-patina-left)" />
      </g>
      <g class="face right">
        <path d="M398 192 265 269V420L398 343Z" fill="url(#boot-copper-right)" />
        <path class="oxide late" d="M398 192 265 269V420L398 343Z" fill="url(#boot-patina-right)" />
      </g>
    </svg>
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
  .face {
    opacity: 0;
    animation: settle 520ms cubic-bezier(0.2, 0, 0, 1) forwards;
  }
  .top {
    --from: translateY(-46px);
  }
  .left {
    --from: translate(-40px, 24px);
    animation-delay: 110ms;
  }
  .right {
    --from: translate(40px, 24px);
    animation-delay: 220ms;
  }
  .oxide {
    opacity: 0;
    animation: oxidise 640ms ease-out 780ms forwards;
  }
  .oxide.late {
    animation-delay: 1000ms;
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
  @keyframes settle {
    from {
      opacity: 0;
      transform: var(--from);
    }
    to {
      opacity: 1;
      transform: none;
    }
  }
  @keyframes oxidise {
    to {
      opacity: 1;
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
    .face,
    .oxide,
    .name,
    .fill {
      animation-duration: 1ms;
      animation-delay: 0ms;
    }
  }
</style>
