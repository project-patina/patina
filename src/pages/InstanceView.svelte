<script lang="ts">
  import { ArrowLeft, FolderOpen, Trash2 } from "@lucide/svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import Console from "../components/Console.svelte";
  import ContentPanel from "../components/ContentPanel.svelte";
  import OfficialMods from "../components/OfficialMods.svelte";
  import Group from "../components/Group.svelte";
  import ImagePicker from "../components/ImagePicker.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Select from "../components/Select.svelte";
  import Toggle from "../components/Toggle.svelte";
  import MemorySlider from "../components/MemorySlider.svelte";
  import LoaderPicker from "../components/LoaderPicker.svelte";
  import AppearancePicker from "../components/AppearancePicker.svelte";
  import VersionPicker from "../components/VersionPicker.svelte";
  import Modal from "../components/Modal.svelte";
  import { api } from "../lib/api";
  import { loaderLabel, playTime, relativeTime } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { dialog, page } from "../lib/motion";
  import { parseWhole } from "../lib/numbers";
  import { bannerFor, iconFor } from "../lib/presets";
  import { app, navigate, refreshInstances, toast, toastError } from "../lib/state.svelte";
  import type { Instance } from "../lib/types";

  const inst = $derived(app.instances.find((i) => i.id === app.instanceId) ?? null);
  const Icon = $derived(iconFor(inst?.icon ?? "box"));
  const meta = $derived(inst ? [
    `${loaderLabel(inst.loader)}${inst.loaderVersion ? ` ${inst.loaderVersion}` : ""} · ${inst.mcVersion}`,
    inst.playTimeSecs > 0 ? playTime(inst.playTimeSecs) : "",
    inst.lastPlayed ? relativeTime(inst.lastPlayed) : "",
  ].filter(Boolean).join(" · ") : "");

  // ---- settings (edited on a local copy, saved explicitly)
  let draft = $state<Instance | null>(null);
  let loaderOk = $state(true);
  $effect(() => {
    if (inst && (!draft || draft.id !== inst.id)) draft = $state.snapshot(inst) as Instance;
  });
  // Only fields the player edits here; play time etc. change after every session.
  // A cleared override is null while an untouched one is absent: both mean "use the global value".
  const editable = (i: Instance) => JSON.stringify([
    i.name, i.icon, i.banner, i.mcVersion, i.loader, i.loaderVersion,
    Object.entries(i.overrides).filter(([, value]) => value != null).sort(),
  ]);
  const dirty = $derived(!!draft && !!inst && editable(draft) !== editable(inst));
  async function save() {
    if (!draft) return;
    try {
      const saved = await api.updateInstance($state.snapshot(draft) as Instance);
      await refreshInstances();
      draft = $state.snapshot(saved) as Instance;
      toast(t("instance.saved"), "success");
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
      toast(t("instance.deleted"));
    } catch (e) {
      toastError(e);
    }
  }

  const fullscreenMode = $derived(draft?.overrides.fullscreen == null ? "global" : draft.overrides.fullscreen ? "on" : "off");
  const tabs = $derived([
    { value: "mods" as const, label: t("instance.tabContent") },
    ...(inst?.loader === "rust" ? [{ value: "official" as const, label: t("instance.tabOfficial") }] : []),
    { value: "settings" as const, label: t("nav.settings") },
    { value: "console" as const, label: t("instance.tabConsole") },
  ]);

  // The tab can outlive the Rust loader that offered it.
  const tab = $derived(app.instanceTab === "official" && inst?.loader !== "rust" ? "mods" : app.instanceTab);

  // Blank means the global value; the rest is held to the same range as the global setting.
  function setSize(key: "gameWidth" | "gameHeight", min: number, max: number) {
    return (event: Event & { currentTarget: HTMLInputElement }) => {
      const text = event.currentTarget.value;
      draft!.overrides[key] = text.trim() ? parseWhole(text, min, max, app.settings?.[key] ?? min) : null;
      event.currentTarget.value = String(draft!.overrides[key] ?? "");
    };
  }
</script>

