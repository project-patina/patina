<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { untrack } from "svelte";
  import { Ban, Check, LoaderCircle, Plus, RefreshCw, Trash2, Upload } from "@lucide/svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import CapeThumb from "../components/CapeThumb.svelte";
  import Group from "../components/Group.svelte";
  import Head from "../components/Head.svelte";
  import Row from "../components/Row.svelte";
  import Segmented from "../components/Segmented.svelte";
  import { api, errText, mediaUrl } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { reveal } from "../lib/motion";
  import { activeAccount, app, loadSkin, toast, toastError } from "../lib/state.svelte";
  import type { Image, McProfile } from "../lib/types";

  const acc = $derived(activeAccount());
  let profile = $state<McProfile | null>(null);
  let loadingProfile = $state(false);
  let profileError = $state("");
  let library = $state<Image[]>([]);
  let libData = $state<Record<string, string>>({});
  let capeData = $state<Record<string, string>>({});
  let busy = $state<"skin" | "cape" | null>(null);

  // Preview selection (not applied until the player confirms).
  let previewSkin = $state<string | null>(null); // library id, or null = current skin
  let previewCape = $state<string | null | undefined>(undefined); // cape id, null = none, undefined = current
  let slim = $state(false);

  const models = $derived([{ value: "classic" as const, label: t("wardrobe.classic") }, { value: "slim" as const, label: t("wardrobe.slim") }]);

  async function toDataUrl(url: string): Promise<string> {
    const blob = await (await fetch(url)).blob();
    return new Promise((res, rej) => {
      const r = new FileReader();
      r.onload = () => res(r.result as string);
      r.onerror = rej;
      r.readAsDataURL(blob);
    });
  }

  async function loadProfile() {
    if (!acc) return;
    const uuid = acc.uuid;
    loadingProfile = true;
    profileError = "";
    try {
      const loaded = await api.getProfile(uuid);
      // The player may have switched accounts while this was on its way.
      if (uuid !== acc?.uuid) return;
      profile = loaded;
      app.accounts = await api.listAccounts();
      slim = activeAccount()?.skinSlim ?? false;
      for (const c of profile.capes) {
        if (!capeData[c.id]) api.skinDataUrl(c.url).then((d) => (capeData[c.id] = d)).catch(() => {});
      }
      const a = activeAccount();
      if (a) await loadSkin(a);
    } catch (e) {
      if (uuid === acc?.uuid) profileError = errText(e);
    } finally {
      loadingProfile = false;
    }
  }

  // Cape texture of the account itself (works even when the profile call fails).
  $effect(() => {
    const url = acc?.capeUrl;
    if (url && !app.skins[url]) api.skinDataUrl(url).then((d) => (app.skins[url] = d)).catch(() => {});
  });

  async function loadLibrary() {
    library = await api.listImages("skin").catch(() => []);
    for (const s of library) if (!libData[s.id]) toDataUrl(mediaUrl("skin", s.id)).then((d) => (libData[s.id] = d)).catch(() => {});
  }

  // Only an account switch resets the page. loadProfile replaces
  // app.accounts, so tracking `acc` itself would re-run this forever.
  const accUuid = $derived(acc?.uuid);
  $effect(() => {
    void accUuid;
    untrack(() => {
      slim = acc?.skinSlim ?? false;
      previewSkin = null;
      previewCape = undefined;
      loadProfile();
      loadLibrary();
    });
  });

  const currentSkin = $derived(acc?.skinUrl ? app.skins[acc.skinUrl] : undefined);
  const activeCape = $derived(profile?.capes.find((c) => c.state === "ACTIVE") ?? null);
  const shownSkin = $derived(previewSkin ? libData[previewSkin] : currentSkin);
  const shownCapeId = $derived(previewCape === undefined ? (activeCape?.id ?? null) : previewCape);
  // Without a loaded profile, fall back to the cape saved on the account.
  const accountCape = $derived(acc?.capeUrl ? (app.skins[acc.capeUrl] ?? null) : null);
  const shownCape = $derived(shownCapeId ? (capeData[shownCapeId] ?? null) : previewCape === undefined && !profile ? accountCape : null);
  const capeChanged = $derived(previewCape !== undefined && previewCape !== (activeCape?.id ?? null));
  const skinChanged = $derived(previewSkin !== null || slim !== (acc?.skinSlim ?? false));

  async function applyCape() {
    if (!acc) return;
    busy = "cape";
    try {
      app.accounts = await api.setCape(acc.uuid, previewCape ?? null);
      await loadProfile();
      previewCape = undefined;
      toast(t(shownCapeId ? "wardrobe.capeOn" : "wardrobe.capeOff"), "success", 6000);
    } catch (e) {
      toastError(e);
    } finally {
      busy = null;
    }
  }

  async function applySkin() {
    if (!acc) return;
    busy = "skin";
    try {
      app.accounts = previewSkin ? await api.uploadSkin(acc.uuid, previewSkin, null, slim) : await api.setSkinVariant(acc.uuid, slim);
      await loadProfile();
      previewSkin = null;
      toast(t("wardrobe.skinChanged"), "success");
    } catch (e) {
      toastError(e);
    } finally {
      busy = null;
    }
  }

  async function addSkins() {
    const picked = await open({ multiple: true, title: t("wardrobe.pickSkin"), filters: [{ name: t("wardrobe.skin"), extensions: ["png"] }] });
    const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
    let last: string | null = null;
    for (const p of paths) {
      try {
        last = (await api.importImage("skin", p)).id;
      } catch (e) {
        toastError(e);
      }
    }
    await loadLibrary();
    if (last) previewSkin = last;
  }

  async function removeSkin(id: string) {
    await api.deleteImage("skin", id).catch(toastError);
    if (previewSkin === id) previewSkin = null;
    await loadLibrary();
  }
