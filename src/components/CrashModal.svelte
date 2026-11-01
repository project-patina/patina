<script lang="ts">
  import { Copy, FolderOpen, SquareTerminal, TriangleAlert } from "@lucide/svelte";
  import Modal from "./Modal.svelte";
  import { api } from "../lib/api";
  import { app, navigate, toast, toastError } from "../lib/state.svelte";

  const crash = $derived(app.crash!);
  const name = $derived(app.instances.find((i) => i.id === crash.id)?.name ?? crash.id);
  const close = () => (app.crash = null);
</script>

<Modal onclose={close} width={680}>
  <div class="head">
    <span class="ico"><TriangleAlert size={22} /></span>
    <div>
      <h3>Gra „{name}” się wysypała</h3>
      <p class="dim">Kod wyjścia {crash.code ?? "—"}. Najczęściej winny jest niepasujący mod — sprawdź ostatnie linie logu.</p>
    </div>
  </div>
  <pre class="tail selectable mono">{crash.tail.join("\n")}</pre>
  <div class="actions">
    <button class="btn" onclick={() => navigator.clipboard?.writeText(crash.tail.join("\n")).then(() => toast("Skopiowano", "success"))}><Copy size={15} /> Kopiuj</button>
    <button class="btn" onclick={() => api.openFolder(crash.id, "crash-reports").catch(() => api.openFolder(crash.id).catch(toastError))}><FolderOpen size={15} /> Raporty</button>
    <div style="flex: 1"></div>
    <button class="btn" onclick={() => { navigate("instance", crash.id, "console"); close(); }}><SquareTerminal size={15} /> Pełna konsola</button>
    <button class="btn primary" onclick={close}>OK</button>
  </div>
</Modal>

<style>
  .head {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }
  .ico {
    width: 42px;
    height: 42px;
    border-radius: 12px;
    display: grid;
    place-items: center;
    background: color-mix(in srgb, var(--danger) 15%, transparent);
    color: var(--danger);
    flex-shrink: 0;
  }
  .head p {
    margin: 4px 0 0;
    line-height: 1.45;
  }
  .tail {
    margin: 16px 0;
    max-height: 300px;
    overflow: auto;
    padding: 12px;
    border-radius: var(--radius-s);
    background: var(--bg-2);
    border: 1px solid var(--border);
    font-size: 0.75rem;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
    user-select: text;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    gap: 8px;
  }
</style>
