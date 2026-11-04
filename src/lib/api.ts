import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ContentItem, ContentKind, ContentUpdate, Image, Installed, McProfile, ModVersion, Project, SearchQuery, SearchResult,
  Account, AppInfo, DeviceCode, GameStateEvent, GearTextures, Instance, JavaInstall, Loader, LoaderVersion,
  LogLine, NewInstance, OfficialCatalog, Progress, AnnouncementFeed, RunningInfo, Settings, UpdateInfo, UpdateProgress, VersionList, ServerEntry, ServerStatus,
} from "./types";
import type { Lang } from "./locales/resolve";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  announcements: (language: Lang) => invoke<AnnouncementFeed>("list_announcements", { language }),
  setLanguage: (language: Lang) => invoke<void>("set_language", { language }),
  updateCheck: () => invoke<UpdateInfo | null>("update_check"),
  updateInstall: () => invoke<void>("update_install"),
  updateRestart: () => invoke<void>("update_restart"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),

  listVersions: () => invoke<VersionList>("list_versions"),
  loaderVersions: (loader: Loader, mc: string) => invoke<LoaderVersion[]>("loader_versions", { loader, mc }),

  listInstances: () => invoke<Instance[]>("list_instances"),
  getInstance: (id: string) => invoke<Instance>("get_instance", { id }),
  createInstance: (req: NewInstance) => invoke<Instance>("create_instance", { req }),
  updateInstance: (instance: Instance) => invoke<Instance>("update_instance", { instance }),
  deleteInstance: (id: string) => invoke<void>("delete_instance", { id }),

  listContent: (id: string, kind: ContentKind, identify = false) => invoke<ContentItem[]>("list_content", { id, kind, identify }),
  setContentEnabled: (id: string, kind: ContentKind, file: string, enabled: boolean) => invoke<void>("set_content_enabled", { id, kind, file, enabled }),
  deleteContent: (id: string, kind: ContentKind, file: string) => invoke<void>("delete_content", { id, kind, file }),
  importContent: (id: string, kind: ContentKind, paths: string[]) => invoke<string[]>("import_content", { id, kind, paths }),
  checkUpdates: (id: string, kind: ContentKind) => invoke<ContentUpdate[]>("check_content_updates", { id, kind }),
  officialMods: (id: string) => invoke<OfficialCatalog>("official_mods", { id }),
  installOfficialMod: (id: string, modId: string) => invoke<void>("install_official_mod", { id, modId }),
  removeOfficialMod: (id: string, modId: string) => invoke<void>("remove_official_mod", { id, modId }),
  search: (query: SearchQuery) => invoke<SearchResult>("modrinth_search", { query }),
  project: (id: string) => invoke<Project>("modrinth_project", { id }),
  versions: (id: string, kind: ContentKind, instance: string | null, allVersions: boolean) =>
    invoke<ModVersion[]>("modrinth_versions", { id, kind, instance, allVersions }),
  installContent: (instance: string, kind: ContentKind, project: string, version: string | null = null) =>
    invoke<Installed[]>("install_content", { instance, kind, project, version }),

  importImage: (kind: "wallpaper" | "skin", path: string) => invoke<Image>("import_image", { kind, path }),
  gearTextures: () => invoke<GearTextures | null>("gear_textures"),
  listImages: (kind: "wallpaper" | "skin") => invoke<Image[]>("list_images", { kind }),
  deleteImage: (kind: "wallpaper" | "skin", id: string) => invoke<void>("delete_image", { kind, id }),
  getProfile: (uuid: string) => invoke<McProfile>("get_profile", { uuid }),
  setCape: (uuid: string, cape: string | null) => invoke<Account[]>("set_cape", { uuid, cape }),
  uploadSkin: (uuid: string, skin: string | null, path: string | null, slim: boolean) => invoke<Account[]>("upload_skin", { uuid, skin, path, slim }),
  setSkinVariant: (uuid: string, slim: boolean) => invoke<Account[]>("set_skin_variant", { uuid, slim }),
  installPerformanceMods: (id: string) => invoke<string[]>("install_performance_mods", { id }),
  openFolder: (id?: string, sub?: string) => invoke<void>("open_folder", { id: id ?? null, sub: sub ?? null }),
  detectJava: () => invoke<JavaInstall[]>("detect_java"),

  listAccounts: () => invoke<Account[]>("list_accounts"),
  setActiveAccount: (uuid: string) => invoke<Account[]>("set_active_account", { uuid }),
  removeAccount: (uuid: string) => invoke<Account[]>("remove_account", { uuid }),
  skinDataUrl: (url: string) => invoke<string>("skin_data_url", { url }),
  loginMicrosoft: () => invoke<Account>("login_microsoft"),
  loginDeviceStart: () => invoke<DeviceCode>("login_device_start"),
  loginDeviceFinish: (code: DeviceCode) => invoke<Account>("login_device_finish", { code }),
  cancelLogin: () => invoke<void>("cancel_login"),

  launch: (id: string, join?: string) => invoke<void>("launch", { id, join: join ?? null }),
  serversList: () => invoke<ServerEntry[]>("servers_list"),
  serverPing: (address: string) => invoke<ServerStatus>("server_ping", { address }),
  stop: (id: string) => invoke<void>("stop", { id }),
  cancelInstall: (id: string) => invoke<void>("cancel_install", { id }),
  running: () => invoke<RunningInfo[]>("running"),
  gameLog: (id: string) => invoke<LogLine[]>("game_log", { id }),
  presencePage: (page: string, idle: boolean) => invoke<void>("presence_page", { page, idle }),
};

export const events = {
  progress: (f: (p: Progress) => void): Promise<UnlistenFn> => listen<Progress>("progress", (e) => f(e.payload)),
  gameState: (f: (s: GameStateEvent) => void): Promise<UnlistenFn> => listen<GameStateEvent>("game-state", (e) => f(e.payload)),
  gameLog: (f: (l: LogLine) => void): Promise<UnlistenFn> => listen<LogLine>("game-log", (e) => f(e.payload)),
  updateProgress: (f: (p: UpdateProgress) => void): Promise<UnlistenFn> => listen<UpdateProgress>("update-progress", (e) => f(e.payload)),
};

/** Errors from commands arrive as plain strings. */
export function errText(e: unknown): string {
  if (typeof e === "string") return e;
  if (e instanceof Error) return e.message;
  return JSON.stringify(e);
}

/** URL of an imported image served by the backend's `media` protocol. */
export function mediaUrl(kind: "wallpaper" | "skin", id: string): string {
  return convertFileSrc(`${kind}/${id}`, "media");
}