</script>

<div class="view">
  <h1 class="page-title">{t("nav.wardrobe")}</h1>
  {#if !acc}
    <p class="hint">{t("wardrobe.signIn")}</p>
  {:else}
    <div class="grid">
      <section class="stage card" in:reveal|global>
        {#if shownSkin}
          <SkinViewer skin={shownSkin} cape={shownCape} {slim} width={256} height={372} zoom={0.78} />
        {:else}
          <div class="noskin"><Head name={acc.name} size={72} radius={8} /><p class="hint">{t("wardrobe.defaultSkin")}</p></div>
        {/if}
        <p class="hint">{t("wardrobe.dragHint")}</p>
      </section>

      <div class="side">
        <div in:reveal|global={{ index: 1 }}>
          <Group title={t("wardrobe.skin")}>
            <Row title={t("wardrobe.model")}>
              <Segmented label={t("wardrobe.model")} value={slim ? "slim" : "classic"} options={models} onchange={(v) => (slim = v === "slim")} />
            </Row>
            <div class="thumbs">
              <button class="thumb" aria-pressed={previewSkin === null} onclick={() => (previewSkin = null)} title={t("wardrobe.current")}>
                <Head skinUrl={acc.skinUrl} name={acc.name} size={40} radius={6} />
                <small>{t("wardrobe.current")}</small>
              </button>
              {#each library as s (s.id)}
                <div class="thumb" class:on={previewSkin === s.id}>
                  <button class="hit" aria-pressed={previewSkin === s.id} onclick={() => (previewSkin = s.id)} title={s.name} aria-label={s.name}></button>
                  <Head src={libData[s.id] ?? null} name={s.name} size={40} radius={6} />
                  <small>{s.name.replace(/\.png$/i, "") || t("wardrobe.skin")}</small>
                  <button class="del" onclick={() => removeSkin(s.id)} title={t("common.delete")} aria-label={t("common.delete")}><Trash2 size={12} /></button>
                </div>
              {/each}
              <button class="thumb add" onclick={addSkins}><Plus size={16} /><small>{t("common.add")}</small></button>
            </div>
            <div class="actions">
              <span class="hint">{t("wardrobe.skinHint")}</span>
              <button class="btn primary" disabled={!skinChanged || !!busy} onclick={applySkin}>
                {#if busy === "skin"}<LoaderCircle size={14} class="spin" />{:else}<Upload size={14} />{/if} {t("wardrobe.applySkin")}
              </button>
            </div>
          </Group>
        </div>

        <div in:reveal|global={{ index: 2 }}>
          <Group title={t("wardrobe.capes")}>
            {#if profileError}
              <p class="err selectable">{profileError}</p>
            {:else if !profile}
              <div class="center"><LoaderCircle size={16} class="spin" /></div>
            {:else}
              <div class="thumbs">
                <button class="thumb" aria-pressed={shownCapeId === null} onclick={() => (previewCape = null)}>
                  <span class="none"><Ban size={16} /></span>
                  <small>{t("wardrobe.noCape")}</small>
                </button>
                {#each profile.capes as c (c.id)}
                  <button class="thumb" aria-pressed={shownCapeId === c.id} onclick={() => (previewCape = c.id)} title={c.alias ?? t("wardrobe.cape")}>
                    {#if capeData[c.id]}<CapeThumb src={capeData[c.id]} width={32} />{:else}<span class="none"><LoaderCircle size={14} class="spin" /></span>{/if}
                    <small>{c.alias ?? t("wardrobe.cape")}</small>
                    {#if c.state === "ACTIVE"}<span class="tick" title={t("wardrobe.wearing")}><Check size={12} /></span>{/if}
                  </button>
                {/each}
              </div>
              {#if profile.capes.length === 0}
                <p class="hint">{t("wardrobe.noCapes")}</p>
              {/if}
            {/if}
            <div class="actions">
              <button class="btn ghost" onclick={loadProfile} disabled={loadingProfile}><RefreshCw size={14} class={loadingProfile ? "spin" : ""} /> {t("common.refresh")}</button>
              {#if profile && !profileError}
                <button class="btn primary" disabled={!capeChanged || !!busy} onclick={applyCape}>{#if busy === "cape"}<LoaderCircle size={14} class="spin" />{/if} {t("wardrobe.applyCape")}</button>
              {/if}
            </div>
          </Group>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .view {
    height: 100%;
    overflow: auto;
    padding: var(--page-pad);
  }
  .page-title { margin-bottom: 16px; }
  .grid {
    display: grid;
    grid-template-columns: 288px minmax(0, 1fr);
    align-items: start;
    gap: 24px;
  }
  .stage {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 12px;
  }
  .noskin {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 372px;
  }
  .hint {
    margin: 0;
    color: var(--text-3);
    font-size: var(--fs-12);
    line-height: 1.4;
  }
  .side {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  .thumbs {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(92px, 1fr));
    gap: 8px;
    padding: 12px 0;
  }
  .thumb {
    position: relative;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px 4px;
    border-radius: var(--radius);
    border: 1px solid var(--border);
    transition: border-color 0.12s;
  }
  .thumb:hover { border-color: var(--border-2); }
  .thumb[aria-pressed="true"], .thumb.on { border-color: var(--accent); }
  .thumb small {
    max-width: 100%;
    color: var(--text-2);
    font-size: var(--fs-12);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .hit {
    position: absolute;
    inset: 0;
    border-radius: inherit;
  }
  .add { border-style: dashed; color: var(--text-3); }
  .add:hover { color: var(--text); }
  .del {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 20px;
    height: 20px;
    display: grid;
    place-items: center;
    border-radius: 4px;
    color: var(--text-2);
    background: var(--panel-3);
    opacity: 0;
    transition: opacity 0.12s;
  }
  .thumb:hover .del, .del:focus-visible { opacity: 1; }
  .none {
    width: 32px;
    height: 51px;
    display: grid;
    place-items: center;
    color: var(--text-3);
  }
  .tick {
    position: absolute;
    top: 4px;
    right: 4px;
    display: grid;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }
  .err {
    margin: 12px 0;
    color: var(--danger);
    font-size: var(--fs-13);
  }
  .center {
    display: grid;
    place-items: center;
    padding: 16px;
    color: var(--text-3);
  }
  @media (max-width: 760px) {
    .grid { grid-template-columns: minmax(0, 1fr); }
    .stage { position: static; }
  }
</style>
