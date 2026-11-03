<script lang="ts">
  import { LoaderCircle } from "@lucide/svelte";
  import Option from "./Option.svelte";
  import SearchInput from "./SearchInput.svelte";
  import { locale, t } from "../lib/i18n.svelte";
  import type { Key } from "../lib/locales/pl";
  import { app, loadVersions, toastError } from "../lib/state.svelte";

  let { value = $bindable("") }: { value?: string } = $props();

  let query = $state("");
  let snapshots = $state(app.settings?.showSnapshots ?? false);
  let old = $state(app.settings?.showOldVersions ?? false);
  let loading = $state(true);

  $effect(() => {
    loadVersions()
      .then((v) => {
        if (!value) value = v.latestRelease;
      })
      .catch(toastError)
      .finally(() => (loading = false));
  });

  const list = $derived(
    (app.versions?.versions ?? []).filter(
      (v) =>
        (v.id === value || v.kind === "release" || (snapshots && v.kind === "snapshot") || (old && v.kind.startsWith("old"))) &&
        (!query || v.id.toLowerCase().includes(query.toLowerCase())),
    ),
  );

  const TAGS: Record<string, Key> = { snapshot: "version.snapshot", old_beta: "version.beta", old_alpha: "version.alpha" };
</script>

<div class="picker">
  <div class="top">
    <div class="grow"><SearchInput bind:value={query} placeholder={t("version.search")} width="100%" /></div>
    <Option bind:checked={snapshots} label={t("version.snapshots")} />
    <Option bind:checked={old} label={t("version.old")} />
  </div>
  <div class="list">
    {#if loading}
      <div class="center"><LoaderCircle size={16} class="spin" /></div>
    {:else}
      {#each list as v (v.id)}
        <button class="ver" aria-pressed={v.id === value} onclick={() => (value = v.id)}>
          <span class="mono">{v.id}</span>
          {#if v.id === app.versions?.latestRelease}<span class="chip">{t("version.latest")}</span>{/if}
          {#if TAGS[v.kind]}<span class="chip">{t(TAGS[v.kind])}</span>{/if}
          <span class="date">{new Date(v.releaseTime).toLocaleDateString(locale())}</span>
        </button>
      {:else}
        <div class="center dim">{t("common.noResults")}</div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .top {
    display: flex;
    align-items: center;
    gap: 8px 12px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .grow { flex: 1; min-width: 160px; }
  .list {
    height: 196px;
    overflow: auto;
    padding: 4px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
  }
  .ver {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 8px;
    border-radius: var(--radius-s);
    font-size: var(--fs-13);
    text-align: left;
    transition: background 0.12s;
  }
  .ver:hover { background: var(--panel-2); }
  .ver[aria-pressed="true"] { background: var(--accent-soft); }
  .date {
    margin-left: auto;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .center {
    display: grid;
    place-items: center;
    height: 100%;
    font-size: var(--fs-13);
  }
</style>
