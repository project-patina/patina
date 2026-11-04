// Dev-only fake backend so the UI can be clicked through in a plain browser
// (`npm run dev`, no Tauri). Never bundled into release builds.
import { emit } from "@tauri-apps/api/event";
import { mockConvertFileSrc, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Account, ContentItem, Instance, OfficialMod, Settings } from "./types";
import devSkin from "./demo/skin.svg?url";
import devCape from "./demo/cape.svg?url";
import devCape2 from "./demo/cape-iris.svg?url";

async function dataUrl(url: string): Promise<string> {
  const b = await (await fetch(url)).blob();
  return new Promise((r) => {
    const f = new FileReader();
    f.onload = () => r(f.result as string);
    f.readAsDataURL(b);
  });
}

const now = Math.floor(Date.now() / 1000);
const params = new URLSearchParams(location.search);

let settings: Settings = {
  theme: (params.get("theme") as Settings["theme"]) ?? "dark", palette: "patina", accent: "#3fb8a0", animations: !params.has("noanim"), uiScale: 1,
  font: "inter", radius: "normal", compact: false, heroWallpaper: params.get("hero") ?? "", appWallpaper: "",
  followCursor: true, showNametag: true,
  showSnapshots: false, showOldVersions: false, defaultLoader: "fabric", installPerformanceMods: true, memoryMinMb: 1024,
  memoryMaxMb: 6144, javaPath: "", jvmArgs: "", optimizedJvmArgs: true, gameWidth: 1280, gameHeight: 720, fullscreen: false,
  onLaunch: "keep", reopenOnExit: true, openConsoleOnLaunch: false, maxConcurrentDownloads: 16, discordPresence: true, discordShowServer: true, gamemode: false, mangohud: false,
  discreteGpu: false, envVars: "", wrapperCommand: "", preLaunchCommand: "", postExitCommand: "",
  rustEngine: { lodDistance: null, resolutionScale: null, antialiasing: "game", graphicsApi: "game", hdr: null, batteryMode: null, fpsOverlay: null, xwayland: true },
  language: params.get("lang") ?? "", brandVersion: 1,
  onboarded: !params.has("onboarding"), lastInstance: "fabric-1-21-1",
};

let accounts: Account[] = params.has("noaccount") ? [] : [{ uuid: "00000000000040008000000000000001", name: "Patina", skinUrl: "https://example.invalid/patina/dev-skin", skinSlim: false, capeUrl: "https://example.invalid/patina/dev-cape", active: true }];

const inst = (id: string, name: string, mc: string, loader: Instance["loader"], icon: string, banner: string, last: number | null, play: number): Instance => ({
  id, name, mcVersion: mc, loader, loaderVersion: loader === "vanilla" ? null : "0.20.1", icon, banner, createdAt: now - 86400 * 5,
  lastPlayed: last, playTimeSecs: play, overrides: {}, installed: true,
});
let instances: Instance[] = [
  inst("fabric-1-21-1", "Fabric 1.21.1", "1.21.1", "fabric", "diamond", "aurora", now - 3600, 5 * 3600 + 1200),
  inst("survival", "Survival z kumplami", "26.3", "fabric", "grass", "forest", now - 86400 * 2, 12 * 3600),
  inst("pvp", "PvP Practice", "1.8.9", "vanilla", "sword", "nether", now - 86400 * 9, 3600),
  inst("quilt-test", "Quilt test", "1.20.1", "quilt", "rocket", "ocean", null, 0),
  { ...inst("rust-26-3", "Rust 26.3", "26.3", "rust", "zap", "end", null, 0), loaderVersion: null },
];

/** ?official=list offers mods, ?official=offline stands for no network or no repository yet. */
const officialList = (): OfficialMod[] => params.get("official") !== "list" ? [] : [
  { id: "minimap", name: "Minimapa", version: "1.0.0", summary: { pl: "Mapa okolicy w rogu ekranu", en: "A map of the area in the screen corner" }, icon: null, size: 2_400_000, state: "available" },
  { id: "zoom", name: "Zoom", version: "1.2.0", summary: { pl: "Przybliżanie pod klawiszem", en: "Zoom on a key" }, icon: null, size: 310_000, state: "installed" },
  { id: "waypoints", name: "Punkty trasy", version: "0.4.0", summary: { pl: "Zapisane miejsca na świecie", en: "Saved places in the world" }, icon: null, size: 880_000, state: "outdated" },
];
let official = officialList();

