import { t } from "./i18n.svelte";
import { pl, type Key } from "./locales/pl";
import type { ContentKind } from "./types";

/** Modrinth's tags offered as filters for each kind of content. */
export const CATEGORIES: Record<ContentKind, string[]> = {
  mod: ["optimization", "technology", "adventure", "decoration", "magic", "storage", "utility", "worldgen", "mobs", "food", "library"],
  shader: ["realistic", "semi-realistic", "cartoon", "fantasy", "vanilla-like", "potato", "low", "high"],
  resourcepack: ["16x", "32x", "64x", "realistic", "simplistic", "themed", "tweaks", "combat", "gui", "fonts"],
};

/** A tag in the interface language; tags without a translation stay as Modrinth wrote them. */
export function categoryLabel(tag: string): string {
  const key = `category.${tag}`;
  return key in pl ? t(key as Key) : tag;
}
