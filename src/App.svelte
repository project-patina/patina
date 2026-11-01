<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fly, fade } from "svelte/transition";
  import Boot from "./components/Boot.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import Toasts from "./components/Toasts.svelte";
  import NewInstanceModal from "./components/NewInstanceModal.svelte";
  import CrashModal from "./components/CrashModal.svelte";
  import Modal from "./components/Modal.svelte";
  import LoginPanel from "./components/LoginPanel.svelte";
  import Welcome from "./pages/Welcome.svelte";
  import Home from "./pages/Home.svelte";
  import News from "./pages/News.svelte";
  import Instances from "./pages/Instances.svelte";
  import InstanceView from "./pages/InstanceView.svelte";
  import Settings from "./pages/Settings.svelte";
  import Discover from "./pages/Discover.svelte";
  import Wardrobe from "./pages/Wardrobe.svelte";
  import PoseEditor from "./pages/PoseEditor.svelte";
  import ProjectModal from "./components/ProjectModal.svelte";
  import { refCss } from "./lib/images";
  import FpsMeter from "./components/FpsMeter.svelte";
  import { api, errText } from "./lib/api";
  import { activeAccount, app, dur, flushSettings, init, toast, toastError } from "./lib/state.svelte";
  import { createCloseHandler } from "./lib/windowClose";

  let bootError = $state("");
  let booting = $state(true);
  let addAccount = $state(false);
  // Onboarding stays open until it calls ondone, even after login sets an account.
  let onboarding = $state<null | "hello" | "login">(null);

  onMount(() => {
    let disposed = false;
    let cleanup: (() => void) | undefined;
    let closeCleanup: (() => void) | undefined;
    init()
      .then((stop) => {
        if (disposed) { stop(); return; }
        cleanup = stop;
        if (!app.settings!.onboarded) onboarding = "hello";
        else if (!activeAccount()) onboarding = "login";
      })
      .catch((e) => (bootError = errText(e)));
    void getCurrentWindow().onCloseRequested(createCloseHandler(flushSettings))
      .then((stop) => { if (disposed) stop(); else closeCleanup = stop; })
      .catch(toastError);
    return () => { disposed = true; cleanup?.(); closeCleanup?.(); };
  });

  // Settings → "Pokaż przewodnik" flips onboarded back.
  $effect(() => {
    if (app.ready && app.settings && !app.settings.onboarded && !onboarding) onboarding = "hello";
  });
  // Logging out of the last account brings back the login step.
  $effect(() => {
    if (app.ready && !onboarding && app.accounts.length === 0) onboarding = "login";
  });
</script>

{#if app.settings?.appWallpaper}
  <div class="app-wall" style:background={refCss(app.settings.appWallpaper)}>
    <div class="app-wall-dim" style:opacity={app.settings.appWallpaperDim / 100}></div>
  </div>
{/if}
<div class="shell" class:walled={!!app.settings?.appWallpaper}>
  <TitleBar showAccount={!onboarding} onAddAccount={() => (addAccount = true)} />

  {#if bootError}
    <div class="boot">
      <p>Nie udało się uruchomić launchera:</p>
      <pre class="selectable">{bootError}</pre>
    </div>
  {:else if !app.ready}
    <div class="boot"></div>
  {:else if onboarding}
    <div class="body" in:fade={{ duration: dur(300) }}>
      <Welcome startAt={onboarding} ondone={() => (onboarding = null)} />
    </div>
  {:else}
    <div class="body" in:fade={{ duration: dur(400) }}>
      <Sidebar />
      <main>
        {#key app.page + (app.page === "instance" ? app.instanceId : "")}
          <div class="page" in:fly={{ y: 14, duration: dur(380), delay: dur(60) }} out:fade={{ duration: dur(120) }}>
            {#if app.page === "home"}
              <Home />
            {:else if app.page === "news"}
              <News />
            {:else if app.page === "instances"}
              <Instances />
            {:else if app.page === "instance"}
              <InstanceView />
            {:else if app.page === "discover"}
              <Discover />
            {:else if app.page === "wardrobe"}
              <Wardrobe />
            {:else if app.page === "pose"}
              <PoseEditor />
            {:else}
              <Settings />
            {/if}
          </div>
        {/key}
      </main>
    </div>
  {/if}

  {#if app.newInstanceOpen}
    <NewInstanceModal />
  {/if}
  {#if app.project}
    <ProjectModal />
  {/if}
  {#if app.crash}
    <CrashModal />
  {/if}
  {#if addAccount}
    <Modal onclose={() => { api.cancelLogin(); addAccount = false; }} width={420}>
      <h3 style="margin-bottom: 16px">Dodaj konto</h3>
      <LoginPanel onsuccess={(a) => { addAccount = false; toast(`Zalogowano jako ${a.name}`, "success"); }} />
    </Modal>
  {/if}
  <Toasts />
  {#if booting && !bootError}<Boot ready={app.ready} ondone={() => (booting = false)} />{/if}
  {#if import.meta.env.DEV && new URLSearchParams(location.search).has("fps")}<FpsMeter />{/if}
</div>

<style>
  .shell {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  .app-wall {
    position: fixed;
    inset: 0;
    z-index: 0;
  }
  .app-wall-dim {
    position: absolute;
    inset: 0;
    background: var(--bg);
  }
  .shell {
    position: relative;
    z-index: 1;
  }
  .shell.walled {
    background: transparent;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  main {
    flex: 1;
    min-width: 0;
    display: grid;
    min-height: 0;
  }
  .page {
    grid-area: 1 / 1;
    min-height: 0;
    min-width: 0;
  }
  .boot {
    flex: 1;
    display: grid;
    place-items: center;
    align-content: center;
    gap: 8px;
    color: var(--text-2);
  }
  .boot pre {
    color: var(--danger);
    max-width: 700px;
    white-space: pre-wrap;
  }
</style>
