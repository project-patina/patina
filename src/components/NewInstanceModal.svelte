<script lang="ts">
  import Modal from "./Modal.svelte";
  import LoaderPicker from "./LoaderPicker.svelte";
  import { api } from "../lib/api";
  import { loaderLabel } from "../lib/format";
  import { app, flushSettings, loadVersions, navigate, refreshInstances, toast, toastError } from "../lib/state.svelte";
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
      app.settings!.lastInstance = inst.id;
      app.newInstanceOpen = false;
      toast(`Utworzono „${inst.name}”`, "success");
      navigate("instance", inst.id, "mods");
    } catch (e) {
      toastError(e);
    } finally {
      busy = false;
    }
  }
</script>

<Modal onclose={() => (app.newInstanceOpen = false)} width={420}>
  <h2>Nowa instancja</h2>

  <input class="input" placeholder={autoName || "Nazwa"} bind:value={name} maxlength="48" aria-label="Nazwa" />

  <LoaderPicker bind:mc bind:loader minimal bind:available />

  <select class="input" bind:value={mc} disabled={loader === "rust"} aria-label="Wersja gry">
    {#if !releases.some((v) => v.id === mc) && mc}<option value={mc}>{mc}</option>{/if}
    {#each releases as v (v.id)}<option value={v.id}>Minecraft {v.id}</option>{/each}
  </select>

  <div class="actions">
    <button class="btn ghost" onclick={() => (app.newInstanceOpen = false)}>Anuluj</button>
    <button class="btn primary" disabled={busy || !mc || !available} onclick={create}>Utwórz</button>
  </div>
</Modal>

<style>
  h2 {
    font-size: 1.15rem;
    margin-bottom: 16px;
  }
  .input {
    width: 100%;
    margin-bottom: 10px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 8px;
  }
</style>
