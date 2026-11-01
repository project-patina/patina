<script lang="ts">
  import { onDestroy } from "svelte";
  import { Raycaster, Vector2, type Object3D } from "three";
  import { TransformControls } from "three/examples/jsm/controls/TransformControls.js";
  import type { SkinViewer as Viewer } from "skinview3d";
  import { ArrowLeft, Move3d, Pause, Play, RotateCcw, Rotate3d } from "@lucide/svelte";
  import SkinViewerView from "../components/SkinViewer.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { activeAccount, app, navigate, toast, updateSettings } from "../lib/state.svelte";
  import { DEFAULT_POSE, PARTS, parsePose, type PartKey, type PartPose, type Pose } from "../lib/pose";
  import type { Gear } from "../lib/playerGear";

  const DEG = Math.PI / 180;
  const keys = Object.keys(PARTS) as PartKey[];

  const acc = $derived(activeAccount());
  const skin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);

  let pose = $state<Pose>(parsePose(app.settings?.modelPose));
  let selected = $state<PartKey>("rightArm");
  let mode = $state<"rotate" | "translate">("rotate");
  let animate = $state(false);

  const current = $derived(pose[selected]);
  const isItem = $derived(selected === "sword" || selected === "shield");

  let viewer: Viewer | null = null;
  let gear: () => Gear | null = () => null;
  let gizmo: TransformControls | null = null;

  function save() {
    updateSettings({ modelPose: JSON.stringify($state.snapshot(pose)) });
  }

  function objectOf(key: PartKey): Object3D | null {
    if (!viewer) return null;
    if (key === "sword" || key === "shield") return gear()?.[key] ?? null;
    return viewer.playerObject.skin[key];
  }

  function attach() {
    const target = objectOf(selected);
    if (!gizmo) return;
    if (target) gizmo.attach(target);
    else gizmo.detach();
    gizmo.setMode(mode);
  }

  const round = (v: number, step: number) => Math.round(v / step) * step;

  function pullFromGizmo() {
    const target = objectOf(selected);
    if (!target) return;
    const p = pose[selected];
    p.rx = round(target.rotation.x / DEG, 0.5);
    p.ry = round(target.rotation.y / DEG, 0.5);
    p.rz = round(target.rotation.z / DEG, 0.5);
    p.x = round(target.position.x, 0.05);
    p.y = round(target.position.y, 0.05);
    p.z = round(target.position.z, 0.05);
    save();
  }

  function pick(event: PointerEvent) {
    if (!viewer || gizmo?.axis) return;
    const box = viewer.canvas.getBoundingClientRect();
    const at = new Vector2(((event.clientX - box.left) / box.width) * 2 - 1, -((event.clientY - box.top) / box.height) * 2 + 1);
    const ray = new Raycaster();
    ray.setFromCamera(at, viewer.camera);
    const targets = keys.map(objectOf).filter((o): o is Object3D => !!o);
    const hit = ray.intersectObjects(targets, true)[0];
    if (!hit) return;
    let o: Object3D | null = hit.object;
    while (o) {
      const key = keys.find((k) => objectOf(k) === o);
      if (key) {
        selected = key;
        return;
      }
      o = o.parent;
    }
  }

  function ready(v: Viewer, items: () => Gear | null) {
    viewer = v;
    gear = items;
    v.controls.enableZoom = true;
    gizmo = new TransformControls(v.camera, v.canvas);
    gizmo.setSpace("local");
    gizmo.setSize(0.8);
    gizmo.addEventListener("dragging-changed", (e) => (v.controls.enabled = !e.value));
    gizmo.addEventListener("objectChange", pullFromGizmo);
    v.scene.add(gizmo);
    v.canvas.addEventListener("pointerdown", pick);
    attach();
  }

  $effect(() => {
    selected;
    mode;
    attach();
  });

  // The items load a moment after the model.
  const retry = setInterval(() => {
    if (gizmo && !gizmo.object) attach();
  }, 300);

  onDestroy(() => {
    clearInterval(retry);
    viewer?.canvas.removeEventListener("pointerdown", pick);
    gizmo?.detach();
    gizmo?.dispose();
  });

  function set(field: keyof PartPose, value: number) {
    pose[selected][field] = value;
    save();
  }

  function resetPart() {
    pose[selected] = structuredClone(DEFAULT_POSE[selected]);
    save();
  }

  function resetAll() {
    pose = structuredClone(DEFAULT_POSE);
    save();
    toast("Przywrócono domyślną pozę");
  }

  const rotation = [["rx", "Obrót X"], ["ry", "Obrót Y"], ["rz", "Obrót Z"]] as const;
  const position = [["x", "W bok"], ["y", "Góra / dół"], ["z", "Przód / tył"]] as const;
