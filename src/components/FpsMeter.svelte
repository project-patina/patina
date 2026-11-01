<script lang="ts">
  // Dev-only frame counter (rendered only when import.meta.env.DEV).
  let fps = $state(0);
  let worst = $state(0);
  $effect(() => {
    let frames = 0;
    let last = performance.now();
    let prev = last;
    let maxGap = 0;
    let raf = 0;
    const tick = (t: number) => {
      frames++;
      maxGap = Math.max(maxGap, t - prev);
      prev = t;
      if (t - last >= 1000) {
        fps = Math.round((frames * 1000) / (t - last));
        worst = Math.round(maxGap);
        frames = 0;
        maxGap = 0;
        last = t;
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  });
</script>

<div class="fps mono">{fps} FPS · max {worst}ms</div>

<style>
  .fps {
    position: fixed;
    left: 8px;
    bottom: 8px;
    z-index: 999;
    padding: 3px 7px;
    border-radius: 6px;
    background: #000;
    color: #0f0;
    font-size: 12px;
    pointer-events: none;
  }
</style>
