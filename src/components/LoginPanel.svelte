<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Copy, ExternalLink, KeyRound, LoaderCircle } from "@lucide/svelte";
  import { api, errText } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { page } from "../lib/motion";
  import { loadSkin, refreshAccounts, toast, toastError } from "../lib/state.svelte";
  import type { Account, DeviceCode } from "../lib/types";

  let { onsuccess }: { onsuccess: (a: Account) => void } = $props();

  // The backend's Error::Cancelled, in either language: closing the sign-in is not a failure.
  const CANCELLED = ["Anulowano", "Cancelled"];

  let mode = $state<"idle" | "window" | "device">("idle");
  let code = $state<DeviceCode | null>(null);
  let error = $state("");

  async function done(a: Account) {
    await refreshAccounts();
    await loadSkin(a);
    mode = "idle";
    onsuccess(a);
  }

  async function withWindow() {
    error = "";
    mode = "window";
    try {
      await done(await api.loginMicrosoft());
    } catch (e) {
      const msg = errText(e);
      if (!CANCELLED.includes(msg)) error = msg;
      mode = "idle";
    }
  }

  async function withCode() {
    error = "";
    try {
      code = await api.loginDeviceStart();
      mode = "device";
      await navigator.clipboard?.writeText(code.userCode).catch(() => {});
      await done(await api.loginDeviceFinish(code));
    } catch (e) {
      const msg = errText(e);
      if (!CANCELLED.includes(msg)) error = msg;
      mode = "idle";
    }
  }

  function cancel() {
    void api.cancelLogin();
    mode = "idle";
  }
</script>

<div class="login">
  {#if mode === "idle"}
    <div in:page class="stack">
      <button class="btn primary ms" onclick={withWindow}>
        <svg width="14" height="14" viewBox="0 0 21 21" fill="currentColor" aria-hidden="true">
          <rect x="1" y="1" width="9" height="9" /><rect x="11" y="1" width="9" height="9" />
          <rect x="1" y="11" width="9" height="9" /><rect x="11" y="11" width="9" height="9" />
        </svg>
        {t("login.microsoft")}
      </button>
      <button class="btn ghost" onclick={withCode}><KeyRound size={14} /> {t("login.code")}</button>
      <p class="note">{t("login.note")}</p>
    </div>
  {:else if mode === "window"}
    <div in:page class="stack center">
      <LoaderCircle size={20} class="spin" />
      <p class="muted">{t("login.finishInWindow")}</p>
      <button class="btn sm" onclick={cancel}>{t("common.cancel")}</button>
    </div>
  {:else if mode === "device" && code}
    <div in:page class="stack center">
      <p class="muted">{t("login.enterCode")}</p>
      <button class="code mono" title={t("common.copy")} onclick={() => navigator.clipboard?.writeText(code!.userCode).then(() => toast(t("login.codeCopied"))).catch(toastError)}>
        {code.userCode} <Copy size={14} />
      </button>
      <div class="row">
        <button class="btn primary" onclick={() => openUrl(code!.verificationUri).catch(toastError)}><ExternalLink size={14} /> {t("login.openPage", { site: code.verificationUri.replace("https://", "") })}</button>
        <button class="btn" onclick={cancel}>{t("common.cancel")}</button>
      </div>
      <p class="note"><LoaderCircle size={12} class="spin" /> {t("login.waiting")}</p>
    </div>
  {/if}
  {#if error}
    <p class="error selectable">{error}</p>
  {/if}
</div>

<style>
  .login { width: 100%; }
  .stack {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .center {
    align-items: center;
    text-align: center;
    padding: 8px 0;
  }
  .ms { height: 40px; font-size: var(--fs-14); }
  .muted { font-size: var(--fs-13); }
  .note {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    margin-top: 4px;
    color: var(--text-3);
    font-size: var(--fs-12);
    line-height: 1.4;
    text-align: center;
  }
  .code {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    border-radius: var(--radius);
    background: var(--panel-2);
    border: 1px solid var(--border-2);
    font-size: var(--fs-20);
    font-weight: 600;
    letter-spacing: 0.08em;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 8px;
  }
  .error {
    margin-top: 12px;
    color: var(--danger);
    font-size: var(--fs-13);
    line-height: 1.4;
    text-align: center;
  }
  p { margin: 0; }
</style>
