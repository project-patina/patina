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

/** Banner gradients. Each is a full CSS background. */
export const BANNERS: Record<string, string> = {
  aurora: "radial-gradient(120% 140% at 0% 0%, #7c3aed 0%, transparent 55%), radial-gradient(120% 140% at 100% 100%, #0ea5e9 0%, transparent 55%), #10101a",
  sunset: "radial-gradient(120% 140% at 10% 0%, #f97316 0%, transparent 55%), radial-gradient(120% 140% at 100% 100%, #db2777 0%, transparent 60%), #1a0f12",
  forest: "radial-gradient(120% 140% at 0% 100%, #16a34a 0%, transparent 55%), radial-gradient(100% 120% at 100% 0%, #65a30d 0%, transparent 55%), #0c140e",
  ocean: "radial-gradient(120% 140% at 0% 0%, #0284c7 0%, transparent 55%), radial-gradient(120% 140% at 100% 100%, #14b8a6 0%, transparent 55%), #0a1318",
  nether: "radial-gradient(120% 140% at 0% 100%, #b91c1c 0%, transparent 55%), radial-gradient(100% 120% at 100% 0%, #ea580c 0%, transparent 55%), #170a0a",
  end: "radial-gradient(120% 140% at 50% 0%, #a855f7 0%, transparent 55%), radial-gradient(120% 140% at 50% 120%, #eab308 0%, transparent 50%), #0f0a16",
  candy: "radial-gradient(120% 140% at 0% 0%, #ec4899 0%, transparent 55%), radial-gradient(120% 140% at 100% 100%, #8b5cf6 0%, transparent 55%), #160d17",
  mono: "radial-gradient(120% 140% at 0% 0%, #52525b 0%, transparent 55%), radial-gradient(120% 140% at 100% 100%, #27272a 0%, transparent 55%), #0d0d0f",
};

export function bannerFor(key: string) {
  // Imported images go through the shared image refs.
  if (key.startsWith("custom:") || key.startsWith("preset:")) return refCss(key, BANNERS.aurora);
  return BANNERS[key] ?? BANNERS.aurora;
}

export const ACCENTS = ["#3fb8a0", "#e08a4f", "#6366f1", "#0ea5e9", "#10b981", "#22c55e", "#f59e0b", "#f97316", "#ef4444", "#ec4899", "#e4e4e7"];

export const LOADERS = [
  { id: "fabric", label: "Fabric", hint: "Lekki, najszybszy, mody optymalizacyjne" },
  { id: "quilt", label: "Quilt", hint: "Fork Fabrica, zgodny z jego modami" },
  { id: "vanilla", label: "Vanilla", hint: "Czysta gra bez modów" },
  { id: "rust", label: "Rust", hint: "Gra przepisana w Ruście, tylko 26.3" },
] as const;

/// The only game version the Rust engine runs (`engine::MC_VERSION`).
export const ENGINE_MC = "26.3";
