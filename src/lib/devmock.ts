// Dev-only fake backend so the UI can be clicked through in a plain browser
// (`npm run dev`, no Tauri). Never bundled into release builds.
import { emit } from "@tauri-apps/api/event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { Account, ContentItem, Instance, Settings } from "./types";
import devSkin from "./demo/skin.svg?url";
import devCape from "./demo/cape.svg?url";
import devCape2 from "./demo/cape-iris.svg?url";
import news from "../../resources/news.json";

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
  theme: (params.get("theme") as Settings["theme"]) ?? "dark", palette: "patina", accent: "#3fb8a0", animations: !params.has("noanim"), blur: true, uiScale: 1,
  font: "inter", radius: "normal", compact: false, heroWallpaper: params.get("hero") ?? "", heroScene: true, heroDim: 25, appWallpaper: "", appWallpaperDim: 75,
  modelPosition: "center", followCursor: true, showNametag: true,
  showSnapshots: false, showOldVersions: false, defaultLoader: "fabric", installPerformanceMods: true, memoryMinMb: 1024,
  memoryMaxMb: 6144, javaPath: "", jvmArgs: "", optimizedJvmArgs: true, gameWidth: 1280, gameHeight: 720, fullscreen: false,
  onLaunch: "keep", reopenOnExit: true, openConsoleOnLaunch: false, maxConcurrentDownloads: 16, gamemode: false, mangohud: false,
  discreteGpu: false, envVars: "", wrapperCommand: "", preLaunchCommand: "", postExitCommand: "",
  rustEngine: { lodDistance: null, resolutionScale: null, antialiasing: "game", graphicsApi: "game", hdr: null, batteryMode: null, fpsOverlay: null, xwayland: true },
  brandVersion: 1,
  modelPose: localStorage.getItem("dev-pose") ?? "",
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
];

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
    emit("progress", { task: id, stage: "assets", done: i, total: steps, message: `Pobieranie zasobów gry (${i * 98}/${steps * 98})` });
    if (i >= steps) {
      clearInterval(t);
      emit("game-state", { id, state: "running", message: null, code: null });
      for (let n = 0; n < 30; n++) emit("game-log", { id, line: `[21:3${n % 10}:00] [Render thread/INFO]: Example log line ${n}`, error: n % 11 === 0 });
    }
  }, 60);
  setTimeout(() => emit("game-state", { id, state: "preparing", message: null, code: null }), 0);
}

export function installDevMock() {
  mockWindows("main");
  mockIPC(
    async (cmd, args: any) => {
      switch (cmd) {
        // ?holdboot keeps the opening on screen, to look at it.
        case "app_info": if (params.has("holdboot")) await new Promise(() => {}); return { version: "0.2.0-dev", dataDir: "/home/user/.local/share/amethyst", totalMemoryMb: 31800, os: "linux", arch: "x86_64", startPage: params.get("page") };
        case "list_news": return news;
        case "get_settings": return settings;
        case "save_settings": settings = args.settings; localStorage.setItem("dev-pose", settings.modelPose); return settings;
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
        case "list_content": return args.kind === "mod" ? mods : [];
        case "delete_content": mods = mods.filter((m) => m.file !== args.file); return null;
        case "check_content_updates": return [{ file: mods[0]?.file, projectId: "sodium", versionId: "new", versionNumber: "mc1.21.1-0.6.14" }];
        case "import_content": return args.paths;
        case "modrinth_search":
          return { hits: [hit("sodium", "Sodium", "Najszybszy silnik renderowania dla Minecrafta", 61_000_000), hit("iris", "Iris Shaders", "Shadery kompatybilne z OptiFine, na Fabric", 40_000_000), hit("xaeros-minimap", "Xaero's Minimap", "Minimapa z waypointami", 18_000_000), hit("modmenu", "Mod Menu", "Lista modów w menu gry", 30_000_000)].filter((h) => h.title.toLowerCase().includes((args.query.query ?? "").toLowerCase())), offset: 0, limit: 20, totalHits: 4 };
        case "modrinth_project":
          return { id: args.id, slug: args.id, title: args.id === "iris" ? "Iris Shaders" : "Sodium", description: "Opis testowy projektu", body: "## Funkcje\n\n- **Szybkie** renderowanie\n- Działa z [Fabric](https://fabricmc.net)\n\n`kod` i obrazek:", iconUrl: null, downloads: 61_000_000, followers: 40_000, categories: ["optimization", "fabric"], projectType: "mod", gallery: [], sourceUrl: "https://github.com/x/y", issuesUrl: null, wikiUrl: null, discordUrl: "https://discord.gg/x", license: { id: "LGPL-3.0", name: null }, clientSide: "required", serverSide: "unsupported", gameVersions: ["1.21.1"], loaders: ["fabric"], updated: null };
        case "modrinth_versions":
          return [0, 1, 2, 3].map((n) => ({ id: `v${n}`, projectId: args.id, name: `Sodium 0.6.${13 - n}`, versionNumber: `mc1.21.1-0.6.${13 - n}`, versionType: n === 1 ? "beta" : "release", gameVersions: n === 3 ? ["1.20.1"] : ["1.21", "1.21.1"], loaders: ["fabric"], datePublished: new Date(Date.now() - n * 30 * 86400e3).toISOString(), downloads: 1_000_000 / (n + 1), files: [], dependencies: [], changelog: null }));
        case "install_content": await new Promise((r) => setTimeout(r, 500)); return [{ title: "Iris Shaders", file: "iris.jar", dependency: false }, { title: "Sodium", file: "sodium.jar", dependency: true }];
        case "list_images": return [];
        case "backdrop_frames": return 0;
        // The source preview never bundles textures extracted from the game.
        case "gear_textures": return null;
        case "import_image": throw "Wybór plików działa tylko w aplikacji";
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
        case "running": return [];
        case "game_log": return [];
        default:
          if (cmd.startsWith("plugin:event|")) return null;
          console.warn("devmock: unhandled", cmd, args);
          return null;
      }
    },
    { shouldMockEvents: true },
  );
}
