<script lang="ts">
  import { onDestroy } from "svelte";
  import { Copy, Handshake, LoaderCircle, Play, RefreshCw, Server, Signal, Users } from "@lucide/svelte";
  import SearchInput from "../components/SearchInput.svelte";
  import Segmented from "../components/Segmented.svelte";
  import Select from "../components/Select.svelte";
  import { api } from "../lib/api";
  import { compact, loaderLabel } from "../lib/format";
  import { t } from "../lib/i18n.svelte";
  import { reveal } from "../lib/motion";
  import { app, play, toast, toastError } from "../lib/state.svelte";
  import type { ServerEntry, ServerRegion, ServerStatus } from "../lib/types";

  type RegionChoice = ServerRegion | "all";
  const REGION_KEY = "patina.servers.region";

  /** The player's region from the system's time zone. */
  function detectRegion(): ServerRegion {
    const zone = Intl.DateTimeFormat().resolvedOptions().timeZone ?? "";
    const south = /^America\/(Argentina|Sao_Paulo|Santiago|Lima|Bogota|Caracas|Montevideo|Asuncion|La_Paz|Guayaquil|Recife|Fortaleza|Belem|Manaus|Bahia|Cuiaba|Campo_Grande|Porto_Velho|Boa_Vista|Rio_Branco|Maceio|Araguaina|Santarem|Noronha|Punta_Arenas|Cayenne|Paramaribo|Guyana)/;
    if (zone.startsWith("America/")) return south.test(zone) ? "sa" : "na";
    if (zone.startsWith("Asia/")) return "as";
    if (zone.startsWith("Australia/") || zone.startsWith("Pacific/")) return "oc";
    return "eu";
  }
  const detected = detectRegion();
  function savedRegion(): RegionChoice {
    try {
      const saved = localStorage.getItem(REGION_KEY);
      if (saved && ["all", "eu", "na", "sa", "as", "oc"].includes(saved)) return saved as RegionChoice;
    } catch {}
    return detected;
  }

  let servers = $state<ServerEntry[]>([]);
  let statuses = $state<Record<string, ServerStatus | undefined>>({});
  let query = $state("");
  let region = $state<RegionChoice>(savedRegion());
  let target = $state<string>(app.settings?.lastInstance && app.instances.some((i) => i.id === app.settings!.lastInstance) ? app.settings.lastInstance : (app.instances[0]?.id ?? ""));

  const regionName = (r: ServerRegion) => t(`servers.region.${r}` as Parameters<typeof t>[0]);
  const regions = $derived<{ value: RegionChoice; label: string }[]>([
    { value: "all", label: t("servers.regionAll") },
    ...(["eu", "na", "sa", "as", "oc"] as const).map((r) => ({ value: r, label: regionName(r) })),
  ]);
  const targets = $derived(app.instances.map((i) => ({ value: i.id, label: `${i.name} · ${loaderLabel(i.loader)} ${i.mcVersion}` })));

  function setRegion(r: RegionChoice) {
    region = r;
    try { localStorage.setItem(REGION_KEY, r); } catch {}
  }

  let alive = true;
  onDestroy(() => (alive = false));
  async function ping(address: string) {
    statuses[address] = undefined;
    try {
      const s = await api.serverPing(address);
      if (alive) statuses[address] = s;
    } catch {
      if (alive) statuses[address] = { online: false, players: null, maxPlayers: null, version: null, motd: "", latencyMs: null, favicon: null };
    }
  }
  function refresh() {
    for (const s of servers) void ping(s.address);
  }
  api.serversList().then((list) => { servers = list; refresh(); }).catch(toastError);

  const needle = $derived(query.trim().toLowerCase());
  const tagName = (tag: string) => t(`servers.tag.${tag}` as Parameters<typeof t>[0]);
  const shown = $derived(
    servers
      .filter((s) => region === "all" || s.region === region)
      .filter((s) => !needle || s.name.toLowerCase().includes(needle) || s.address.includes(needle) || s.tags.some((tag) => tag.includes(needle) || tagName(tag).toLowerCase().includes(needle)))
      // The player's region first, then the busiest.
      .sort((a, b) => Number(b.region === detected) - Number(a.region === detected) || (statuses[b.address]?.players ?? -1) - (statuses[a.address]?.players ?? -1)),
  );
  /** A typed address that is not on the list gets a row of its own. */
  const typed = $derived(/^[a-z0-9.-]+\.[a-z]{2,}(:\d{1,5})?$|^\[?[0-9a-f:.]+\]?(:\d{1,5})?$/i.test(needle) && needle.includes(".") && !servers.some((s) => s.address === needle) ? needle : "");
  let typedTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const address = typed;
    clearTimeout(typedTimer);
    if (address && !(address in statuses)) typedTimer = setTimeout(() => void ping(address), 400);
  });
  onDestroy(() => clearTimeout(typedTimer));

  function copy(address: string) {
    navigator.clipboard?.writeText(address).then(() => toast(t("servers.copied", { address }), "success")).catch(toastError);
  }
  function join(address: string) {
    if (!target) return toast(t("servers.noInstance"), "error");
    void play(target, address);
  }
  const busy = $derived(!!target && !!app.running[target]);
</script>

