<script lang="ts">
  import Modal from "./Modal.svelte";
  import LoaderPicker from "./LoaderPicker.svelte";
  import { api } from "../lib/api";
  import { loaderLabel } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { app, flushSettings, loadVersions, navigate, refreshInstances, toast, toastError, updateSettings } from "../lib/state.svelte";
  import type { Loader } from "../lib/types";

  let mc = $state("");
  let loader = $state<Loader>(app.settings?.defaultLoader ?? "fabric");
  let name = $state("");
  let busy = $state(false);
  let available = $state(true);

  const releases = $derived((app.versions?.versions ?? []).filter((v) => v.kind === "release"));
  const autoName = $derived(mc ? `${loaderLabel(loader)} ${mc}` : "");

  $effect(() => {
    loadVersions()
      .then((v) => {
        if (!mc) mc = v.latestRelease;
      })
      .catch(toastError);
  });

  async function create() {
    busy = true;
    try {
      await flushSettings();
      const inst = await api.createInstance({ name: name.trim() || autoName, mcVersion: mc, loader, loaderVersion: null, icon: "grass", banner: "aurora" });
      await refreshInstances();
      updateSettings({ lastInstance: inst.id });
      app.newInstanceOpen = false;
      toast(t("newInstance.created", { name: inst.name }), "success");
      navigate("instance", inst.id, "mods");
    } catch (e) {
      toastError(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal onclose={() => (app.newInstanceOpen = false)} width={420}>
  <h3 class="title">{t("home.newInstance")}</h3>

  <div class="field">
    <span class="label">{t("instance.name")}</span>
    <input class="input" placeholder={autoName} bind:value={name} maxlength="48" aria-label={t("instance.name")} />
  </div>

  <div class="field">
    <span class="label">{t("loaderPicker.title")}</span>
    <LoaderPicker bind:mc bind:loader minimal bind:available />
  </div>

  <div class="field">
    <span class="label">{t("newInstance.version")}</span>
    <select class="input" bind:value={mc} disabled={loader === "rust"} aria-label={t("newInstance.version")}>
      {#if !releases.some((v) => v.id === mc) && mc}<option value={mc}>{mc}</option>{/if}
      {#each releases as v (v.id)}<option value={v.id}>{v.id}</option>{/each}
    </select>
  </div>

  <div class="actions">
    <button class="btn ghost" onclick={() => (app.newInstanceOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" disabled={busy || !mc || !available} onclick={create}>{t("common.create")}</button>
  </div>
</Modal>

<style>
  .title { font-size: var(--fs-16); margin-bottom: 16px; }
  .field {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-bottom: 12px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
