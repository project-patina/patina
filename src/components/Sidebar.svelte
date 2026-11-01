<script lang="ts">
  import { Compass, House, LayoutGrid, Newspaper, Settings, Shirt } from "@lucide/svelte";
  import { app, navigate, type Page } from "../lib/state.svelte";

  const items: { page: Page; label: string; icon: typeof House }[] = [
    { page: "home", label: "Start", icon: House },
    { page: "news", label: "NEWS — aktualności", icon: Newspaper },
    { page: "instances", label: "Instancje", icon: LayoutGrid },
    { page: "discover", label: "Mody, shadery, paczki", icon: Compass },
    { page: "wardrobe", label: "Garderoba: skin i peleryny", icon: Shirt },
  ];

  const active = $derived(app.page === "instance" ? "instances" : app.page === "pose" ? "wardrobe" : app.page);
  const runningCount = $derived(Object.values(app.running).filter((s) => s === "running").length);
</script>

<nav class="rail" aria-label="Menu główne">
  <div class="group">
    {#each items as it (it.page)}
      <button class="nav" class:active={active === it.page} onclick={() => navigate(it.page)} aria-label={it.label} aria-current={active === it.page ? "page" : undefined}>
        <it.icon size={19} />
        <span class="tip">{it.label}</span>
        {#if it.page === "instances" && runningCount}<span class="dot"></span>{/if}
      </button>
    {/each}
  </div>

  <div class="group">
    <button class="nav" class:active={active === "settings"} onclick={() => navigate("settings")} aria-label="Ustawienia" aria-current={active === "settings" ? "page" : undefined}>
      <Settings size={19} />
      <span class="tip">Ustawienia</span>
    </button>
  </div>
</nav>

<style>
  .rail {
    width: 64px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0 14px;
  }
  .group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .nav {
    position: relative;
    width: 42px;
    height: 42px;
    display: grid;
    place-items: center;
    border-radius: 12px;
    color: var(--text-3);
    border: 1px solid transparent;
    transition: color 0.2s, background 0.25s var(--ease), border-color 0.25s, transform 0.2s var(--ease);
  }
  .nav:hover {
    color: var(--text);
    background: var(--panel);
  }
  .nav:active {
    transform: scale(0.94);
  }
  .nav.active {
    color: var(--text);
    background: var(--panel-2);
    border-color: var(--border-2);
  }
  .nav.active::before {
    content: "";
    position: absolute;
    left: -11px;
    top: 11px;
    bottom: 11px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--accent);
  }
  .dot {
    position: absolute;
    top: 8px;
    right: 8px;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--success);
    box-shadow: 0 0 0 3px var(--bg);
  }
  .tip {
    position: absolute;
    left: 52px;
    padding: 5px 9px;
    border-radius: 7px;
    background: var(--panel-3);
    border: 1px solid var(--border-2);
    color: var(--text);
    font-size: 0.8rem;
    font-weight: 600;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transform: translateX(-4px);
    transition: opacity 0.15s, transform 0.2s var(--ease);
    z-index: 40;
  }
  .nav:hover .tip,
  .nav:focus-visible .tip {
    opacity: 1;
    transform: none;
  }
</style>
