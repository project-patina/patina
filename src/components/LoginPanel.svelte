<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { fade } from "svelte/transition";
  import { Copy, ExternalLink, KeyRound, LoaderCircle, ShieldCheck } from "@lucide/svelte";
  import { api, errText } from "../lib/api";
  import { dur, loadSkin, refreshAccounts, toast } from "../lib/state.svelte";
  import type { Account, DeviceCode } from "../lib/types";

  let { onsuccess }: { onsuccess: (a: Account) => void } = $props();

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
      if (msg !== "Anulowano") error = msg;
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
      if (msg !== "Anulowano") error = msg;
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
    <div in:fade={{ duration: dur(200) }} class="stack">
      <button class="ms" onclick={withWindow}>
        <svg width="18" height="18" viewBox="0 0 21 21" aria-hidden="true">
          <rect x="1" y="1" width="9" height="9" fill="#f25022" /><rect x="11" y="1" width="9" height="9" fill="#7fba00" />
          <rect x="1" y="11" width="9" height="9" fill="#00a4ef" /><rect x="11" y="11" width="9" height="9" fill="#ffb900" />
        </svg>
        Zaloguj przez Microsoft
      </button>
      <button class="btn ghost alt" onclick={withCode}><KeyRound size={15} /> Zaloguj kodem w przeglądarce</button>
      <p class="note"><ShieldCheck size={14} /> Logujesz się na oficjalnej stronie Microsoftu. Launcher nie widzi Twojego hasła.</p>
    </div>
  {:else if mode === "window"}
    <div in:fade={{ duration: dur(200) }} class="stack center">
      <LoaderCircle size={26} class="spin" />
      <p class="muted">Dokończ logowanie w oknie Microsoft…</p>
      <button class="btn sm" onclick={cancel}>Anuluj</button>
    </div>
  {:else if mode === "device" && code}
    <div in:fade={{ duration: dur(200) }} class="stack center">
      <p class="muted">Wejdź na stronę i wpisz kod (już skopiowany):</p>
      <button class="code mono" onclick={() => navigator.clipboard?.writeText(code!.userCode).then(() => toast("Skopiowano kod"))}>
        {code.userCode} <Copy size={15} />
      </button>
      <div class="row">
        <button class="btn primary" onclick={() => openUrl(code!.verificationUri)}><ExternalLink size={15} /> Otwórz {code.verificationUri.replace("https://", "")}</button>
        <button class="btn" onclick={cancel}>Anuluj</button>
      </div>
      <p class="dim small"><LoaderCircle size={13} class="spin" /> Czekam na potwierdzenie…</p>
    </div>
  {/if}
  {#if error}
    <p class="error selectable" transition:fade={{ duration: dur(150) }}>{error}</p>
  {/if}
</div>

<style>
  .login {
    width: 100%;
  }
  .stack {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .center {
    align-items: center;
    text-align: center;
    padding: 8px 0;
  }
  .ms {
    height: 48px;
    border-radius: var(--radius);
    background: var(--play-bg);
    color: var(--play-fg);
    font-weight: 650;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 12px;
    transition: transform 0.2s var(--ease), box-shadow 0.25s var(--ease);
  }
  .ms:hover {
    transform: translateY(-1px);
    box-shadow: 0 10px 30px -10px var(--accent);
  }
  .ms:active {
    transform: scale(0.98);
  }
  .alt {
    color: var(--text-2);
  }
  .note {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin: 6px 0 0;
    color: var(--text-3);
    font-size: 0.8rem;
  }
  .code {
    font-size: 1.7rem;
    font-weight: 700;
    letter-spacing: 0.18em;
    padding: 12px 20px;
    border-radius: var(--radius);
    background: var(--panel-2);
    border: 1px dashed var(--accent-line);
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .row {
    display: flex;
    gap: 8px;
  }
  .small {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.82rem;
    margin: 0;
  }
  .error {
    color: var(--danger);
    font-size: 0.88rem;
    text-align: center;
    margin: 12px 0 0;
    line-height: 1.45;
  }
  p {
    margin: 0;
  }
</style>
