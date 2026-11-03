import { open } from "@tauri-apps/plugin-dialog";
import { api, mediaUrl } from "./api";
import { t } from "./i18n.svelte";
import { BANNERS } from "./presets";

export function presetCss(key: string): string | undefined {
  return BANNERS[key];
}

/**
 * Image refs used across the app: `""` (none), `preset:<key>` (a built-in scene) or `custom:<id>` (an imported wallpaper). Bare keys are treated as
 * presets for older instance banners.
 */
export function refCss(ref: string | null | undefined, fallback = "none"): string {
  if (!ref) return fallback;
  // A deleted image must not leave a hole: the fallback stays as the layer underneath.
  if (ref.startsWith("custom:")) return `center / cover no-repeat url("${mediaUrl("wallpaper", ref.slice(7))}"), ${fallback}`;
  const key = ref.startsWith("preset:") ? ref.slice(7) : ref;
  return presetCss(key) ?? fallback;
}

/** Let the user pick image files and import them as wallpapers. */
export async function importWallpapers(): Promise<string[]> {
  const picked = await open({
    multiple: true,
    title: t("images.pick"),
    filters: [{ name: t("images.filter"), extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
  });
  const paths = Array.isArray(picked) ? picked : picked ? [picked] : [];
  const ids: string[] = [];
  for (const p of paths) ids.push((await api.importImage("wallpaper", p)).id);
  return ids;
}
