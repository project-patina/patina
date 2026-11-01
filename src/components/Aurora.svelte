<script lang="ts">
  // Slow drifting colour blobs used behind the welcome screens.
  let { intensity = 1 }: { intensity?: number } = $props();
</script>

<div class="aurora" style:opacity={intensity} aria-hidden="true">
  <span class="b1"></span>
  <span class="b2"></span>
  <span class="b3"></span>
</div>

<style>
  .aurora {
    position: absolute;
    inset: 0;
    overflow: hidden;
    pointer-events: none;
  }
  span {
    position: absolute;
    border-radius: 50%;
    opacity: 0.55;
    /* Soft edge from the gradient itself: a CSS blur filter here cost frames in WebKitGTK. */
    background: radial-gradient(closest-side, var(--c), transparent);
    will-change: transform;
  }
  .b1 {
    width: 760px;
    height: 760px;
    left: -140px;
    top: -160px;
    --c: var(--accent);
    animation: d1 18s ease-in-out infinite alternate;
  }
  .b2 {
    width: 700px;
    height: 700px;
    right: -120px;
    bottom: -180px;
    --c: color-mix(in srgb, var(--accent) 50%, #0ea5e9);
    animation: d2 22s ease-in-out infinite alternate;
  }
  .b3 {
    width: 520px;
    height: 520px;
    left: 45%;
    top: 40%;
    --c: color-mix(in srgb, var(--accent) 40%, #ec4899);
    opacity: 0.3;
    animation: d3 26s ease-in-out infinite alternate;
  }
  :global([data-theme="light"]) span {
    opacity: 0.28;
  }
  @keyframes d1 {
    to {
      transform: translate(180px, 120px) scale(1.15);
    }
  }
  @keyframes d2 {
    to {
      transform: translate(-200px, -90px) scale(0.9);
    }
  }
  @keyframes d3 {
    to {
      transform: translate(-160px, 80px) scale(1.3);
    }
  }
</style>
