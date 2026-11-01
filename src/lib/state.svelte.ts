import { api, errText, events } from "./api";
import type { Account, AppInfo, ContentKind, Image, Instance, LogLine, Progress, ReleaseNote, Settings, VersionList } from "./types";
import { createSettingsWriter } from "./settingsWriter";

export type Page = "home" | "instances" | "instance" | "discover" | "wardrobe" | "settings" | "pose" | "news";

export interface Toast {
  id: number;
  kind: "info" | "success" | "error";
  text: string;
}

export interface Crash {
  id: string;
  code: number | null;
  tail: string[];
}

export const app = $state({
  ready: false,
  info: null as AppInfo | null,
  settings: null as Settings | null,
  settingsSave: "saved" as "saved" | "pending" | "error",
  news: null as ReleaseNote[] | null,
  accounts: [] as Account[],
  instances: [] as Instance[],
  versions: null as VersionList | null,
  page: "home" as Page,
  instanceId: null as string | null,
  /** Instance tab to open when navigating to the instance view. */
  instanceTab: "mods" as "mods" | "settings" | "console",
  /** Per-instance: "preparing" | "running". */
  running: {} as Record<string, "preparing" | "running">,
  progress: {} as Record<string, Progress>,
  logs: {} as Record<string, LogLine[]>,
  toasts: [] as Toast[],
  crash: null as Crash | null,
  newInstanceOpen: false,
  skins: {} as Record<string, string>,
  wallpapers: null as Image[] | null,
  /** Discover page: what to browse and which instance it installs into. */
  discover: { kind: "mod" as ContentKind, instanceId: null as string | null },
  /** Open project page (modal) from anywhere. */
  project: null as null | { id: string; kind: ContentKind; tab?: "about" | "gallery" | "versions" },
  /** Bumped after installs so open content lists reload. */
  contentRev: 0,
});

export function openDiscover(kind: ContentKind = "mod", instanceId: string | null = null) {
  app.discover.kind = kind;
  if (instanceId) app.discover.instanceId = instanceId;
  app.page = "discover";
}

let toastId = 0;
export function toast(text: string, kind: Toast["kind"] = "info", ms = 4200) {
  const id = ++toastId;
  app.toasts.push({ id, kind, text });
  setTimeout(() => dismissToast(id), ms);
}
export function dismissToast(id: number) {
  const i = app.toasts.findIndex((t) => t.id === id);
  if (i >= 0) app.toasts.splice(i, 1);
}
export function toastError(e: unknown) {
  toast(errText(e), "error", 7000);
}

export function activeAccount(): Account | null {
  return app.accounts.find((a) => a.active) ?? null;
}

export function navigate(page: Page, instanceId: string | null = null, tab?: typeof app.instanceTab) {
  app.page = page;
  if (instanceId) app.instanceId = instanceId;
  if (tab) app.instanceTab = tab;
}

// ---------------------------------------------------------------------------
// Theme

const media = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: light)") : null;
const reducedMotion = typeof window !== "undefined" ? window.matchMedia("(prefers-reduced-motion: reduce)") : null;

