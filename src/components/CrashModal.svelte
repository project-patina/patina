<script lang="ts">
  import { Copy, FolderOpen, SquareTerminal } from "@lucide/svelte";
  import Modal from "./Modal.svelte";
  import { api } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import { app, navigate, toast, toastError } from "../lib/state.svelte";

  const crash = $derived(app.crash!);
  const name = $derived(app.instances.find((i) => i.id === crash.id)?.name ?? crash.id);
  const close = () => (app.crash = null);
</script>

<Modal onclose={close} width={680}>
  <h3>{t("crash.title", { name })}</h3>
  <p>{t("crash.body", { code: crash.code ?? "—" })}</p>
  <pre class="tail selectable mono">{crash.tail.join("\n")}</pre>
  <div class="actions">
    <button class="btn" onclick={() => navigator.clipboard?.writeText(crash.tail.join("\n")).then(() => toast(t("crash.copied"), "success")).catch(toastError)}><Copy size={14} /> {t("common.copy")}</button>
    <button class="btn" onclick={() => api.openFolder(crash.id, "crash-reports").catch(() => api.openFolder(crash.id).catch(toastError))}><FolderOpen size={14} /> {t("crash.reports")}</button>
    <button class="btn console" onclick={() => { navigate("instance", crash.id, "console"); close(); }}><SquareTerminal size={14} /> {t("crash.console")}</button>
    <button class="btn primary" onclick={close}>OK</button>
  </div>
</Modal>

<style>
  h3 { font-size: var(--fs-16); }
  p {
    margin: 4px 0 0;
    color: var(--text-2);
    font-size: var(--fs-13);
    line-height: 1.5;
  }
  .tail {
    margin: 16px 0;
    max-height: 300px;
    overflow: auto;
    padding: 12px;
    border-radius: var(--radius-s);
    background: var(--bg-2);
    border: 1px solid var(--border);
    font-size: var(--fs-12);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .console { margin-left: auto; }
</style>