const meta = (title: string, v: string, icon: string) => ({ kind: "mod" as const, projectId: title.toLowerCase(), versionId: v, title, iconUrl: icon, versionNumber: v });
let mods: ContentItem[] = [
  { file: "sodium-fabric-0.6.13+mc1.21.1.jar", enabled: true, size: 1_200_000, meta: meta("Sodium", "mc1.21.1-0.6.13", "https://cdn.modrinth.com/data/AANobbMI/295862f4724dc3f78df3447ad6072b2dcd3ef0c9_96.webp") },
  { file: "lithium-fabric-0.15.1+mc1.21.1.jar", enabled: true, size: 780_000, meta: meta("Lithium", "mc1.21.1-0.15.1", "https://cdn.modrinth.com/data/gvQqBUqZ/d6a7eefe4c7e1b6c4b0a6f36a95b4ea8b9a2c8a6_96.webp") },
  { file: "ferritecore-7.0.2-fabric.jar", enabled: false, size: 120_000, meta: null },
];
const hit = (id: string, title: string, desc: string, dl: number) => ({
  projectId: id, slug: id, title, description: desc, author: "dev", iconUrl: null, downloads: dl, follows: Math.round(dl / 50),
  categories: ["optimization"], displayCategories: ["optimization", "fabric"], projectType: "mod", dateModified: new Date(Date.now() - 86400e3 * 3).toISOString(), featuredGallery: null, color: null,
});

const versions = ["26.3", "26.2", "26.1.2", "26.1.1", "26.1", "1.21.11", "1.21.8", "1.21.4", "1.21.1", "1.21", "1.20.6", "1.20.4", "1.20.1", "1.19.4", "1.18.2", "1.16.5", "1.12.2", "1.8.9"];

function fakeLaunch(id: string) {
  const steps = 40;
  let i = 0;
  const t = setInterval(() => {
    i++;
    emit("progress", { task: id, stage: "assets", done: i, total: steps, message: `Downloading game assets (${i * 98}/${steps * 98})` });
    if (i >= steps) {
      clearInterval(t);
      emit("game-state", { id, state: "running", message: null, code: null });
      for (let n = 0; n < 30; n++) emit("game-log", { id, line: `[21:3${n % 10}:00] [Render thread/INFO]: Example log line ${n}`, error: n % 11 === 0 });
      // ?crash ends the fake game with a crash, to show the crash dialog.
      if (params.has("crash")) emit("game-state", { id, state: "crashed", message: null, code: 1 });
    }
  }, 60);
  setTimeout(() => emit("game-state", { id, state: "preparing", message: null, code: null }), 0);
}

function announcement(language: "pl" | "en") {
  const pl = language === "pl";
  return {
    id: "first-version", date: "2026-11-01", image: "/announcements/first-version.png",
    title: pl ? "Patina: pierwsza wersja" : "Patina: the first version",
    summary: "",
    markdown: pl
      ? "# Pierwsza wersja\n\nPatina jest już dostępna. Wiadomości i aktualizacje przychodzą teraz prosto do launchera.\n\n- Instancje Fabric\n- Silnik Rust dla 26.3\n- [Kod źródłowy](https://github.com/project-patina/patina)"
      : "# The first version\n\nPatina is out. News and updates now arrive right in the launcher.\n\n- Fabric instances\n- Rust engine for 26.3\n- [Source code](https://github.com/project-patina/patina)",
  };
}

/** ?update=normal|critical offers a release; add &uninstallable for a dev build's prompt, &updatefail to fail it. */
function pendingUpdate() {
  const kind = params.get("update");
  if (kind !== "normal" && kind !== "critical") return null;
  return {
    version: "0.3.0", current: "0.2.0-dev", critical: kind === "critical", date: "2026-11-01", sizeBytes: 24_000_000,
    installable: !params.has("uninstallable"),
    notes: { pl: "Nowy wygląd ustawień i wybór języka.\nPoprawki stabilności.", en: "Redesigned settings and a language choice.\nStability fixes." },
  };
}

async function fakeInstall() {
  const total = 24_000_000;
  for (let i = 1; i <= 60; i++) {
    await new Promise((r) => setTimeout(r, 100));
    emit("update-progress", { stage: "download", done: (total * i) / 60, total });
    if (params.has("updatefail") && i === 30) throw "Connection lost while downloading the update";
  }
  emit("update-progress", { stage: "verify", done: 0, total: 0 });
  await new Promise((r) => setTimeout(r, 900));
  emit("update-progress", { stage: "install", done: 0, total: 0 });
  await new Promise((r) => setTimeout(r, 900));
}

