<script lang="ts">
  import { ArrowLeft, ArrowRight, Check, LoaderCircle } from "@lucide/svelte";
  import LoginPanel from "../components/LoginPanel.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import Head from "../components/Head.svelte";
  import ThemePicker from "../components/ThemePicker.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { api } from "../lib/api";
  import { lang, t } from "../lib/i18n.svelte";
  import { page } from "../lib/motion";
  import { ACCENTS } from "../lib/presets";
  import { activeAccount, app, loadVersions, refreshInstances, toastError, updateSettings } from "../lib/state.svelte";

  let { startAt = "hello", ondone }: { startAt?: "hello" | "login"; ondone: () => void } = $props();

  const steps = ["language", "look", "login", "ready"] as const;
  type Step = (typeof steps)[number];
  // Each language is named in itself: it has to be readable before one is chosen.
  const LANGUAGES = [{ id: "pl", name: "Polski" }, { id: "en", name: "English" }];

  // svelte-ignore state_referenced_locally
  let step = $state<Step>(startAt === "login" ? "login" : "language");
  let quickInstance = $state(true);
  let busy = $state(false);

  const acc = $derived(activeAccount());
  const skin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);
  const s = $derived(app.settings!);
  const modes = $derived([
    { value: "dark" as const, label: t("look.dark") },
    { value: "light" as const, label: t("look.light") },
    { value: "system" as const, label: t("general.system") },
  ]);

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
  {#if startAt === "hello"}
    <div class="dots" aria-hidden="true">
      {#each steps as st (st)}<span class:on={st === step}></span>{/each}
    </div>
  {/if}

  {#key step}
    <section class="step" in:page>
      {#if step === "language"}
        <img src="/icon.svg" alt="" width="48" height="48" />
        <h1 class="page-title">{t("welcome.language")}</h1>
        <div class="langs">
          {#each LANGUAGES as l (l.id)}
            <button class="lang" aria-pressed={lang() === l.id} onclick={() => updateSettings({ language: l.id })}>{l.name}</button>
          {/each}
        </div>
        <div class="nav">
          <button class="btn primary" onclick={() => { updateSettings({ language: lang() }); step = "look"; }}>{t("common.next")} <ArrowRight size={14} /></button>
        </div>
      {:else if step === "look"}
        <h1 class="page-title">{t("welcome.look")}</h1>
        <p class="lead">{t("welcome.lookHint")}</p>
        <div class="look">
          <ThemePicker />
          <Row title={t("look.mode")}>
            <Segmented label={t("look.mode")} value={s.theme} options={modes} onchange={(theme) => updateSettings({ theme })} />
          </Row>
          <Row title={t("look.accent")}>
            <div class="accents">
              {#each ACCENTS.slice(0, 6) as colour (colour)}
                <button class="swatch" style:background={colour} aria-label={colour} aria-pressed={s.accent === colour} onclick={() => updateSettings({ accent: colour })}>
                  {#if s.accent === colour}<Check size={12} />{/if}
                </button>
              {/each}
            </div>
          </Row>
        </div>
        <div class="nav">
          <button class="btn ghost" onclick={() => (step = "language")}><ArrowLeft size={14} /> {t("common.back")}</button>
          <button class="btn primary" onclick={() => (step = acc ? "ready" : "login")}>{t("common.next")} <ArrowRight size={14} /></button>
        </div>
      {:else if step === "login"}
        <h1 class="page-title">{t("welcome.login")}</h1>
        <p class="lead">{t("welcome.loginHint")}</p>
        <div class="login"><LoginPanel onsuccess={() => (step = "ready")} /></div>
        {#if startAt === "hello"}
          <div class="nav"><button class="btn ghost" onclick={() => (step = "look")}><ArrowLeft size={14} /> {t("common.back")}</button></div>
        {/if}
      {:else if step === "ready" && acc}
        <div class="ready">
          {#if skin}
            <SkinViewer {skin} slim={acc.skinSlim} width={220} height={320} />
          {:else}
            <Head name={acc.name} size={96} radius={8} />
          {/if}
          <div class="ready-text">
            <h1 class="page-title">{t("account.loggedIn", { name: acc.name })}</h1>
            <p class="lead">{t("welcome.readyHint")}</p>
            {#if app.instances.length === 0}
              <Row title={t("welcome.quickInstance")} desc={t("welcome.quickInstanceDesc")}>
                <Toggle label={t("welcome.quickInstance")} bind:checked={quickInstance} />
              </Row>
            {/if}
            <button class="btn primary finish" disabled={busy} onclick={finish}>
              {#if busy}<LoaderCircle size={14} class="spin" />{/if} {t("welcome.finish")}
            </button>
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
    min-width: 0;
    display: grid;
    place-items: center;
    padding: 32px 24px;
    overflow: auto;
  }
  .dots {
    position: absolute;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 8px;
  }
  .dots span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--border-2);
    transition: background 0.12s;
  }
  .dots span.on { background: var(--accent); }
  .step {
    grid-area: 1 / 1;
    width: 100%;
    max-width: 440px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    text-align: center;
  }
  .step:has(.ready) { max-width: 640px; }
  .lead {
    margin: 0;
    color: var(--text-2);
    font-size: var(--fs-13);
    line-height: 1.5;
  }
  .langs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    width: 100%;
    margin-top: 8px;
  }
  .lang {
    height: 56px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    background: var(--panel);
    font-size: var(--fs-14);
    font-weight: 500;
    transition: border-color 0.12s;
  }
  .lang:hover { border-color: var(--border-2); }
  .lang[aria-pressed="true"] { border-color: var(--accent); }
  .nav {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-top: 8px;
  }
  .look { width: 100%; text-align: left; }
  .accents { display: flex; gap: 4px; }
  .swatch {
    width: 24px;
    height: 24px;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    color: #101214;
    box-shadow: inset 0 0 0 1px rgb(128 128 128 / 0.3);
  }
  .swatch[aria-pressed="true"] { outline: 2px solid var(--text); outline-offset: 1px; }
  .login { width: 100%; max-width: 360px; margin-top: 8px; }
  .ready {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 32px;
    width: 100%;
    text-align: left;
  }
  .ready-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  .ready-text :global(.row) { width: 100%; }
  .finish { height: 36px; padding: 0 16px; }
</style>