export function applyTheme(s: Settings | null = app.settings) {
  if (!s) return;
  const root = document.documentElement;
  const light = s.theme === "light" || (s.theme === "system" && media?.matches);
  root.dataset.theme = light ? "light" : "dark";
  root.dataset.palette = s.palette;
  root.style.setProperty("--accent", s.accent);
  const rgb = [1, 3, 5].map((offset) => parseInt(s.accent.slice(offset, offset + 2), 16) / 255)
    .map((c) => c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const luminance = rgb[0] * 0.2126 + rgb[1] * 0.7152 + rgb[2] * 0.0722;
  root.style.setProperty("--accent-fg", luminance > 0.179 ? "#09090b" : "#ffffff");
  root.style.setProperty("--ui-scale", String(s.uiScale));
  root.classList.toggle("no-anim", !s.animations);
  root.classList.toggle("no-blur", !s.blur);
  root.classList.toggle("compact", s.compact);
  root.dataset.radius = s.radius;
  root.dataset.font = s.font;
}
media?.addEventListener("change", () => applyTheme());

/** Transition duration helper: zero when animations are off. */
export function dur(ms: number): number {
  return app.settings?.animations === false || reducedMotion?.matches ? 0 : ms;
}

const settingsWriter = createSettingsWriter<Settings>({
  save: (settings) => api.saveSettings(settings),
  onSaved(saved, submitted) {
    if (!app.settings) return;
    // Adopt backend normalization only if this field has not been edited since.
    for (const key of Object.keys(saved) as (keyof Settings)[]) {
      if (JSON.stringify(app.settings[key]) === JSON.stringify(submitted[key])) {
        Object.assign(app.settings, { [key]: saved[key] });
      }
    }
    app.settingsSave = settingsWriter.hasPending() ? "pending" : "saved";
    applyTheme();
  },
  onError(error) {
    app.settingsSave = "error";
    toastError(error);
  },
});

export const flushSettings = () => settingsWriter.flush();

/** Patch settings locally right away, persist debounced. */
export function updateSettings(patch: Partial<Settings>) {
  if (!app.settings) return;
  Object.assign(app.settings, patch);
  applyTheme();
  app.settingsSave = "pending";
  settingsWriter.schedule($state.snapshot(app.settings) as Settings);
}

// ---------------------------------------------------------------------------
// Data

export async function refreshInstances() {
  app.instances = await api.listInstances();
}

export async function refreshAccounts() {
  app.accounts = await api.listAccounts();
  for (const a of app.accounts) void loadSkin(a);
}

export async function loadSkin(a: Account) {
  if (!a.skinUrl || app.skins[a.skinUrl]) return;
  try {
    app.skins[a.skinUrl] = await api.skinDataUrl(a.skinUrl);
  } catch {
    /* no skin is fine */
  }
}

export async function loadVersions() {
  if (app.versions) return app.versions;
  app.versions = await api.listVersions();
  return app.versions;
}

export async function loadNews() {
  if (app.news) return app.news;
  app.news = await api.listNews();
  return app.news;
}

export async function init() {
  const [info, settings] = await Promise.all([api.appInfo(), api.getSettings()]);
  app.info = info;
  app.settings = settings;
  applyTheme(settings);
  await Promise.all([refreshAccounts(), refreshInstances()]);
  const unlisten: (() => void)[] = [];
  const observed = new Set<string>();
  try {
    unlisten.push(await events.progress((p) => {
      app.progress[p.task] = p;
    }));
    unlisten.push(await events.gameLog((l) => {
      const list = (app.logs[l.id] ??= []);
      list.push(l);
      if (list.length > 3000) list.splice(0, list.length - 3000);
    }));
    unlisten.push(await events.gameState((s) => {
      observed.add(s.id);
      const inst = app.instances.find((i) => i.id === s.id);
      const name = inst?.name ?? s.id;
      switch (s.state) {
        case "preparing":
          app.running[s.id] = "preparing";
          app.logs[s.id] = [];
          break;
        case "running":
          app.running[s.id] = "running";
          delete app.progress[s.id];
          if (app.settings?.openConsoleOnLaunch) navigate("instance", s.id, "console");
          break;
        case "crashed":
          app.crash = { id: s.id, code: s.code, tail: (app.logs[s.id] ?? []).slice(-60).map((l) => l.line) };
        // fallthrough
        case "exited":
        case "cancelled":
          delete app.running[s.id];
          delete app.progress[s.id];
          if (s.state === "cancelled") toast(`Anulowano przygotowanie „${name}”`);
          void refreshInstances().catch(toastError);
          break;
        case "error":
          delete app.running[s.id];
          delete app.progress[s.id];
          toast(s.message ?? "Nie udało się uruchomić gry", "error", 9000);
          break;
      }
    }));
    // Events observed while the snapshot is being read take precedence.
    for (const r of await api.running()) if (!observed.has(r.id)) app.running[r.id] = r.state;
  } catch (error) {
    unlisten.forEach((stop) => stop());
    throw error;
  }
  if (info.startPage && ["home", "instances", "discover", "wardrobe", "settings", "pose", "news"].includes(info.startPage)) app.page = info.startPage as Page;
  app.ready = true;
  return () => unlisten.forEach((stop) => stop());
}

const pendingLaunches = new Set<string>();

export async function play(id: string) {
  if (app.running[id] || pendingLaunches.has(id)) return;
  if (!activeAccount()) {
    toast("Najpierw zaloguj się kontem Microsoft", "error");
    return;
  }
  pendingLaunches.add(id);
  try {
    await flushSettings();
    app.running[id] = "preparing";
    await api.launch(id);
    if (app.settings) app.settings.lastInstance = id;
  } catch (e) {
    delete app.running[id];
    toastError(e);
  } finally {
    pendingLaunches.delete(id);
  }
}

export async function stopOrCancel(id: string) {
  try {
    if (app.running[id] === "running") await api.stop(id);
    else await api.cancelInstall(id);
  } catch (error) { toastError(error); }
}