</script>

<div class="editor">
  <header>
    <button class="btn sm" onclick={() => navigate("home")}><ArrowLeft size={15} /> Wróć</button>
    <h2>Edytor pozy</h2>
    <span class="dim hint">Kliknij część ciała i ciągnij koła. „Widok z menu” pokazuje postać tak, jak na ekranie głównym.</span>
  </header>

  <div class="work">
    <div class="stage card">
      {#if skin}
        <SkinViewerView {skin} slim={acc?.skinSlim ?? false} armed pose={() => pose} still={!animate} followCursor={false} width={430} height={560} zoom={0.72} onready={ready} />
      {:else}
        <p class="dim">Zaloguj się, żeby zobaczyć swoją postać.</p>
      {/if}
    </div>

    <aside class="panel">
      <div class="chips">
        {#each keys as key (key)}
          <button class="chip" class:accent={selected === key} onclick={() => (selected = key)}>{PARTS[key]}</button>
        {/each}
      </div>

      <div class="tools">
        <div class="seg">
          <button class:on={mode === "rotate"} onclick={() => (mode = "rotate")}><Rotate3d size={14} /> Obracaj</button>
          <button class:on={mode === "translate"} onclick={() => (mode = "translate")}><Move3d size={14} /> Przesuwaj</button>
        </div>
        <label class="anim">{#if animate}<Play size={14} />{:else}<Pause size={14} />{/if} Animacja <Toggle bind:checked={animate} /></label>
      </div>

      <h3>{PARTS[selected]}</h3>
      {#each rotation as [field, label] (field)}
        <label class="slider">
          <span>{label}</span>
          <input type="range" min="-180" max="180" step="0.5" value={current[field]} oninput={(e) => set(field, +e.currentTarget.value)} />
          <span class="mono">{current[field].toFixed(1)}°</span>
        </label>
      {/each}
      {#each position as [field, label] (field)}
        <label class="slider">
          <span>{label}</span>
          <input type="range" min="-16" max="16" step="0.05" value={current[field]} oninput={(e) => set(field, +e.currentTarget.value)} />
          <span class="mono">{current[field].toFixed(2)}</span>
        </label>
      {/each}
      {#if isItem}
        <label class="slider">
          <span>Rozmiar</span>
          <input type="range" min="20" max="150" step="1" value={current.size ?? 74} oninput={(e) => set("size", +e.currentTarget.value)} />
          <span class="mono">{current.size ?? 74}%</span>
        </label>
      {/if}
      <label class="slider">
        <span>Wysokość</span>
        <input type="range" min="-6" max="2" step="0.05" value={pose.sink} oninput={(e) => { pose.sink = +e.currentTarget.value; save(); }} />
        <span class="mono">{pose.sink.toFixed(2)}</span>
      </label>

      <label class="slider">
        <span>Obrót postaci</span>
        <input type="range" min="-180" max="180" step="1" value={pose.yaw} oninput={(e) => { pose.yaw = +e.currentTarget.value; save(); }} />
        <span class="mono">{pose.yaw}°</span>
      </label>

      <div class="actions">
        <button class="btn sm" onclick={() => viewer?.controls.reset()}>Widok z menu</button>
        <button class="btn sm" onclick={resetPart}><RotateCcw size={14} /> Resetuj część</button>
        <button class="btn sm ghost" onclick={resetAll}>Resetuj wszystko</button>
      </div>
    </aside>
  </div>
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 20px;
    height: 100%;
    min-height: 0;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }
  .hint {
    font-size: 0.85rem;
  }
  .work {
    display: flex;
    gap: 14px;
    flex: 1;
    min-height: 0;
  }
  .stage {
    flex: 1;
    min-width: 280px;
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  .panel {
    width: 320px;
    flex-shrink: 0;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-right: 6px;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 4px;
  }
  .seg {
    display: flex;
    gap: 3px;
    background: var(--panel-2);
    border-radius: var(--radius);
    padding: 3px;
  }
  .seg button {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 5px 9px;
    border-radius: var(--radius);
    font-size: 0.82rem;
    color: var(--text-3);
  }
  .seg button.on {
    background: var(--accent-soft);
    color: var(--text);
  }
  .anim {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.82rem;
  }
  h3 {
    margin-top: 6px;
  }
  .slider {
    display: grid;
    grid-template-columns: 82px 1fr 58px;
    align-items: center;
    gap: 8px;
    font-size: 0.82rem;
  }
  .slider input {
    width: 100%;
    accent-color: var(--accent);
  }
  .slider .mono {
    text-align: right;
    color: var(--text-3);
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
  }
</style>