{#if inst && draft}
  <div class="view">
    <header class="hero" style:background={bannerFor(inst.banner)}>
      <div class="scrim"></div>
      <button class="tool back" onclick={() => navigate("instances")}><ArrowLeft size={14} /> {t("nav.instances")}</button>
      <div class="hero-row">
        <span class="ico"><Icon size={20} /></span>
        <div class="title">
          <h1>{inst.name}</h1>
          <p>{meta}</p>
        </div>
        <button class="tool folder" title={t("instance.folder")} aria-label={t("instance.folder")} onclick={() => api.openFolder(inst.id).catch(toastError)}><FolderOpen size={16} /></button>
        <PlayButton instance={inst} />
      </div>
    </header>

    <div class="tabs">
      <Segmented label={t("instance.sections")} value={tab} options={tabs} onchange={(v) => (app.instanceTab = v)} />
    </div>

    {#key tab}
      <section class="content" in:page>
        {#if tab === "mods"}
          <ContentPanel {inst} />
        {:else if tab === "official"}
          <OfficialMods {inst} />
        {:else if tab === "settings"}
          <div class="column">
            <Group title={t("instance.general")}>
              <Row title={t("instance.name")}><input class="input wide" aria-label={t("instance.name")} bind:value={draft.name} maxlength="48" /></Row>
              <Row title={t("appearance.icon")} stacked><AppearancePicker bind:icon={draft.icon} /></Row>
              <Row title={t("appearance.banner")} stacked>
                <ImagePicker bind:value={draft.banner} allowNone={false} />
              </Row>
            </Group>

            <Group title={t("instance.versionLoader")}>
              <p class="note">{t("instance.versionNote")}</p>
              <div class="pad"><VersionPicker bind:value={draft.mcVersion} /></div>
              <div class="pad"><LoaderPicker bind:mc={draft.mcVersion} bind:loader={draft.loader} bind:loaderVersion={draft.loaderVersion} auto={false} bind:available={loaderOk} /></div>
            </Group>

            <Group title={t("settings.java")}>
              <Row title={t("instance.memory")} desc={t("instance.memoryDesc")}>
                {#if draft.overrides.memoryMaxMb != null}<MemorySlider bind:value={draft.overrides.memoryMaxMb} />{/if}
                <Toggle label={t("instance.memory")} checked={draft.overrides.memoryMaxMb != null} onchange={(v) => (draft!.overrides.memoryMaxMb = v ? (app.settings?.memoryMaxMb ?? 4096) : null)} />
              </Row>
              <Row title={t("java.path")} desc={t("instance.javaDesc")}>
                <input class="input mono wide" aria-label={t("java.path")} placeholder={t("java.auto")} value={draft.overrides.javaPath ?? ""} oninput={(e) => (draft!.overrides.javaPath = e.currentTarget.value || null)} />
              </Row>
              <Row title={t("instance.jvmArgs")} stacked>
                <input class="input mono" aria-label={t("instance.jvmArgs")} placeholder="-Dsodium.checks.issue2561=false" value={draft.overrides.jvmArgs ?? ""} oninput={(e) => (draft!.overrides.jvmArgs = e.currentTarget.value || null)} />
              </Row>
            </Group>

            <Group title={t("settings.game")}>
              <Row title={t("game.resolution")} desc={t("instance.resolutionDesc")}>
                <input class="input num" type="number" min="320" max="15360" step="1" aria-label={t("game.width")} placeholder={String(app.settings?.gameWidth)} value={draft.overrides.gameWidth ?? ""} onchange={setSize("gameWidth", 320, 15360)} />
                <span class="dim">×</span>
                <input class="input num" type="number" min="240" max="8640" step="1" aria-label={t("game.height")} placeholder={String(app.settings?.gameHeight)} value={draft.overrides.gameHeight ?? ""} onchange={setSize("gameHeight", 240, 8640)} />
              </Row>
              <Row title={t("game.fullscreen")}>
                <Select label={t("game.fullscreen")} value={fullscreenMode} options={[{ value: "global", label: t("instance.likeGlobal") }, { value: "on", label: t("common.on") }, { value: "off", label: t("common.off") }]}
                  onchange={(v) => (draft!.overrides.fullscreen = v === "global" ? null : v === "on")} />
              </Row>
              <Row title={t("instance.joinServer")} desc={t("instance.joinServerDesc")}>
                <input class="input mono wide" aria-label={t("instance.joinServer")} placeholder={t("common.none")} value={draft.overrides.joinServer ?? ""} oninput={(e) => (draft!.overrides.joinServer = e.currentTarget.value || null)} />
              </Row>
            </Group>

            <Group title={t("settings.advanced")}>
              <Row title={t("advanced.wrapper")} desc={t("instance.wrapperDesc")}>
                <input class="input mono wide" aria-label={t("advanced.wrapper")} placeholder={t("common.none")} value={draft.overrides.wrapperCommand ?? ""} oninput={(e) => (draft!.overrides.wrapperCommand = e.currentTarget.value || null)} />
              </Row>
              <Row title={t("advanced.env")} desc={t("instance.envDesc")} stacked>
                <textarea class="input" aria-label={t("advanced.env")} rows="3" placeholder="MESA_GL_VERSION_OVERRIDE=4.6" value={draft.overrides.envVars ?? ""} oninput={(e) => (draft!.overrides.envVars = e.currentTarget.value || null)}></textarea>
              </Row>
              <Row title={t("instance.delete")} desc={t("instance.deleteDesc")}>
                <button class="btn danger" onclick={() => (confirmDelete = true)}><Trash2 size={14} /> {t("common.delete")}</button>
              </Row>
            </Group>

            {#if dirty}
              <div class="savebar" in:dialog out:dialog={{ duration: 100 }}>
                <span>{t("instance.unsaved")}</span>
                <button class="btn ghost" onclick={() => (draft = $state.snapshot(inst) as Instance)}>{t("common.revert")}</button>
                <button class="btn primary" disabled={!loaderOk || !draft.name.trim()} onclick={save}>{t("common.save")}</button>
              </div>
            {/if}
          </div>
        {:else}
          <Console id={inst.id} />
        {/if}
      </section>
    {/key}
  </div>

  {#if confirmDelete}
    <Modal onclose={() => (confirmDelete = false)} width={420}>
      <h3 class="modal-title">{t("instance.deleteTitle", { name: inst.name })}</h3>
      <p class="muted">{t("instance.deleteBody")}</p>
      <div class="confirm">
        <button class="btn ghost" onclick={() => (confirmDelete = false)}>{t("common.cancel")}</button>
        <button class="btn danger" onclick={remove}><Trash2 size={14} /> {t("common.delete")}</button>
      </div>
    </Modal>
  {/if}
{:else}
  <div class="missing dim">{t("instance.missing")} <button class="btn sm" onclick={() => navigate("instances")}>{t("common.back")}</button></div>
{/if}

<style>
  .view {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    min-height: 0;
  }
  .hero {
    position: relative;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 12px;
    height: 128px;
    padding: 12px;
    border-radius: var(--radius-l);
    overflow: hidden;
    color: #eef0ef;
  }
  .scrim {
    position: absolute;
    inset: 0;
    background: rgb(12 14 15 / 0.48);
  }
  .tool {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    border-radius: var(--radius-s);
    color: #eef0ef;
    background: rgb(12 14 15 / 0.48);
    font-size: var(--fs-12);
    font-weight: 500;
    transition: background 0.12s;
  }
  .tool:hover { background: rgb(12 14 15 / 0.72); }
  .back { align-self: flex-start; height: 28px; padding: 0 8px; }
  .folder { flex-shrink: 0; width: 44px; height: 44px; border-radius: var(--radius); }
  .hero-row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .ico {
    width: 44px;
    height: 44px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: var(--radius);
    background: rgb(12 14 15 / 0.48);
  }
  .title { flex: 1; min-width: 0; }
  h1 {
    font-family: var(--font-display);
    font-size: var(--fs-20);
    font-weight: 600;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .title p {
    margin: 2px 0 0;
    color: #c6cbc9;
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .tabs { flex-shrink: 0; }
  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  .column { max-width: 640px; margin: 0 auto; }
  .pad { padding: 12px 0; border-bottom: 1px solid var(--border); }
  .pad:last-child { border-bottom: 0; }
  .note { margin: 8px 0 0; color: var(--text-3); font-size: var(--fs-12); }
  .wide { width: 240px; }
  .num { width: 88px; }
  .savebar {
    position: sticky;
    bottom: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 16px;
    padding: 8px 8px 8px 12px;
    border: 1px solid var(--border-2);
    border-radius: var(--radius);
    background: var(--panel);
    box-shadow: var(--shadow);
    font-size: var(--fs-13);
  }
  .savebar span { flex: 1; min-width: 0; }
  .modal-title { font-size: var(--fs-16); margin-bottom: 8px; }
  .muted { margin: 0; font-size: var(--fs-13); line-height: 1.5; }
  .confirm {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .missing {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 32px;
    font-size: var(--fs-13);
  }
  @media (max-height: 640px) {
    .view { padding: 12px; gap: 8px; }
    .hero { height: 112px; padding: 8px 12px; gap: 8px; }
  }
</style>
