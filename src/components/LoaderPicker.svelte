<script lang="ts">
  import Segmented from "./Segmented.svelte";
  import { api } from "../lib/api";
  import { loaderLabel } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { ENGINE_MC, LOADERS } from "../lib/presets";
  import type { Loader, LoaderVersion } from "../lib/types";

  // Loader choice + loader version select for a given Minecraft version.
  // `auto = false` keeps a null loader version as "pick newest at launch".
  let {
    mc = $bindable(""),
    loader = $bindable("fabric"),
    loaderVersion = $bindable(null),
    auto = true,
    available = $bindable(true),
    minimal = false,
  }: { mc: string; loader?: Loader; loaderVersion?: string | null; auto?: boolean; available?: boolean; minimal?: boolean } = $props();

  let versions = $state<LoaderVersion[]>([]);
  let loading = $state(false);
  let failed = $state(false);

  $effect(() => {
    const l = loader;
    const v = mc;
    if (l === "rust" && v !== ENGINE_MC) mc = ENGINE_MC;
    if (l === "vanilla" || l === "rust" || !v) {
      versions = [];
      loaderVersion = null;
      // A request cut short here never reaches the `finally` below.
      loading = false;
      failed = false;
      return;
    }
    loading = true;
    failed = false;
    let stale = false;
    api
      .loaderVersions(l, v)
      .then((list) => {
        if (stale) return;
        versions = list;
        if ((auto && !loaderVersion) || (loaderVersion && !list.some((x) => x.version === loaderVersion))) {
          loaderVersion = (list.find((x) => x.stable) ?? list[0])?.version ?? null;
        }
      })
      .catch(() => !stale && (failed = true))
      .finally(() => !stale && (loading = false));
    return () => (stale = true);
  });

  const plain = $derived(loader === "vanilla" || loader === "rust");
  const unsupported = $derived(!plain && !!mc && !loading && !failed && versions.length === 0);
  // A Quilt or Vanilla instance keeps its loader until the player picks another one.
  const offered = $derived(LOADERS.find((l) => l.id === loader));
  const hint = $derived(offered ? offered.hint : t("loaderPicker.legacy", { loader: loaderLabel(loader) }));
  const options = $derived(LOADERS.map((l) => ({ value: l.id, label: l.label })));
  $effect(() => {
    available = !unsupported && !loading;
  });
</script>

<div class="choice">
  <Segmented label={t("loaderPicker.title")} value={loader} {options} onchange={(v) => (loader = v)} />
  <p class="hint">{hint}</p>
  {#if unsupported}<p class="warn">{t("loaderPicker.unsupported", { loader: loaderLabel(loader), mc })}</p>{/if}
</div>
<!-- With `minimal` the rest is decided at launch. -->
{#if !minimal && loader === "rust"}
  <p class="hint">{t("loaderPicker.rustNote")}</p>
{:else if !minimal && !plain && !unsupported}
  <div class="lv">
    <span class="label">{t("loaderPicker.version")}</span>
    {#if loading}
      <span class="hint">{t("common.loading")}</span>
    {:else if failed}
      <span class="warn">{t("loaderPicker.failed")}</span>
    {:else}
      <select class="input sel" aria-label={t("loaderPicker.version")} value={loaderVersion ?? ""} onchange={(e) => (loaderVersion = e.currentTarget.value || null)}>
        {#if !auto}<option value="">{t("loaderPicker.newest")}</option>{/if}
        {#each versions as v (v.version)}
          <option value={v.version}>{v.version}{v.stable ? "" : " (beta)"}</option>
        {/each}
      </select>
    {/if}
  </div>
{/if}

<style>
  .choice {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  .hint, .warn {
    margin: 0;
    font-size: var(--fs-12);
    line-height: 1.4;
  }
  .hint { color: var(--text-3); }
  .warn { color: var(--warning); }
  .choice + .hint { margin-top: 4px; }
  .lv {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 32px;
    margin-top: 8px;
  }
  .sel { width: 200px; }
</style>
