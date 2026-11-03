import { t } from "./i18n.svelte";
import { refCss } from "./images";
import {
  Axe, Box, Castle, Crown, Flame, Gem, Leaf, Mountain, Pickaxe, Rocket, Skull, Snowflake, Sparkles, Sword, Trees, Zap,
} from "@lucide/svelte";

export const ICONS = {
  grass: Trees,
  diamond: Gem,
  sword: Sword,
  pickaxe: Pickaxe,
  axe: Axe,
  fire: Flame,
  mountain: Mountain,
  castle: Castle,
  rocket: Rocket,
  zap: Zap,
  leaf: Leaf,
  snow: Snowflake,
  skull: Skull,
  crown: Crown,
  box: Box,
  sparkles: Sparkles,
} as const;

export type IconKey = keyof typeof ICONS;

export function iconFor(key: string) {
  return ICONS[key as IconKey] ?? Box;
}

const BLOCK = 24;
const SCENE_W = 960;
const SCENE_H = 384;

/** One stepped ridge, built from a small seeded sequence so the skyline never changes. */
function ridge(seed: number, base: number, swing: number, fill: string): string {
  let x = 0;
  let y = base;
  let state = seed;
  let path = `M0 ${SCENE_H}V${y}`;
  while (x < SCENE_W) {
    state = (state * 1103515245 + 12345) % 2147483648;
    const run = BLOCK * (2 + (state >> 8) % 5);
    const step = BLOCK * ((state >> 16) % 3 - 1);
    x = Math.min(SCENE_W, x + run);
    y = Math.max(base - swing, Math.min(base + swing, y + step));
    path += `H${x}V${y}`;
  }
  return `<path fill="${fill}" d="${path}V${SCENE_H}H0Z"/>`;
}

/** A blocky horizon in three flat tones, as a full CSS background. */
function scene(sky: string, far: string, mid: string, near: string, seed: number): string {
  const art = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${SCENE_W} ${SCENE_H}" preserveAspectRatio="xMidYMax slice">`
    + ridge(seed, 216, 48, far) + ridge(seed + 7, 264, 48, mid) + ridge(seed + 13, 312, 24, near) + "</svg>";
  return `center bottom / cover no-repeat url("data:image/svg+xml,${encodeURIComponent(art)}"), ${sky}`;
}

/** Built-in banners. Each is a full CSS background. */
export const BANNERS: Record<string, string> = {
  aurora: scene("#1c2a2e", "#27414a", "#2c5a5c", "#2f7a6e", 11),
  sunset: scene("#3a2a2c", "#5a3a33", "#7c4e38", "#a7683f", 23),
  forest: scene("#1e2b24", "#2a4034", "#335740", "#3f734b", 37),
  ocean: scene("#1a2836", "#23405a", "#29597c", "#2e7499", 41),
  nether: scene("#2c1a1a", "#4a2422", "#6a2e26", "#8f3d2a", 53),
  end: scene("#221e30", "#352e4d", "#4a3f6c", "#655590", 67),
  candy: scene("#2e2230", "#4a3050", "#6a4070", "#8c5490", 71),
  mono: scene("#1f2122", "#2c2f30", "#3a3e40", "#4b5053", 83),
};

export function bannerFor(key: string) {
  // Imported images go through the shared image refs.
  if (key.startsWith("custom:") || key.startsWith("preset:")) return refCss(key, BANNERS.aurora);
  return BANNERS[key] ?? BANNERS.aurora;
}

export const ACCENTS = ["#3fb8a0", "#e08a4f", "#6366f1", "#0ea5e9", "#10b981", "#22c55e", "#f59e0b", "#f97316", "#ef4444", "#ec4899", "#e4e4e7"];

export const LOADERS = [
  { id: "fabric", label: "Fabric", get hint() { return t("loader.fabric"); } },
  { id: "rust", label: "Rust", get hint() { return t("loader.rust"); } },
] as const;

/// The only game version the Rust engine runs (`engine::MC_VERSION`).
export const ENGINE_MC = "26.3";
