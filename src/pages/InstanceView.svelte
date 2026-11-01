<script lang="ts">
  import { fade, fly } from "svelte/transition";
  import { ArrowLeft, FolderOpen, Trash2 } from "@lucide/svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import Console from "../components/Console.svelte";
  import ContentPanel from "../components/ContentPanel.svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import LoaderPicker from "../components/LoaderPicker.svelte";
  import AppearancePicker from "../components/AppearancePicker.svelte";
  import VersionPicker from "../components/VersionPicker.svelte";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { bannerFor, iconFor } from "../lib/presets";
  import { loaderLabel, playTime, relativeTime } from "../lib/format";
  import { app, dur, navigate, refreshInstances, toast, toastError } from "../lib/state.svelte";
  import type { Instance } from "../lib/types";

  const inst = $derived(app.instances.find((i) => i.id === app.instanceId) ?? null);
  const Icon = $derived(iconFor(inst?.icon ?? "box"));

  // ---- settings (edited on a local copy, saved explicitly)
  let draft = $state<Instance | null>(null);
  let loaderOk = $state(true);
  $effect(() => {
    if (inst && (!draft || draft.id !== inst.id)) draft = $state.snapshot(inst) as Instance;
  });
  // Only fields the player edits here; play time etc. change after every session.
  const editable = (i: Instance) => JSON.stringify([i.name, i.icon, i.banner, i.mcVersion, i.loader, i.loaderVersion, i.overrides]);
  const dirty = $derived(!!draft && !!inst && editable(draft) !== editable(inst));
  async function save() {
    if (!draft) return;
    try {
      const saved = await api.updateInstance($state.snapshot(draft) as Instance);
      await refreshInstances();
      draft = $state.snapshot(saved) as Instance;
      toast("Zapisano ustawienia instancji", "success");
    } catch (e) {
      toastError(e);
    }
  }

  let confirmDelete = $state(false);
  async function remove() {
    try {
      await api.deleteInstance(inst!.id);
      confirmDelete = false;
      await refreshInstances();
      navigate("instances");
      toast("Instancja przeniesiona do kosza");
    } catch (e) {
      toastError(e);
    }
  }

  const fullscreenMode = $derived(draft?.overrides.fullscreen == null ? "global" : draft.overrides.fullscreen ? "on" : "off");
  const tabs = [
    ["mods", "Zawartość"],
    ["settings", "Ustawienia"],
    ["console", "Konsola"],
  ] as const;
</script>

