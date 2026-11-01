<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { untrack } from "svelte";
  import { fly } from "svelte/transition";
  import { Ban, Check, LoaderCircle, Plus, RefreshCw, Shirt, Trash2, Upload } from "@lucide/svelte";
  import SkinViewer from "../components/SkinViewer.svelte";
  import CapeThumb from "../components/CapeThumb.svelte";
  import Head from "../components/Head.svelte";
  import { api, errText, mediaUrl } from "../lib/api";
  import { activeAccount, app, dur, loadSkin, toast, toastError, updateSettings } from "../lib/state.svelte";
    import type { Image, McProfile } from "../lib/types";

  const acc = $derived(activeAccount());
  let profile = $state<McProfile | null>(null);
  let loadingProfile = $state(false);
  let profileError = $state("");
  let library = $state<Image[]>([]);
  let libData = $state<Record<string, string>>({});
  let capeData = $state<Record<string, string>>({});
  let busy = $state(false);

  // Preview selection (not applied until the player confirms).
  let previewSkin = $state<string | null>(null); // library id, or null = current skin
  let previewCape = $state<string | null | undefined>(undefined); // cape id, null = none, undefined = current
  let slim = $state(false);


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
    loadingProfile = true;
    profileError = "";
    try {
      profile = await api.getProfile(acc.uuid);
      app.accounts = await api.listAccounts();
      slim = activeAccount()?.skinSlim ?? false;
      for (const c of profile.capes) {
        if (!capeData[c.id]) api.skinDataUrl(c.url).then((d) => (capeData[c.id] = d)).catch(() => {});
      }
      const a = activeAccount();
      if (a) await loadSkin(a);
    } catch (e) {
      profileError = errText(e);
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
    busy = true;
    try {
      app.accounts = await api.setCape(acc.uuid, previewCape ?? null);
      await loadProfile();
      previewCape = undefined;
      toast(shownCapeId ? "Peleryna założona — w grze zobaczysz ją po ponownym wejściu" : "Peleryna zdjęta", "success", 6000);
    } catch (e) {
      toastError(e);
    } finally {
      busy = false;
    }
  }

  async function applySkin() {
    if (!acc) return;
    busy = true;
    try {
      app.accounts = previewSkin ? await api.uploadSkin(acc.uuid, previewSkin, null, slim) : await api.setSkinVariant(acc.uuid, slim);
      await loadProfile();
      previewSkin = null;
      toast("Skin zmieniony", "success");
    } catch (e) {
      toastError(e);
    } finally {
      busy = false;
    }
  }

  async function addSkins() {
    const picked = await open({ multiple: true, title: "Wybierz skin (PNG 64×64)", filters: [{ name: "Skin", extensions: ["png"] }] });
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

<div class="page">
  <h1>Garderoba</h1>
  {#if !acc}
    <p class="dim">Zaloguj się, aby zmieniać skin i peleryny.</p>
  {:else}
    <div class="grid">
      <section class="stage card" style:--stage="var(--accent)">
        <div class="glow"></div>
        {#if shownSkin}
          <SkinViewer skin={shownSkin} cape={shownCape} {slim} width={300} height={420} zoom={0.78} />
        {:else}
          <div class="noskin"><Head name={acc.name} size={96} radius={22} /><p class="dim">Domyślny skin (Steve/Alex)</p></div>
        {/if}
        <p class="hint dim">Przeciągnij, aby obrócić</p>
      </section>

      <div class="side">
        <section class="card block" in:fly={{ y: 10, duration: dur(300) }}>
          <div class="sec-head">
            <h3><Shirt size={17} /> Skin</h3>
            <div class="seg">
              <button class:on={!slim} onclick={() => (slim = false)}>Klasyczny</button>
              <button class:on={slim} onclick={() => (slim = true)}>Smukły</button>
            </div>
          </div>
          <div class="skins">
            <button class="skin" class:on={previewSkin === null} onclick={() => (previewSkin = null)} title="Obecny skin">
              <Head skinUrl={acc.skinUrl} name={acc.name} size={44} radius={8} />
              <small>Obecny</small>
            </button>
            {#each library as s (s.id)}
              <div class="skin" class:on={previewSkin === s.id}>
                <button class="hit" onclick={() => (previewSkin = s.id)} title={s.name} aria-label={s.name}></button>
                <Head src={libData[s.id] ?? null} name={s.name} size={44} radius={8} />
                <small>{s.name.replace(/\.png$/i, "").slice(0, 10) || "Skin"}</small>
                <button class="del" onclick={() => removeSkin(s.id)} aria-label="Usuń"><Trash2 size={11} /></button>
              </div>
            {/each}
            <button class="skin add" onclick={addSkins}><Plus size={20} /><small>Dodaj</small></button>
          </div>
          <div class="actions">
            <span class="dim small">Zmiana skina działa od razu na serwerach i w grze.</span>
            <button class="btn primary" disabled={!skinChanged || busy} onclick={applySkin}>
              {#if busy}<LoaderCircle size={15} class="spin" />{:else}<Upload size={15} />{/if} Ustaw skin
            </button>
          </div>
        </section>

        <section class="card block" in:fly={{ y: 10, duration: dur(300), delay: dur(60) }}>
          <div class="sec-head">
            <h3>Peleryny</h3>
            <button class="btn ghost sm" onclick={loadProfile} disabled={loadingProfile}><RefreshCw size={14} class={loadingProfile ? "spin" : ""} /></button>
          </div>
          {#if profileError}
            <p class="err">{profileError}</p>
          {:else if !profile}
            <div class="center"><LoaderCircle size={20} class="spin" /></div>
          {:else}
            <div class="capes">
              <button class="cape" class:on={shownCapeId === null} onclick={() => (previewCape = null)}>
                <span class="none"><Ban size={22} /></span>
                <small>Bez peleryny</small>
              </button>
              {#each profile.capes as c (c.id)}
                <button class="cape" class:on={shownCapeId === c.id} onclick={() => (previewCape = c.id)}>
                  {#if capeData[c.id]}<CapeThumb src={capeData[c.id]} width={40} />{:else}<span class="none"><LoaderCircle size={16} class="spin" /></span>{/if}
                  <small>{c.alias ?? "Peleryna"}</small>
                  {#if c.state === "ACTIVE"}<span class="tick"><Check size={11} /></span>{/if}
                </button>
              {/each}
            </div>
            {#if profile.capes.length === 0}
              <p class="dim small">To konto nie ma żadnych peleryn (dostaje się je np. za Minecon, migrację konta czy wydarzenia).</p>
            {/if}
            <div class="actions">
              <span></span>
              <button class="btn primary" disabled={!capeChanged || busy} onclick={applyCape}>{#if busy}<LoaderCircle size={15} class="spin" />{/if} Załóż</button>
            </div>
          {/if}
        </section>
      </div>
    </div>
  {/if}
</div>

<style>
  .page {
    height: 100%;
    overflow: auto;
    padding: 6px 22px 22px 6px;
  }
  h1 {
    font-size: 1.65rem;
    font-weight: 700;
    margin: 4px 0 14px;
  }
  .grid {
    display: grid;
    grid-template-columns: 360px minmax(0, 1fr);
    gap: 16px;
  }
  .stage {
    position: relative;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 18px 12px 14px;
    min-height: 520px;
  }
  .glow {
    position: absolute;
    left: 50%;
    top: 45%;
    width: 340px;
    height: 340px;
    transform: translate(-50%, -50%);
    border-radius: 50%;
    background: radial-gradient(closest-side, color-mix(in srgb, var(--accent) 30%, transparent), transparent);
    pointer-events: none;
  }
  .stage :global(canvas) {
    position: relative;
  }
  .noskin {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }
  .hint {
    font-size: 0.76rem;
    margin: 10px 0 0;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .block {
    padding: 16px 18px;
  }
  .sec-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }
  h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1.05rem;
  }
  .seg {
    display: flex;
    padding: 3px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }
  .seg button {
    padding: 5px 11px;
    border-radius: 7px;
    font-weight: 600;
    font-size: 0.84rem;
    color: var(--text-2);
  }
  .seg button.on {
    background: var(--panel-3);
    color: var(--text);
  }
  .skins,
  .capes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(78px, 1fr));
    gap: 8px;
  }
  .skin,
  .cape {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 10px 6px 8px;
    border-radius: var(--radius);
    background: var(--panel-2);
    border: 2px solid transparent;
    transition: border-color 0.15s, transform 0.2s var(--ease);
  }
  .skin:hover,
  .cape:hover {
    transform: translateY(-2px);
  }
  .skin.on,
  .cape.on {
    border-color: var(--accent);
  }
  .skin small,
  .cape small {
    font-size: 0.72rem;
    color: var(--text-2);
    text-align: center;
    line-height: 1.2;
  }
  .hit {
    position: absolute;
    inset: 0;
  }
  .add {
    border: 1px dashed var(--border-2);
    color: var(--text-3);
    justify-content: center;
  }
  .add:hover {
    color: var(--accent);
  }
  .del {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 5px;
    display: grid;
    place-items: center;
    background: rgb(0 0 0 / 0.5);
    color: white;
    opacity: 0;
  }
  .skin:hover .del {
    opacity: 1;
  }
  .none {
    width: 40px;
    height: 64px;
    display: grid;
    place-items: center;
    color: var(--text-3);
  }
  .tick {
    position: absolute;
    top: 5px;
    right: 5px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--success);
    color: white;
    display: grid;
    place-items: center;
  }
  .actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-top: 14px;
  }
  .small {
    font-size: 0.8rem;
  }
  .err {
    color: var(--danger);
    font-size: 0.88rem;
  }
  .center {
    display: grid;
    place-items: center;
    padding: 20px;
  }
  @media (max-width: 1000px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
