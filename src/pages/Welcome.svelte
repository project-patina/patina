<script lang="ts">
  import { fly, fade, scale } from "svelte/transition";
  import { ArrowLeft, ArrowRight, Check, Monitor, Moon, Rocket, Sun } from "@lucide/svelte";
  import Aurora from "../components/Aurora.svelte";
  import LoginPanel from "../components/LoginPanel.svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import Head from "../components/Head.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { api } from "../lib/api";
  import { ACCENTS } from "../lib/presets";
  import { activeAccount, app, dur, loadVersions, refreshInstances, toastError, updateSettings } from "../lib/state.svelte";

  let { startAt = "hello", ondone }: { startAt?: "hello" | "login"; ondone: () => void } = $props();

  const steps = ["hello", "theme", "login", "ready"] as const;
  type Step = (typeof steps)[number];
  // svelte-ignore state_referenced_locally
  let step = $state<Step>(startAt);
  let dir = $state(1);
  let quickInstance = $state(true);
  let busy = $state(false);

  const acc = $derived(activeAccount());
  const skin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);
  const s = $derived(app.settings!);

  function go(to: Step) {
    dir = steps.indexOf(to) > steps.indexOf(step) ? 1 : -1;
    step = to;
  }

  async function finish() {
    busy = true;
    try {
      if (quickInstance && app.instances.length === 0) {
        const v = await loadVersions();
        await api.createInstance({ name: `Fabric ${v.latestRelease}`, mcVersion: v.latestRelease, loader: "fabric", icon: "diamond", banner: "aurora" });
        await refreshInstances();
      }
      updateSettings({ onboarded: true });
      ondone();
    } catch (e) {
      toastError(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="welcome">
  <Aurora />

  <div class="dots">
    {#each steps as st}
      <span class:on={st === step} class:past={steps.indexOf(st) < steps.indexOf(step)}></span>
    {/each}
  </div>

  {#key step}
    <section class="step" in:fly={{ x: 60 * dir, duration: dur(520), delay: dur(120), opacity: 0 }} out:fly={{ x: -40 * dir, duration: dur(220), opacity: 0 }}>
      {#if step === "hello"}
        <div class="center">
          <div class="logo" in:scale={{ start: 0.6, duration: dur(900), opacity: 0 }}>
            <img src="/icon.svg" alt="" width="96" height="96" />
          </div>
          <h1 class="big">Witaj w <span class="grad">Patina</span></h1>
          <p class="lead">Lekki launcher Minecraft. Logujesz się, wybierasz wersję i grasz — Fabric z modami optymalizacyjnymi gotowy od pierwszego uruchomienia.</p>
          <button class="btn primary cta" onclick={() => go("theme")}>Zaczynamy <ArrowRight size={17} /></button>
        </div>
      {:else if step === "theme"}
        <div class="center">
          <h2 class="title">Jak ma wyglądać?</h2>
          <p class="lead">Zmienisz to w każdej chwili w ustawieniach.</p>
          <div class="themes">
            {#each [{ id: "dark", label: "Ciemny", icon: Moon }, { id: "light", label: "Jasny", icon: Sun }, { id: "system", label: "Systemowy", icon: Monitor }] as t (t.id)}
              <button class="theme" class:on={s.theme === t.id} onclick={() => updateSettings({ theme: t.id as any })}>
                <div class="preview {t.id}">
                  <div class="p-rail"></div>
                  <div class="p-main"><div class="p-hero"></div><div class="p-line"></div><div class="p-line short"></div></div>
                </div>
                <div class="t-label"><t.icon size={15} /> {t.label} {#if s.theme === t.id}<Check size={15} class="ok" />{/if}</div>
              </button>
            {/each}
          </div>
          <div class="accents">
            {#each ACCENTS as c}
              <button class="swatch" class:on={s.accent === c} style:background={c} aria-label="Kolor {c}" onclick={() => updateSettings({ accent: c })}></button>
            {/each}
          </div>
          <div class="nav">
            <button class="btn ghost" onclick={() => go("hello")}><ArrowLeft size={16} /> Wstecz</button>
            <button class="btn primary" onclick={() => go(acc ? "ready" : "login")}>Dalej <ArrowRight size={16} /></button>
          </div>
        </div>
      {:else if step === "login"}
        <div class="center narrow">
          <h2 class="title">Zaloguj się</h2>
          <p class="lead">Użyj konta Microsoft, na którym masz Minecraft: Java Edition.</p>
          <div class="login card glass">
            <LoginPanel onsuccess={() => go("ready")} />
          </div>
          {#if startAt === "hello"}
            <button class="btn ghost" onclick={() => go("theme")}><ArrowLeft size={16} /> Wstecz</button>
          {/if}
        </div>
      {:else if step === "ready" && acc}
        <div class="ready">
          <div class="model">
            {#if skin}
              <SkinViewer skin={skin} slim={acc.skinSlim} width={280} height={380} />
            {:else}
              <Head name={acc.name} size={120} radius={24} />
            {/if}
          </div>
          <div class="ready-text">
            <span class="chip accent" in:fade={{ delay: dur(500) }}>Zalogowano</span>
            <h1 class="big">Cześć, <span class="grad">{acc.name}</span>!</h1>
            <p class="lead left">Wszystko gotowe. Gra i Java pobiorą się same przy pierwszym uruchomieniu.</p>
            {#if app.instances.length === 0}
              <label class="quick card">
                <Toggle bind:checked={quickInstance} />
                <div>
                  <div class="q-title">Utwórz instancję Fabric (najnowsza wersja)</div>
                  <div class="dim q-desc">z Sodium, Lithium, FerriteCore i innymi modami na FPS</div>
                </div>
              </label>
            {/if}
            <button class="btn primary cta" disabled={busy} onclick={finish}><Rocket size={17} /> Wchodzę do gry</button>
          </div>
        </div>
      {/if}
    </section>
  {/key}
</div>

<style>
  .welcome {
    position: relative;
    flex: 1;
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  .dots {
    position: absolute;
    top: 14px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 6px;
    z-index: 2;
  }
  .dots span {
    width: 6px;
    height: 6px;
    border-radius: 6px;
    background: var(--border-2);
    transition: width 0.4s var(--ease), background 0.3s;
  }
  .dots span.past {
    background: var(--accent-line);
  }
  .dots span.on {
    width: 22px;
    background: var(--accent);
  }
  .step {
    grid-area: 1 / 1;
    position: relative;
    z-index: 1;
    width: 100%;
    display: grid;
    place-items: center;
    padding: 24px;
  }
  .center {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 16px;
    max-width: 640px;
  }
  .narrow {
    max-width: 420px;
    width: 100%;
  }
  .logo {
    animation: float 6s ease-in-out infinite;
    border-radius: 22px;
    box-shadow: 0 22px 60px -10px color-mix(in srgb, var(--accent) 60%, transparent);
    margin-bottom: 8px;
  }
  .big {
    font-size: 3.2rem;
    font-weight: 800;
    line-height: 1.05;
    letter-spacing: -0.03em;
  }
  .title {
    font-size: 2.2rem;
    font-weight: 750;
  }
  .grad {
    background: linear-gradient(100deg, var(--text) 10%, var(--accent) 90%);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .lead {
    color: var(--text-2);
    font-size: 1.05rem;
    line-height: 1.55;
    margin: 0;
    max-width: 520px;
  }
  .lead.left {
    text-align: left;
  }
  .cta {
    height: 46px;
    padding: 0 22px;
    font-size: 1rem;
    border-radius: var(--radius);
    margin-top: 8px;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(3, 180px);
    gap: 14px;
    margin-top: 8px;
  }
  .theme {
    border-radius: var(--radius-l);
    padding: 10px;
    background: var(--panel);
    border: 1px solid var(--border);
    transition: border-color 0.2s, transform 0.25s var(--ease), box-shadow 0.25s;
  }
  .theme:hover {
    transform: translateY(-3px);
  }
  .theme.on {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .preview {
    height: 100px;
    border-radius: 10px;
    display: flex;
    gap: 6px;
    padding: 8px;
    overflow: hidden;
  }
  .preview.dark {
    background: #0b0b0e;
  }
  .preview.light {
    background: #f3f3f5;
  }
  .preview.system {
    background: linear-gradient(135deg, #0b0b0e 50%, #f3f3f5 50%);
  }
  .p-rail {
    width: 14px;
    border-radius: 5px;
    background: color-mix(in srgb, gray 30%, transparent);
  }
  .p-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .p-hero {
    flex: 1;
    border-radius: 6px;
    background: linear-gradient(120deg, var(--accent), color-mix(in srgb, var(--accent) 30%, #0ea5e9));
  }
  .p-line {
    height: 8px;
    border-radius: 4px;
    background: color-mix(in srgb, gray 35%, transparent);
  }
  .p-line.short {
    width: 60%;
  }
  .t-label {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 10px 4px 2px;
    font-weight: 600;
  }
  .t-label :global(.ok) {
    margin-left: auto;
    color: var(--accent);
  }
  .accents {
    display: flex;
    gap: 10px;
    margin-top: 6px;
  }
  .swatch {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    border: 2px solid var(--bg);
    box-shadow: 0 0 0 1px var(--border-2);
    transition: transform 0.25s var(--ease-spring), box-shadow 0.2s;
  }
  .swatch:hover {
    transform: scale(1.15);
  }
  .swatch.on {
    box-shadow: 0 0 0 2px var(--text);
    transform: scale(1.1);
  }
  .nav {
    display: flex;
    gap: 10px;
    margin-top: 14px;
  }
  .login {
    width: 100%;
    padding: 22px;
    margin-top: 6px;
  }
  .ready {
    display: flex;
    align-items: center;
    gap: 40px;
    max-width: 860px;
  }
  .model {
    position: relative;
  }
  .model::after {
    content: "";
    position: absolute;
    left: 15%;
    right: 15%;
    bottom: 2px;
    height: 36px;
    border-radius: 50%;
    background: radial-gradient(closest-side, rgb(0 0 0 / 0.35), transparent);
    z-index: -1;
  }
  .ready-text {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 14px;
  }
  .quick {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
    cursor: pointer;
    text-align: left;
  }
  .q-title {
    font-weight: 600;
  }
  .q-desc {
    font-size: 0.85rem;
    margin-top: 2px;
  }
</style>
