import "@fontsource-variable/inter";
import "@fontsource-variable/outfit";
import "@fontsource-variable/jetbrains-mono";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

// The window chrome is ours; block the browser context menu outside inputs.
window.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement;
  if (!t.closest("input, textarea, .selectable")) e.preventDefault();
});

async function start() {
  // Plain browser during development: swap in a fake backend.
  if (import.meta.env.DEV && !(window as any).__TAURI_INTERNALS__?.invoke) {
    const { installDevMock } = await import("./lib/devmock");
    installDevMock();
  }
  mount(App, { target: document.getElementById("app")! });
}

void start();
