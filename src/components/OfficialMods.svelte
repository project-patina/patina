<script lang="ts">
  import { ArrowUpCircle, Download, LoaderCircle, Package, Trash2 } from "@lucide/svelte";
  import { api, errText } from "../lib/api";
  import { bytes } from "../lib/format";
  import { lang, t } from "../lib/i18n.svelte";
  import { toast, toastError } from "../lib/state.svelte";
  import type { Instance, OfficialCatalog, OfficialMod } from "../lib/types";

  let { inst }: { inst: Instance } = $props();

  let catalog = $state<OfficialCatalog | null>(null);
  let error = $state("");
  let busy = $state<Record<string, boolean>>({});

  let seq = 0;
  async function load() {
    const my = ++seq;
    try {
      const list = await api.officialMods(inst.id);
      if (my !== seq) return;
      catalog = list;
      error = "";
    } catch (e) {
      if (my === seq) error = errText(e);
    }
  }

  $effect(() => {
    void inst.id;
    catalog = null;
    error = "";
    load();
  });

  async function act(id: string, name: string, work: Promise<void>, message: "official.installed" | "official.removed") {
    busy[id] = true;
    try {
      await work;
      await load();
      toast(t(message, { name }), "success");
    } catch (e) {
      toastError(e);
    } finally {
      delete busy[id];
    }
  }

  const install = (m: OfficialMod) => act(m.id, m.name, api.installOfficialMod(inst.id, m.id), "official.installed");
  const remove = (id: string, name: string) => act(id, name, api.removeOfficialMod(inst.id, id), "official.removed");
</script>

{#if error}
  <p class="empty warn selectable">{error}</p>
{:else if !catalog}
  <div class="empty"><LoaderCircle size={16} class="spin" /></div>
{:else if !catalog.mods.length && !catalog.local.length}
  <p class="empty">{t(catalog.available ? "official.empty" : "official.unavailable")}</p>
{:else}
  {#if !catalog.available}<p class="note">{t("official.unavailable")}</p>{/if}
  <div class="list">
    {#each catalog.mods as m (m.id)}
      <div class="row">
        {#if m.icon}<img src={m.icon} alt="" loading="lazy" />{:else}<span class="ph"><Package size={14} /></span>{/if}
        <div class="info">
          <b>{m.name}</b>
          <small>{m.summary[lang()]}</small>
          <small class="mono">{m.version} · {bytes(m.size)}</small>
        </div>
        {#if m.state === "available"}
          <button class="btn sm primary" disabled={busy[m.id]} onclick={() => install(m)}>
            {#if busy[m.id]}<LoaderCircle size={14} class="spin" />{:else}<Download size={14} />{/if} {t("common.install")}
          </button>
        {:else}
          {#if m.state === "outdated"}
            <button class="btn sm" disabled={busy[m.id]} onclick={() => install(m)}>
              {#if busy[m.id]}<LoaderCircle size={14} class="spin" />{:else}<ArrowUpCircle size={14} />{/if} {t("official.update")}
            </button>
          {:else}
            <span class="state">{t("official.isInstalled")}</span>
          {/if}
          <button class="btn ghost sm icon danger" title={t("common.delete")} aria-label={t("common.delete")} disabled={busy[m.id]} onclick={() => remove(m.id, m.name)}><Trash2 size={14} /></button>
        {/if}
      </div>
    {/each}
    {#each catalog.local as id (id)}
      <div class="row">
        <span class="ph"><Package size={14} /></span>
        <div class="info">
          <b>{id}</b>
          <small>{t("official.local")}</small>
        </div>
        <button class="btn ghost sm icon danger" title={t("common.delete")} aria-label={t("common.delete")} disabled={busy[id]} onclick={() => remove(id, id)}><Trash2 size={14} /></button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .list { border-top: 1px solid var(--border); }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 56px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child { border-bottom: 0; }
  .row img,
  .ph {
    width: 32px;
    height: 32px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    border-radius: var(--radius-s);
    object-fit: cover;
    background: var(--panel-2);
    color: var(--text-3);
  }
  .info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .info b {
    font-size: var(--fs-13);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .info small {
    color: var(--text-3);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .state {
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .note {
    margin: 0 0 8px;
    color: var(--text-3);
    font-size: var(--fs-12);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 32px 16px;
    color: var(--text-3);
    font-size: var(--fs-13);
    line-height: 1.5;
    text-align: center;
  }
  .warn { color: var(--warning); }
</style>