{#snippet row(name: string, address: string, entryRegion: ServerRegion | null, tags: string[], n: number)}
  {@const s = statuses[address]}
  <div class="srv" in:reveal|global={{ index: n }}>
    {#if s?.favicon}<img class="icon" src={s.favicon} alt="" />{:else}<span class="icon ph"><Server size={18} /></span>{/if}
    <div class="txt">
      <span class="name">
        <b>{name}</b>
        <small class="mono">{address}</small>
        {#if entryRegion}<span class="chip">{regionName(entryRegion)}</span>{/if}
      </span>
      <span class="motd">{#if s === undefined}{t("servers.checking")}{:else if !s.online}{t("servers.offline")}{:else}{s.motd || s.version || ""}{/if}</span>
      <span class="meta">
        {#if s?.online}
          <span><Users size={12} /> {(s.maxPlayers ?? 0) > 0 ? t("servers.players", { online: compact(s.players ?? 0), max: compact(s.maxPlayers ?? 0) }) : t("servers.playersOnline", { online: compact(s.players ?? 0) })}</span>
          {#if s.latencyMs !== null}<span><Signal size={12} /> {s.latencyMs} ms</span>{/if}
          {#if s.version}<span class="ver">{s.version}</span>{/if}
        {:else if s === undefined}
          <span><LoaderCircle size={12} class="spin" /></span>
        {:else}
          <span class="off">● {t("servers.offline")}</span>
        {/if}
        {#if tags.length}<span class="tags">{tags.map(tagName).join(" · ")}</span>{/if}
      </span>
    </div>
    <div class="actions">
      <button class="btn ghost" title={t("servers.copy")} aria-label={t("servers.copy")} onclick={() => copy(address)}><Copy size={14} /></button>
      <button class="btn primary" disabled={!target || busy} onclick={() => join(address)}><Play size={14} /> {t("servers.play")}</button>
    </div>
  </div>
{/snippet}

<div class="view">
  <header class="head">
    <h1 class="page-title">{t("nav.servers")}</h1>
    <div class="target">
      <span class="dim">{t("servers.playOn")}</span>
      <Select label={t("servers.playOn")} width={260} value={target} options={targets} onchange={(id) => (target = id)} />
    </div>
  </header>

  <section class="partners" aria-labelledby="partners-title">
    <h2 id="partners-title" class="section-title"><Handshake size={15} /> {t("servers.partners")} <span class="soon">{t("servers.soon")}</span></h2>
    <p class="empty-box">{t("servers.partnersSoon")}</p>
  </section>

  <h2 class="section-title">{t("servers.popular")}</h2>
  <div class="filters">
    <Segmented label={t("servers.region")} value={region} options={regions} onchange={setRegion} />
    <span class="dim small">{t("servers.yourRegion", { region: regionName(detected) })}</span>
  </div>
  <div class="filters">
    <div class="grow"><SearchInput bind:value={query} placeholder={t("servers.search")} width="100%" /></div>
    <button class="btn" onclick={refresh}><RefreshCw size={14} /> {t("servers.refresh")}</button>
  </div>

  <div class="results">
    {#if typed}
      {@render row(t("servers.typed"), typed, null, [], 0)}
    {/if}
    {#each shown as s, n (s.id)}
      {@render row(s.name, s.address, s.region, s.tags, n + 1)}
    {:else}
      {#if !typed && servers.length}<p class="empty">{t("servers.nothing")}</p>{/if}
    {/each}
  </div>
</div>

<style>
  .view {
    height: 100%;
    overflow: auto;
    padding: var(--page-pad);
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    margin-bottom: 16px;
  }
  .target {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    font-size: var(--fs-13);
    white-space: nowrap;
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 8px;
    font-size: var(--fs-14);
    font-weight: 600;
  }
  .soon {
    padding: 1px 6px;
    border-radius: var(--radius-s);
    background: var(--accent-soft);
    color: var(--text-2);
    font-size: var(--fs-12);
    font-weight: 500;
  }
  .partners { margin-bottom: 24px; }
  .empty-box {
    margin: 0;
    padding: 16px;
    border: 1px dashed var(--border);
    border-radius: var(--radius);
    color: var(--text-3);
    font-size: var(--fs-13);
    text-align: center;
  }
  .filters {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px 16px;
    flex-wrap: wrap;
    margin-bottom: 8px;
  }
  .grow { flex: 1; min-width: 200px; }
  .small { font-size: var(--fs-12); }
  .results { border-top: 1px solid var(--border); margin-top: 8px; }
  .srv {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
  }
  .icon {
    width: 48px;
    height: 48px;
    flex-shrink: 0;
    border-radius: var(--radius);
    image-rendering: pixelated;
    background: var(--panel-2);
  }
  .ph {
    display: grid;
    place-items: center;
    color: var(--text-3);
  }
  .txt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name, .motd, .meta { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .name { display: flex; align-items: center; gap: 8px; }
  .name b { font-size: var(--fs-14); font-weight: 500; }
  .name small { color: var(--text-3); font-size: var(--fs-12); overflow: hidden; text-overflow: ellipsis; }
  .chip {
    flex-shrink: 0;
    padding: 0 6px;
    border-radius: var(--radius-s);
    background: var(--panel-2);
    color: var(--text-2);
    font-size: var(--fs-12);
  }
  .motd { color: var(--text-2); font-size: var(--fs-13); }
  .meta {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .meta > span { display: inline-flex; align-items: center; gap: 4px; flex-shrink: 0; }
  .meta .tags, .meta .ver { display: block; flex-shrink: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .off { color: var(--text-3); }
  .actions { display: flex; gap: 6px; flex-shrink: 0; }
  .empty {
    margin: 0;
    padding: 32px 0;
    color: var(--text-3);
    font-size: var(--fs-13);
    text-align: center;
  }
</style>
