import { t } from "./i18n.svelte";
import type { Settings } from "./types";

export const THEMES: { id: Settings["palette"]; readonly name: string; accent: string; dark: string; light: string }[] = [
  { id: "patina", get name() { return t("palette.patina"); }, accent: "#3fb8a0", dark: "#141716", light: "#f3f5f4" },
  { id: "graphite", get name() { return t("palette.graphite"); }, accent: "#a8b0bd", dark: "#151619", light: "#f2f3f4" },
  { id: "ocean", get name() { return t("palette.ocean"); }, accent: "#62aaf5", dark: "#121a24", light: "#f0f4f8" },
  { id: "forest", get name() { return t("palette.forest"); }, accent: "#80bc8c", dark: "#141c17", light: "#f1f5f2" },
  { id: "ember", get name() { return t("palette.ember"); }, accent: "#e4a16e", dark: "#1f1814", light: "#f7f3f0" },
  { id: "iris", get name() { return t("palette.iris"); }, accent: "#b3a0ed", dark: "#191622", light: "#f4f2f8" },
];
