<script lang="ts">
  import { Compass, House, LayoutGrid, Server, Settings, Shirt } from "@lucide/svelte";
  import { t } from "../lib/i18n.svelte";
  import { indicator } from "../lib/motion";
  import { app, navigate, type Page } from "../lib/state.svelte";

  const items = $derived<{ page: Page; label: string; icon: typeof House }[]>([
    { page: "home", label: t("nav.home"), icon: House },
    { page: "instances", label: t("nav.instances"), icon: LayoutGrid },
    { page: "discover", label: t("nav.discover"), icon: Compass },
    { page: "servers", label: t("nav.servers"), icon: Server },
    { page: "wardrobe", label: t("nav.wardrobe"), icon: Shirt },
  ]);

  const active = $derived(app.page === "instance" ? "instances" : app.page);
  const running = $derived(Object.values(app.running).some((s) => s === "running"));
</script>

<nav class="side" aria-label={t("nav.main")} use:indicator={active}>
  <span class="nav-indicator" data-indicator></span>
  {#each items as it (it.page)}
    <button class="nav-item" onclick={() => navigate(it.page)} title={it.label} aria-current={active === it.page ? "page" : undefined}>
      <it.icon size={16} />
      <span class="text">{it.label}</span>
      {#if it.page === "instances" && running}<span class="dot"></span>{/if}
    </button>
  {/each}
  <button class="nav-item last" onclick={() => navigate("settings")} title={t("nav.settings")} aria-current={active === "settings" ? "page" : undefined}>
    <Settings size={16} />
    <span class="text">{t("nav.settings")}</span>
  </button>
</nav>

<style>
  .side {
    position: relative;
    width: 200px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    border-right: 1px solid var(--border);
  }
  .last { margin-top: auto; }
  .dot {
    margin-left: auto;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--success);
  }
  @media (max-width: 979px) {
    .side { width: 56px; padding: 12px 8px; }
    .side > .nav-item { justify-content: center; padding: 0; }
    .text { display: none; }
    .dot { position: absolute; top: 6px; right: 6px; margin: 0; }
  }
</style>
