<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { LinearSRGBColorSpace } from "three";
  import { SkinViewer } from "skinview3d";
  import { DEFAULT_POSE, type Pose } from "../lib/pose";
  import type { Gear } from "../lib/playerGear";
  import { makeAnimation, type AnyAnimation, type LookTarget } from "../lib/skinAnimations";

  let {
    skin,
    cape = null,
    slim = false,
    width = 260,
    height = 360,
    zoom = 0.85,
    interactive = true,
    followCursor = true,
    armed = false,
    pose = () => DEFAULT_POSE,
    still = false,
    onready,
  }: {
    skin: string;
    cape?: string | null;
    slim?: boolean;
    width?: number;
    height?: number;
    zoom?: number;
    interactive?: boolean;
    followCursor?: boolean;
    armed?: boolean;
    /** The armed stance, read every frame. */
    pose?: () => Pose;
    /** Hold the stance exactly, for the pose editor. */
    still?: boolean;
    onready?: (viewer: SkinViewer, gear: () => Gear | null) => void;
  } = $props();

  let canvas: HTMLCanvasElement;
  let viewer: SkinViewer | null = null;
  let anim: AnyAnimation | null = null;
  let loaded = $state(false);
  const look: LookTarget = { x: 0, y: 0, at: -1e9 };
  const BODY_YAW = -0.45;

  // WebGL frames are expensive in WebKitGTK; the blocky animations look the
  // same at 30 FPS, so render at that rate and leave the rest of the UI at full
  // refresh. Nothing is drawn while the canvas is off-screen or the window hidden.
  const MODEL_FPS = 30;
  let raf = 0;
  let visible = true;
  let lastDraw = 0;

  function loop(t: number) {
    raf = requestAnimationFrame(loop);
    const v = viewer;
    if (!v || !visible || document.hidden || t - lastDraw < 1000 / MODEL_FPS - 2) return;
    const dt = Math.min(0.1, lastDraw ? (t - lastDraw) / 1000 : 0);
    lastDraw = t;
    v.animation?.update(v.playerObject, dt);
    v.controls.update();
    // Straight to the canvas: skips skinview3d's FXAA post-process pass, which
    // costs more than the scene itself and only blurs pixel art.
    v.renderer.render(v.scene, v.camera);
  }

  function onPointer(e: PointerEvent) {
    if (!followCursor || !canvas) return;
    const r = canvas.getBoundingClientRect();
    // Head height is about a fifth from the top of the canvas.
    look.x = (e.clientX - (r.left + r.width / 2)) / Math.max(300, window.innerWidth / 2);
    look.y = (e.clientY - (r.top + r.height * 0.22)) / Math.max(300, window.innerHeight / 2);
    look.at = performance.now();
  }

  function setAnimation() {
    if (!viewer) return;
    // Head yaw is relative to the body, which stands turned away from the camera.
    anim?.dispose?.();
    // Every animation starts from the plain pose the last one may have changed.
    const p = viewer.playerObject;
    p.resetJoints();
    p.skin.body.scale.set(1, 1, 1);
    p.skin.body.position.x = 0;
    p.skin.head.position.set(0, 0, 0);
    p.position.y = 0;
    anim = makeAnimation(look, () => viewer!.playerObject.rotation.y, armed ? { pose, still: () => still } : null);
    viewer.animation = anim;
  }

  onMount(() => {
    viewer = new SkinViewer({ canvas, width, height, zoom, fov: 40, preserveDrawingBuffer: false, renderPaused: true, pixelRatio: 1 });
    viewer.background = null as any;
    viewer.autoRotate = false;
    viewer.controls.enableZoom = false;
    viewer.controls.enablePan = false;
    viewer.controls.enableRotate = interactive;
    viewer.playerObject.rotation.y = BODY_YAW;
    // skinview3d leaves textures unconverted and draws through a composer that adds no
    // gamma. Drawn straight to the canvas, three.js would add it and wash the skin out.
    viewer.renderer.outputColorSpace = LinearSRGBColorSpace;
    setAnimation();
    onready?.(viewer, () => anim?.gear?.() ?? null);
    const io = new IntersectionObserver(([e]) => (visible = e.isIntersecting));
    io.observe(canvas);
    window.addEventListener("pointermove", onPointer, { passive: true });
    raf = requestAnimationFrame(loop);
    return () => {
      io.disconnect();
      window.removeEventListener("pointermove", onPointer);
      cancelAnimationFrame(raf);
    };
  });

  $effect(() => {
    const v = viewer;
    if (!v || !skin) return;
    loaded = false;
    v.loadSkin(skin, { model: slim ? "slim" : "default" }).then(() => (loaded = true));
  });

  $effect(() => {
    const v = viewer;
    if (!v) return;
    if (cape) v.loadCape(cape);
    else v.loadCape(null);
  });

  $effect(() => {
    viewer?.setSize(width, height);
  });

  onDestroy(() => {
    anim?.dispose?.();
    viewer?.dispose();
  });
</script>

<canvas bind:this={canvas} class:loaded onclick={() => anim?.strike?.()} title={armed ? "Kliknij, żeby zaatakował" : undefined}></canvas>

<style>
  canvas {
    display: block;
    opacity: 0;
    transform: translateY(12px) scale(0.98);
    transition: opacity 0.6s var(--ease), transform 0.8s var(--ease);
    cursor: grab;
  }
  canvas:active {
    cursor: grabbing;
  }
  canvas.loaded {
    opacity: 1;
    transform: none;
  }
</style>