/** ?gear draws plain stand-ins, so clipping can be checked without the game's textures. */
function fakeTexture(size: number, colour: string): string {
  const canvas = document.createElement("canvas");
  canvas.width = canvas.height = size;
  const context = canvas.getContext("2d")!;
  context.fillStyle = colour;
  if (size === 16) for (let i = 0; i < 12; i++) context.fillRect(3 + i, 12 - i, 2, 2);
  else context.fillRect(1, 1, 12, 22);
  return canvas.toDataURL();
}

export function installDevMock() {
  (window as { __devmock?: boolean }).__devmock = true;
  mockWindows("main");
  mockConvertFileSrc("linux");
  mockIPC(
    async (cmd, args: any) => {
      switch (cmd) {
        // ?holdboot keeps the opening on screen, to look at it.
        case "app_info": if (params.has("holdboot")) await new Promise(() => {}); return { version: "0.2.0-dev", dataDir: "/home/user/.local/share/amethyst", totalMemoryMb: 31800, os: "linux", arch: "x86_64", startPage: params.get("page") };
        case "list_announcements": return { items: [announcement(args.language)], source: "bundled" };
        case "set_language": return null;
        case "update_check": return pendingUpdate();
        case "update_install": await fakeInstall(); return null;
        case "update_restart": location.href = location.pathname; return null;
        case "get_settings": return settings;
        case "save_settings": settings = args.settings; return settings;
        case "list_versions": return { latestRelease: "26.3", latestSnapshot: "26.4-snapshot-6", versions: versions.map((id, i) => ({ id, kind: "release", releaseTime: new Date(Date.now() - i * 40 * 86400e3).toISOString() })) };
        case "loader_versions": return args.mc === "1.8.9" ? [] : [{ version: "0.20.1", stable: true }, { version: "0.20.0", stable: true }, { version: "0.19.5", stable: true }];
        case "list_instances": return instances;
        case "get_instance": return instances.find((i) => i.id === args.id);
        case "create_instance": {
          const i = inst(args.req.name.toLowerCase().replace(/[^a-z0-9]+/g, "-"), args.req.name, args.req.mcVersion, args.req.loader, args.req.icon, args.req.banner, null, 0);
          instances = [i, ...instances];
          return i;
        }
        case "update_instance": instances = instances.map((i) => (i.id === args.instance.id ? args.instance : i)); return args.instance;
        case "delete_instance": instances = instances.filter((i) => i.id !== args.id); return null;
        case "list_content":
          if (args.kind !== "mod") return [];
          return instances.find((i) => i.id === args.id)?.mcVersion === "26.3" ? [{ file: "patina-client.jar", enabled: true, size: 243_588, meta: null }, ...mods] : mods;
        case "official_mods": return { available: params.get("official") !== "offline", mods: official, local: [] };
        case "install_official_mod": await new Promise((r) => setTimeout(r, 500)); official = official.map((m) => (m.id === args.modId ? { ...m, state: "installed" } : m)); return null;
        case "remove_official_mod": official = official.map((m) => (m.id === args.modId ? { ...m, state: "available" } : m)); return null;
        case "delete_content": mods = mods.filter((m) => m.file !== args.file); return null;
        case "check_content_updates": return [{ file: mods[0]?.file, projectId: "sodium", versionId: "new", versionNumber: "mc1.21.1-0.6.14" }];
        case "import_content": return args.paths;
        case "modrinth_search":
          return { hits: [hit("sodium", "Sodium", "The fastest rendering engine for Minecraft", 61_000_000), hit("iris", "Iris Shaders", "OptiFine-compatible shaders on Fabric", 40_000_000), hit("xaeros-minimap", "Xaero's Minimap", "Minimap with waypoints", 18_000_000), hit("modmenu", "Mod Menu", "A list of mods in the game menu", 30_000_000)].filter((h) => h.title.toLowerCase().includes((args.query.query ?? "").toLowerCase())), offset: 0, limit: 20, totalHits: 4 };
        case "modrinth_project":
          return { id: args.id, slug: args.id, title: args.id === "iris" ? "Iris Shaders" : "Sodium", description: "Sample project description", body: "## Features\n\n- **Fast** rendering\n- Works with [Fabric](https://fabricmc.net)\n\n`code` and an image:", iconUrl: null, downloads: 61_000_000, followers: 40_000, categories: ["optimization", "fabric"], projectType: "mod", gallery: [], sourceUrl: "https://github.com/x/y", issuesUrl: null, wikiUrl: null, discordUrl: "https://discord.gg/x", license: { id: "LGPL-3.0", name: null }, clientSide: "required", serverSide: "unsupported", gameVersions: ["1.21.1"], loaders: ["fabric"], updated: null };
        case "modrinth_versions":
          return [0, 1, 2, 3].map((n) => ({ id: `v${n}`, projectId: args.id, name: `Sodium 0.6.${13 - n}`, versionNumber: `mc1.21.1-0.6.${13 - n}`, versionType: n === 1 ? "beta" : "release", gameVersions: n === 3 ? ["1.20.1"] : ["1.21", "1.21.1"], loaders: ["fabric"], datePublished: new Date(Date.now() - n * 30 * 86400e3).toISOString(), downloads: 1_000_000 / (n + 1), files: [], dependencies: [], changelog: null }));
        case "install_content": await new Promise((r) => setTimeout(r, 500)); return [{ title: "Iris Shaders", file: "iris.jar", dependency: false }, { title: "Sodium", file: "sodium.jar", dependency: true }];
        case "list_images": return [];
        // The source preview never bundles textures extracted from the game.
        case "gear_textures": return params.has("gear") ? { sword: fakeTexture(16, "#c8d0d4"), shield: fakeTexture(64, "#8a5a33") } : null;
        case "import_image": throw "Picking files only works in the app";
        case "get_profile":
          return { id: "00000000000040008000000000000001", name: "Patina", skins: [], capes: [{ id: "c1", state: "ACTIVE", url: "https://example.invalid/patina/dev-cape", variant: null, alias: "Patina" }, { id: "c2", state: "INACTIVE", url: "https://example.invalid/patina/dev-cape2", variant: null, alias: "Iris" }] };
        case "set_cape": case "upload_skin": case "set_skin_variant": return accounts;
        case "set_content_enabled": case "open_folder": case "stop": case "cancel_install": case "cancel_login": return null;
        case "install_performance_mods": return ["Sodium", "Lithium"];
        case "detect_java": return [{ path: "/usr/lib/jvm/java-21-openjdk/bin/java", version: "21.0.5", major: 21, source: "system" }];
        case "list_accounts": return accounts;
        case "set_active_account": accounts = accounts.map((a) => ({ ...a, active: a.uuid === args.uuid })); return accounts;
        case "remove_account": accounts = accounts.filter((a) => a.uuid !== args.uuid); if (accounts[0]) accounts[0].active = true; return accounts;
        case "skin_data_url": return dataUrl(args.url.endsWith("dev-skin") ? devSkin : args.url.endsWith("dev-cape2") ? devCape2 : devCape);
        case "login_microsoft":
        case "login_device_finish":
          await new Promise((r) => setTimeout(r, 1200));
          accounts = [{ uuid: "00000000000040008000000000000001", name: "Patina", skinUrl: "https://example.invalid/patina/dev-skin", skinSlim: false, capeUrl: null, active: true }];
          return accounts[0];
        case "login_device_start": return { userCode: "ABCD-1234", deviceCode: "x", verificationUri: "https://www.microsoft.com/link", interval: 5, expiresIn: 900 };
        case "launch": fakeLaunch(args.id); return null;
        case "servers_list": return [
          { id: "hypixel", name: "Hypixel", address: "mc.hypixel.net", region: "na", tags: ["minigames", "skyblock", "bedwars"] },
          { id: "cubecraft", name: "CubeCraft", address: "play.cubecraft.net", region: "eu", tags: ["minigames", "skyblock"] },
          { id: "earthmc", name: "EarthMC", address: "play.earthmc.net", region: "eu", tags: ["towny", "survival"] },
          { id: "2b2t", name: "2b2t", address: "2b2t.org", region: "na", tags: ["anarchy", "survival"] },
        ];
        case "server_ping": {
          // Fake answers: the anarchy server is down, the rest answer after a moment.
          await new Promise((r) => setTimeout(r, 300 + Math.random() * 700));
          if (args.address === "2b2t.org") return { online: false, players: null, maxPlayers: null, version: null, motd: "", latencyMs: null, favicon: null };
          const players = args.address.length * 1371 % 50000;
          return { online: true, players, maxPlayers: 100000, version: "1.8-26.3", motd: `${args.address} · Welcome!`, latencyMs: 20 + args.address.length * 3, favicon: null };
        }
        case "running": return [];
        case "game_log": return [];
        case "presence_page": return null;
        default:
          if (cmd.startsWith("plugin:event|")) return null;
          console.warn("devmock: unhandled", cmd, args);
          return null;
      }
    },
    { shouldMockEvents: true },
  );
}
