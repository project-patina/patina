export type Loader = "vanilla" | "fabric" | "quilt" | "rust";

export interface EngineSettings {
  lodDistance: number | null;
  resolutionScale: number | null;
  antialiasing: "game" | "off" | "fxaa" | "taa";
  graphicsApi: "game" | "auto" | "vulkan" | "opengl";
  hdr: boolean | null;
  batteryMode: boolean | null;
  fpsOverlay: boolean | null;
  xwayland: boolean;
}

export interface Settings {
  theme: "dark" | "light" | "system";
  palette: "patina" | "graphite" | "ocean" | "forest" | "ember" | "iris";
  accent: string;
  animations: boolean;
  blur: boolean;
  uiScale: number;
  font: "inter" | "outfit" | "system" | "mono";
  radius: "sharp" | "normal" | "round";
  compact: boolean;
  heroWallpaper: string;
  heroScene: boolean;
  heroDim: number;
  appWallpaper: string;
  appWallpaperDim: number;
  modelPosition: "center" | "left" | "right";
  followCursor: boolean;
  showNametag: boolean;
  showSnapshots: boolean;
  showOldVersions: boolean;
  defaultLoader: Loader;
  installPerformanceMods: boolean;
  memoryMinMb: number;
  memoryMaxMb: number;
  javaPath: string;
  jvmArgs: string;
  optimizedJvmArgs: boolean;
  gameWidth: number;
  gameHeight: number;
  fullscreen: boolean;
  onLaunch: "keep" | "minimize" | "hide" | "close";
  reopenOnExit: boolean;
  openConsoleOnLaunch: boolean;
  maxConcurrentDownloads: number;
  gamemode: boolean;
  mangohud: boolean;
  discreteGpu: boolean;
  envVars: string;
  wrapperCommand: string;
  preLaunchCommand: string;
  postExitCommand: string;
  rustEngine: EngineSettings;
  /** JSON of the home screen stance; empty for the default. */
  modelPose: string;
  brandVersion: number;
  onboarded: boolean;
  lastInstance: string | null;
}

export interface Overrides {
  memoryMinMb?: number | null;
  memoryMaxMb?: number | null;
  javaPath?: string | null;
  jvmArgs?: string | null;
  gameWidth?: number | null;
  gameHeight?: number | null;
  fullscreen?: boolean | null;
  envVars?: string | null;
  wrapperCommand?: string | null;
  joinServer?: string | null;
}

export interface Instance {
  id: string;
  name: string;
  mcVersion: string;
  loader: Loader;
  loaderVersion: string | null;
  icon: string;
  banner: string;
  createdAt: number;
  lastPlayed: number | null;
  playTimeSecs: number;
  overrides: Overrides;
  installed: boolean;
}

export interface NewInstance {
  name: string;
  mcVersion: string;
  loader: Loader;
  loaderVersion?: string | null;
  icon?: string;
  banner?: string;
}

export interface Account {
  uuid: string;
  name: string;
  skinUrl: string | null;
  skinSlim: boolean;
  capeUrl: string | null;
  active: boolean;
}

export interface DeviceCode {
  userCode: string;
  deviceCode: string;
  verificationUri: string;
  interval: number;
  expiresIn: number;
}

export interface GameVersion {
  id: string;
  kind: "release" | "snapshot" | "old_beta" | "old_alpha";
  releaseTime: string;
}

export interface VersionList {
  latestRelease: string;
  latestSnapshot: string;
  versions: GameVersion[];
}

export interface LoaderVersion {
  version: string;
  stable: boolean;
}

export interface ModFile {
  file: string;
  enabled: boolean;
  size: number;
}

export interface JavaInstall {
  path: string;
  version: string;
  major: number;
  source: "bundled" | "system";
}

export interface AppInfo {
  version: string;
  dataDir: string;
  totalMemoryMb: number;
  os: string;
  arch: string;
  startPage?: string | null;
}

export interface ReleaseNote {
  version: string;
  date: string;
  title: string;
  summary: string;
  sections: { title: string; items: string[] }[];
}

export interface Progress {
  task: string;
  stage: string;
  done: number;
  total: number;
  message: string;
}

export interface GameStateEvent {
  id: string;
  state: "preparing" | "running" | "exited" | "crashed" | "error" | "cancelled";
  message: string | null;
  code: number | null;
}

export interface LogLine {
  id: string;
  line: string;
  error: boolean;
}

export interface RunningInfo {
  id: string;
  state: "preparing" | "running";
  started: number | null;
}

export type ContentKind = "mod" | "resourcepack" | "shader";

export interface ContentMeta {
  kind: ContentKind;
  projectId: string;
  versionId: string;
  title: string;
  iconUrl: string | null;
  versionNumber: string;
}

export interface ContentItem {
  file: string;
  enabled: boolean;
  size: number;
  meta: ContentMeta | null;
}

export interface SearchQuery {
  query: string;
  kind: ContentKind;
  gameVersion?: string | null;
  loader?: Loader | null;
  categories?: string[];
  sort?: "relevance" | "downloads" | "follows" | "newest" | "updated";
  offset?: number;
}

export interface SearchHit {
  projectId: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  iconUrl: string | null;
  downloads: number;
  follows: number;
  categories: string[];
  displayCategories: string[];
  projectType: string;
  dateModified: string;
  featuredGallery: string | null;
  color: number | null;
}

export interface SearchResult {
  hits: SearchHit[];
  offset: number;
  limit: number;
  totalHits: number;
}

export interface Project {
  id: string;
  slug: string;
  title: string;
  description: string;
  body: string;
  iconUrl: string | null;
  downloads: number;
  followers: number;
  categories: string[];
  projectType: string;
  gallery: { url: string; title: string | null; featured: boolean }[];
  sourceUrl: string | null;
  issuesUrl: string | null;
  wikiUrl: string | null;
  discordUrl: string | null;
  license: { id: string; name: string | null } | null;
  clientSide: string | null;
  serverSide: string | null;
  gameVersions: string[];
  loaders: string[];
  updated: string | null;
}

export interface ModVersion {
  id: string;
  projectId: string;
  name: string;
  versionNumber: string;
  versionType: "release" | "beta" | "alpha";
  gameVersions: string[];
  loaders: string[];
  datePublished: string;
  downloads: number;
  files: { url: string; filename: string; primary: boolean; size: number }[];
  dependencies: { projectId: string | null; versionId: string | null; dependencyType: string }[];
  changelog: string | null;
}

export interface Installed {
  title: string;
  file: string;
  dependency: boolean;
}

export interface ContentUpdate {
  file: string;
  projectId: string;
  versionId: string;
  versionNumber: string;
}

export interface GearTextures {
  sword: string;
  shield: string;
}

export interface Image {
  id: string;
  size: number;
  added: number;
  name: string;
}

export interface McTexture {
  id: string;
  state: "ACTIVE" | "INACTIVE";
  url: string;
  variant: string | null;
  alias: string | null;
}

export interface McProfile {
  id: string;
  name: string;
  skins: McTexture[];
  capes: McTexture[];
}
