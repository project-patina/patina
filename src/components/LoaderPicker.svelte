<script lang="ts">
  import { api } from "../lib/api";
  import { ENGINE_MC, LOADERS } from "../lib/presets";
  import type { Loader, LoaderVersion } from "../lib/types";

  // Loader cards + loader version select for a given Minecraft version.
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
  const unsupported = $derived(!plain && !loading && !failed && versions.length === 0);
  $effect(() => {
    available = !unsupported && !loading;
  });
</script>

{#if minimal}
  <div class="seg">
    {#each LOADERS as l (l.id)}
      <button class:on={loader === l.id} title={l.hint} onclick={() => (loader = l.id)}>{l.label}</button>
    {/each}
  </div>
  {#if unsupported}<p class="warn small">{loader === "fabric" ? "Fabric" : "Quilt"} nie obsługuje jeszcze {mc}</p>{/if}
{:else}
  <div class="loaders">
    {#each LOADERS as l (l.id)}
      <button class="ld" class:on={loader === l.id} onclick={() => (loader = l.id)}>
        <b>{l.label}</b>
        <small>{l.hint}</small>
      </button>
    {/each}
  </div>
{/if}
{#if minimal}
  <!-- The rest is decided at launch. -->
{:else if loader === "rust"}
  <p class="note">Bez Javy i bez modów Fabric. Przy pierwszym uruchomieniu launcher przygotuje zasoby z Twojej kopii gry. Działają paczki zasobów.</p>
{:else if !plain}
  <div class="lv">
    <span class="dim">Wersja loadera</span>
    {#if loading}
      <span class="dim">ładowanie…</span>
    {:else if unsupported}
      <span class="warn">{loader === "fabric" ? "Fabric" : "Quilt"} nie obsługuje jeszcze {mc}</span>
    {:else if failed}
      <span class="warn">Nie udało się pobrać listy (zostanie wybrana automatycznie)</span>
    {:else}
      <select class="input sel" value={loaderVersion ?? ""} onchange={(e) => (loaderVersion = e.currentTarget.value || null)}>
        {#if !auto}<option value="">Automatycznie (najnowsza)</option>{/if}
        {#each versions as v (v.version)}
          <option value={v.version}>{v.version}{v.stable ? "" : " (beta)"}</option>
        {/each}
      </select>
    {/if}
  </div>
{/if}

<style>
  .seg {
    display: flex;
    gap: 2px;
    padding: 3px;
    margin-bottom: 10px;
    border-radius: var(--radius);
    background: var(--panel-2);
  }
  .seg button {
    flex: 1;
    padding: 7px 4px;
    border-radius: calc(var(--radius) - 3px);
    font-size: 0.88rem;
    font-weight: 560;
    color: var(--text-3);
    transition: background 0.16s, color 0.16s;
  }
  .seg button.on {
    background: var(--panel-3);
    color: var(--text);
  }
  .small {
    margin: -4px 0 10px;
    font-size: 0.84rem;
  }
  .loaders {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 8px;
  }
  .ld {
    padding: 12px;
    border-radius: var(--radius);
    text-align: left;
    background: var(--panel-2);
    border: 1px solid var(--border);
    transition: border-color 0.18s, background 0.18s, transform 0.2s var(--ease);
  }
  .ld:hover {
    border-color: var(--border-2);
    transform: translateY(-1px);
  }
  .ld.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .ld b {
    display: block;
    font-weight: 650;
  }
  .ld small {
    color: var(--text-3);
    font-size: 0.78rem;
    line-height: 1.35;
    display: block;
    margin-top: 3px;
  }
  .lv {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 10px;
    font-size: 0.9rem;
    min-height: 34px;
  }
  .sel {
    width: 200px;
    height: 34px;
  }
  .note {
    margin-top: 10px;
    color: var(--text-3);
    font-size: 0.85rem;
    line-height: 1.4;
  }
  .warn {
    color: var(--warning);
  }
</style>
