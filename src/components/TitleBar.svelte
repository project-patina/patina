<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { Check, ChevronDown, LogOut, Minus, Plus, Square, X } from "@lucide/svelte";
  import Head from "./Head.svelte";
  import { api } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { APP_NAME } from "../lib/flavour";
  import { dialog } from "../lib/motion";
  import { activeAccount, app, refreshAccounts, toastError } from "../lib/state.svelte";

  let { showAccount = true }: { showAccount?: boolean } = $props();

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

<svelte:window
  onclick={(e) => menu && !(e.target as HTMLElement).closest(".account") && (menu = false)}
  onkeydown={(e) => e.key === "Escape" && (menu = false)}
/>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <img src="/icon.svg" alt="" width="16" height="16" />
    <span>{APP_NAME}</span>
  </div>

  <div class="right">
    {#if showAccount && acc}
      <div class="account">
        <button class="acc-btn" aria-expanded={menu} onclick={() => (menu = !menu)}>
          <Head skinUrl={acc.skinUrl} name={acc.name} size={18} radius={4} />
          <span>{acc.name}</span>
          <ChevronDown size={12} />
        </button>
        {#if menu}
          <div class="menu card" in:dialog out:dialog={{ duration: 100 }}>
            {#each app.accounts as a (a.uuid)}
              <div class="item">
                <button class="pick" onclick={() => switchTo(a.uuid)}>
                  <Head skinUrl={a.skinUrl} name={a.name} size={22} radius={4} />
                  <span>{a.name}</span>
                  {#if a.active}<Check size={14} class="check" />{/if}
                </button>
                <button class="btn ghost sm icon" title={t("account.logout")} aria-label={t("account.logout")} onclick={() => remove(a.uuid)}><LogOut size={14} /></button>
              </div>
            {/each}
            <button class="item add" onclick={() => { menu = false; app.addAccountOpen = true; }}>
              <Plus size={14} /> {t("account.add")}
            </button>
          </div>
        {/if}
      </div>
    {/if}

    <div class="win">
      <button onclick={() => win.minimize()} aria-label={t("window.minimize")}><Minus size={14} /></button>
      <button onclick={() => win.toggleMaximize()} aria-label={t("window.maximize")}><Square size={11} /></button>
      <button class="close" onclick={() => win.close().catch(toastError)} aria-label={t("window.close")}><X size={14} /></button>
    </div>
  </div>
</header>

<style>
  .titlebar {
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-left: 12px;
    flex-shrink: 0;
    position: relative;
    z-index: 30;
    border-bottom: 1px solid var(--border);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-13);
    font-weight: 600;
  }
  .brand img { pointer-events: none; }
  .right {
    display: flex;
    align-items: center;
    height: 100%;
    gap: 8px;
  }
  .account { position: relative; }
  .acc-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 8px 0 4px;
    border-radius: var(--radius-s);
    font-size: var(--fs-13);
    font-weight: 500;
    color: var(--text-2);
    transition: background 0.12s, color 0.12s;
  }
  .acc-btn:hover, .acc-btn[aria-expanded="true"] { background: var(--panel-2); color: var(--text); }
  .menu {
    position: absolute;
    right: 0;
    top: 32px;
    width: 248px;
    padding: 4px;
    box-shadow: var(--shadow);
  }
  .item {
    display: flex;
    align-items: center;
    gap: 4px;
    border-radius: var(--radius-s);
    width: 100%;
    padding-right: 4px;
  }
  .item:hover { background: var(--panel-2); }
  .pick {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 8px;
    font-size: var(--fs-13);
    font-weight: 500;
    text-align: left;
  }
  .pick span { overflow: hidden; text-overflow: ellipsis; }
  .pick :global(.check) { margin-left: auto; flex-shrink: 0; color: var(--accent); }
  .add {
    height: 32px;
    gap: 8px;
    padding: 0 8px;
    margin-top: 4px;
    color: var(--text-2);
    font-size: var(--fs-13);
    font-weight: 500;
  }
  .add:hover { color: var(--text); }
  .win { display: flex; height: 100%; }
  .win button {
    width: 40px;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--text-2);
    transition: background 0.12s, color 0.12s;
  }
  .win button:hover { background: var(--panel-2); color: var(--text); }
  .win .close:hover { background: #c94a4a; color: #fff; }
</style>
