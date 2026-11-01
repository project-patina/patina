<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { fly } from "svelte/transition";
  import { Check, ChevronDown, LogOut, Minus, Plus, Square, X } from "@lucide/svelte";
  import Head from "./Head.svelte";
  import { api } from "../lib/api";
  import { activeAccount, app, dur, refreshAccounts, toastError } from "../lib/state.svelte";

  let { onAddAccount, showAccount = true }: { onAddAccount?: () => void; showAccount?: boolean } = $props();

  const win = getCurrentWindow();
  let menu = $state(false);
  const acc = $derived(activeAccount());

  async function switchTo(uuid: string) {
    try {
      app.accounts = await api.setActiveAccount(uuid);
    } catch (e) {
      toastError(e);
    }
    menu = false;
  }

  async function remove(uuid: string) {
    try {
      app.accounts = await api.removeAccount(uuid);
      await refreshAccounts();
    } catch (e) {
      toastError(e);
    }
    menu = false;
  }
</script>

<svelte:window onclick={(e) => menu && !(e.target as HTMLElement).closest(".account") && (menu = false)} />

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <img src="/icon.svg" alt="" width="20" height="20" />
    <span>PATINA</span>
  </div>

  <div class="right">
    {#if showAccount && acc}
      <div class="account">
        <button class="acc-btn" onclick={() => (menu = !menu)}>
          <Head skinUrl={acc.skinUrl} name={acc.name} size={22} radius={5} />
          <span>{acc.name}</span>
          <ChevronDown size={14} />
        </button>
        {#if menu}
          <div class="menu card glass" transition:fly={{ y: -6, duration: dur(180) }}>
            <div class="label" style="padding: 6px 10px 8px">Konta Microsoft</div>
            {#each app.accounts as a (a.uuid)}
              <div class="item" class:active={a.active}>
                <button class="pick" onclick={() => switchTo(a.uuid)}>
                  <Head skinUrl={a.skinUrl} name={a.name} size={26} radius={6} />
                  <span>{a.name}</span>
                  {#if a.active}<Check size={15} class="check" />{/if}
                </button>
                <button class="btn ghost sm icon" title="Wyloguj" onclick={() => remove(a.uuid)}><LogOut size={14} /></button>
              </div>
            {/each}
            <button class="item add" onclick={() => { menu = false; onAddAccount?.(); }}>
              <Plus size={16} /> Dodaj konto
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="win">
      <button onclick={() => win.minimize()} aria-label="Minimalizuj"><Minus size={15} /></button>
      <button onclick={() => win.toggleMaximize()} aria-label="Maksymalizuj"><Square size={12} /></button>
      <button class="close" onclick={() => win.close().catch(toastError)} aria-label="Zamknij"><X size={16} /></button>
    </div>
  </div>
</header>

<style>
  .titlebar {
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 18px;
    flex-shrink: 0;
    position: relative;
    z-index: 30;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-display);
    font-weight: 800;
    letter-spacing: 0.06em;
    font-size: 0.95rem;
  }
  .brand img {
    pointer-events: none;
  }
  .right {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .account {
    position: relative;
  }
  .acc-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 10px 0 5px;
    border-radius: 9px;
    background: var(--panel);
    border: 1px solid var(--border);
    font-weight: 600;
    font-size: 0.88rem;
    transition: background 0.18s, border-color 0.18s;
  }
  .acc-btn:hover {
    background: var(--panel-2);
    border-color: var(--border-2);
  }
  .menu {
    position: absolute;
    right: 0;
    top: 40px;
    width: 260px;
    padding: 6px;
    border-radius: var(--radius);
    box-shadow: var(--shadow);
  }
  .item {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: 8px;
    width: 100%;
  }
  .item:hover {
    background: var(--panel-2);
  }
  .pick {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    font-weight: 560;
    text-align: left;
  }
  .pick :global(.check) {
    margin-left: auto;
    color: var(--accent);
  }
  .add {
    padding: 9px 10px;
    gap: 10px;
    color: var(--text-2);
    font-weight: 560;
    margin-top: 4px;
    border-top: 1px solid var(--border);
    border-radius: 0 0 8px 8px;
  }
  .add:hover {
    color: var(--text);
  }
  .win {
    display: flex;
  }
  .win button {
    width: 38px;
    height: 32px;
    display: grid;
    place-items: center;
    border-radius: 8px;
    color: var(--text-2);
    transition: background 0.15s, color 0.15s;
  }
  .win button:hover {
    background: var(--panel-2);
    color: var(--text);
  }
  .win .close:hover {
    background: #e5484d;
    color: white;
  }
</style>
