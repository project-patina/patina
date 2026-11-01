import type { Settings } from "./types";

export const THEMES: { id: Settings["palette"]; name: string; accent: string; dark: string; light: string }[] = [
  { id: "patina", name: "Patina", accent: "#3fb8a0", dark: "#121215", light: "#f3f3f5" },
  { id: "graphite", name: "Grafit", accent: "#a8b0bd", dark: "#151619", light: "#f0f1f3" },
  { id: "ocean", name: "Ocean", accent: "#62aaf5", dark: "#101b28", light: "#edf3fa" },
  { id: "forest", name: "Las", accent: "#80bc8c", dark: "#151e18", light: "#eff5f0" },
  { id: "ember", name: "Miedź", accent: "#e4a16e", dark: "#231a15", light: "#f9f1ec" },
  { id: "iris", name: "Irys", accent: "#b3a0ed", dark: "#1b1727", light: "#f3eff9" },
];