{#if inst && draft}
  <div class="view">
    <header class="hero" style:background={bannerFor(inst.banner)} in:fade={{ duration: dur(300) }}>
      <div class="shade"></div>
      <button class="back btn sm" onclick={() => navigate("instances")}><ArrowLeft size={15} /> Instancje</button>
      <div class="hero-row">
        <span class="ico"><Icon size={30} /></span>
        <div class="title">
          <h1>{inst.name}</h1>
          <div class="chips">
            <span class="chip">{loaderLabel(inst.loader)}{inst.loaderVersion ? ` ${inst.loaderVersion}` : ""}</span>
            <span class="chip mono">{inst.mcVersion}</span>
            <span class="chip">{playTime(inst.playTimeSecs)}</span>
            <span class="chip">{relativeTime(inst.lastPlayed)}</span>
          </div>
        </div>
        <div class="hero-actions">
          <button class="btn" onclick={() => api.openFolder(inst.id).catch(toastError)}><FolderOpen size={16} /> Folder</button>
          <PlayButton instance={inst} />
        </div>
      </div>
    </header>

    <div class="tabs">
      {#each tabs as [k, label] (k)}
        <button class="tab" class:on={app.instanceTab === k} onclick={() => (app.instanceTab = k)}>{label}</button>
      {/each}
    </div>

    {#key app.instanceTab}
      <section class="content" in:fly={{ y: 8, duration: dur(260) }}>
        {#if app.instanceTab === "mods"}
          <ContentPanel {inst} />
        {:else if app.instanceTab === "settings"}
          <div class="settings">
            <div class="card block">
              <h3>Ogólne</h3>
              <Row title="Nazwa"><input class="input" style="width: 300px" bind:value={draft.name} maxlength="48" /></Row>
              <div class="pad"><AppearancePicker bind:icon={draft.icon} bind:banner={draft.banner} /></div>
              <div class="pad">
                <div class="label" style="margin-bottom: 8px">Baner — własny obraz lub scena</div>
                <ImagePicker bind:value={draft.banner} allowNone={false} />
              </div>
            </div>

            <div class="card block">
              <h3>Wersja i loader</h3>
              <p class="dim note">Zmiana wersji nie usuwa światów, ale stare mody mogą nie pasować.</p>
              <div class="pad"><VersionPicker bind:value={draft.mcVersion} /></div>
              <div class="pad"><LoaderPicker bind:mc={draft.mcVersion} bind:loader={draft.loader} bind:loaderVersion={draft.loaderVersion} auto={false} bind:available={loaderOk} /></div>
            </div>

            <div class="card block">
              <h3>Java i pamięć</h3>
              <Row title="Własny przydział RAM" desc="Domyślnie używana jest wartość z ustawień globalnych.">
                {#if draft.overrides.memoryMaxMb != null}<MemorySlider bind:value={draft.overrides.memoryMaxMb} />{/if}
                <Toggle checked={draft.overrides.memoryMaxMb != null} onchange={(v) => (draft!.overrides.memoryMaxMb = v ? (app.settings?.memoryMaxMb ?? 4096) : null)} />
              </Row>
              <Row title="Ścieżka Javy" desc="Puste = automatycznie (Java dopasowana do wersji gry).">
                <input class="input mono" style="width: 340px" placeholder="automatycznie" value={draft.overrides.javaPath ?? ""} oninput={(e) => (draft!.overrides.javaPath = e.currentTarget.value || null)} />
              </Row>
              <Row title="Dodatkowe argumenty JVM" stacked>
                <input class="input mono" placeholder="np. -Dsodium.checks.issue2561=false" value={draft.overrides.jvmArgs ?? ""} oninput={(e) => (draft!.overrides.jvmArgs = e.currentTarget.value || null)} />
              </Row>
            </div>

            <div class="card block">
              <h3>Okno gry</h3>
              <Row title="Rozdzielczość" desc="Puste = z ustawień globalnych.">
                <input class="input num" type="number" placeholder={String(app.settings?.gameWidth)} value={draft.overrides.gameWidth ?? ""} oninput={(e) => (draft!.overrides.gameWidth = e.currentTarget.value ? +e.currentTarget.value : null)} />
                <span class="dim">×</span>
                <input class="input num" type="number" placeholder={String(app.settings?.gameHeight)} value={draft.overrides.gameHeight ?? ""} oninput={(e) => (draft!.overrides.gameHeight = e.currentTarget.value ? +e.currentTarget.value : null)} />
              </Row>
              <Row title="Pełny ekran">
                <select class="input" style="width: 180px" value={fullscreenMode} onchange={(e) => { const v = e.currentTarget.value; draft!.overrides.fullscreen = v === "global" ? null : v === "on"; }}>
                  <option value="global">Jak globalnie</option>
                  <option value="on">Tak</option>
                  <option value="off">Nie</option>
                </select>
              </Row>
              <Row title="Dołącz do serwera po starcie" desc="Adres serwera, np. play.example.com lub host:25565.">
                <input class="input mono" style="width: 260px" placeholder="brak" value={draft.overrides.joinServer ?? ""} oninput={(e) => (draft!.overrides.joinServer = e.currentTarget.value || null)} />
              </Row>
            </div>

            <div class="card block">
              <h3>Zaawansowane</h3>
              <Row title="Komenda opakowująca" desc="Np. prime-run, obs-gamecapture. Zastępuje globalną.">
                <input class="input mono" style="width: 260px" placeholder="brak" value={draft.overrides.wrapperCommand ?? ""} oninput={(e) => (draft!.overrides.wrapperCommand = e.currentTarget.value || null)} />
              </Row>
              <Row title="Zmienne środowiskowe" desc="Jedna na linię: KLUCZ=wartość. Dodawane do globalnych." stacked>
                <textarea class="input" rows="3" placeholder="MESA_GL_VERSION_OVERRIDE=4.6" value={draft.overrides.envVars ?? ""} oninput={(e) => (draft!.overrides.envVars = e.currentTarget.value || null)}></textarea>
              </Row>
            </div>

            <div class="card block danger-zone">
              <Row title="Usuń instancję" desc="Folder instancji (światy, mody) trafi do kosza systemowego.">
                <button class="btn danger" onclick={() => (confirmDelete = true)}><Trash2 size={15} /> Usuń</button>
              </Row>
            </div>

            {#if dirty}
              <div class="savebar card glass" transition:fly={{ y: 20, duration: dur(250) }}>
                <span>Masz niezapisane zmiany</span>
                <button class="btn ghost" onclick={() => (draft = $state.snapshot(inst) as Instance)}>Cofnij</button>
                <button class="btn primary" disabled={!loaderOk} onclick={save}>Zapisz</button>
              </div>
            {/if}
          </div>
        {:else}
          <div class="console-wrap"><Console id={inst.id} /></div>
        {/if}
      </section>
    {/key}
  </div>

  {#if confirmDelete}
    <Modal onclose={() => (confirmDelete = false)} width={420}>
      <h3>Usunąć „{inst.name}”?</h3>
      <p class="muted">Folder z modami i światami zostanie przeniesiony do kosza — możesz go jeszcze odzyskać.</p>
      <div class="confirm">
        <button class="btn ghost" onclick={() => (confirmDelete = false)}>Anuluj</button>
        <button class="btn danger" onclick={remove}><Trash2 size={15} /> Usuń</button>
      </div>
    </Modal>
  {/if}
{:else}
  <div class="missing dim">Nie znaleziono instancji. <button class="btn sm" onclick={() => navigate("instances")}>Wróć</button></div>
{/if}

<style>
  .view {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 6px 22px 0 6px;
    min-height: 0;
  }
  .hero {
    position: relative;
    border-radius: var(--radius-l);
    overflow: hidden;
    border: 1px solid var(--border);
    padding: 16px 18px 18px;
    min-height: 150px;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    flex-shrink: 0;
  }
  .shade {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, rgb(0 0 0 / 0.55), rgb(0 0 0 / 0.05));
  }
  .back {
    position: relative;
    align-self: flex-start;
    color: white;
    background: rgb(0 0 0 / 0.3);
    border-color: rgb(255 255 255 / 0.12);
  }
  .hero-row {
    position: relative;
    display: flex;
    align-items: flex-end;
    gap: 16px;
    margin-top: 18px;
  }
  .ico {
    width: 64px;
    height: 64px;
    border-radius: 18px;
    display: grid;
    place-items: center;
    color: white;
    background: rgb(255 255 255 / 0.14);
    border: 1px solid rgb(255 255 255 / 0.18);
    flex-shrink: 0;
  }
  .title {
    flex: 1;
    min-width: 0;
    color: white;
  }
  .title h1 {
    font-size: 1.8rem;
    font-weight: 750;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .chips {
    display: flex;
    gap: 6px;
    margin-top: 8px;
    flex-wrap: wrap;
  }
  .chips .chip {
    background: rgb(0 0 0 / 0.3);
    border-color: rgb(255 255 255 / 0.12);
    color: rgb(255 255 255 / 0.85);
  }
  .hero-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .hero-actions .btn {
    height: 58px;
    border-radius: 14px;
    color: white;
    background: rgb(0 0 0 / 0.3);
    border-color: rgb(255 255 255 / 0.12);
  }
  .tabs {
    display: flex;
    gap: 2px;
    margin: 14px 0 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .tab {
    padding: 9px 14px;
    font-weight: 600;
    color: var(--text-3);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color 0.2s, border-color 0.25s var(--ease);
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.on {
    color: var(--text);
    border-color: var(--accent);
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-bottom: 22px;
  }
  .settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 860px;
    padding-bottom: 70px;
  }
  .block {
    padding: 16px 20px 6px;
  }
  .block h3 {
    font-size: 1.05rem;
    margin-bottom: 4px;
  }
  .pad {
    padding: 10px 0 14px;
  }
  .note {
    margin: 2px 0 0;
    font-size: 0.85rem;
  }
  .num {
    width: 90px;
  }
  .danger-zone {
    padding: 6px 20px;
    border-color: color-mix(in srgb, var(--danger) 30%, var(--border));
  }
  .savebar {
    position: sticky;
    bottom: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 10px 10px 18px;
    border-radius: var(--radius);
    box-shadow: var(--shadow);
  }
  .savebar span {
    flex: 1;
    font-weight: 600;
  }
  .console-wrap {
    height: 100%;
    min-height: 300px;
  }
  .confirm {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 18px;
  }
  .missing {
    padding: 40px;
    display: flex;
    gap: 12px;
    align-items: center;
  }
</style>
