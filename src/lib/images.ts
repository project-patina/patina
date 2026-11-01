import { open } from "@tauri-apps/plugin-dialog";
import { api, mediaUrl } from "./api";
import { BANNERS } from "./presets";

export function presetCss(key: string): string | undefined {
  return BANNERS[key];
}

/**
 * Image refs used across the app: `""` (none), `preset:<key>` (a gradient) or `custom:<id>` (an imported wallpaper). Bare keys are treated as
 * presets for older instance banners.
 */
export function refCss(ref: string | null | undefined, fallback = "none"): string {
  if (!ref) return fallback;
  if (ref.startsWith("custom:")) return `center / cover no-repeat url("${mediaUrl("wallpaper", ref.slice(7))}")`;
  const key = ref.startsWith("preset:") ? ref.slice(7) : ref;
  return presetCss(key) ?? fallback;
}

/** Let the user pick image files and import them as wallpapers. */
export async function importWallpapers(): Promise<string[]> {
  const picked = await open({
    multiple: true,
    title: "Wybierz obrazy",
    filters: [{ name: "Obrazy", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
  const ids: string[] = [];
  for (const p of paths) ids.push((await api.importImage("wallpaper", p)).id);
  return ids;
}
