import { locale, t } from "./i18n.svelte";

export function relativeTime(unixSecs: number | null | undefined): string {
  if (!unixSecs) return t("time.never");
  const diff = Date.now() / 1000 - unixSecs;
  if (diff < 60) return t("time.justNow");
  const relative = new Intl.RelativeTimeFormat(locale(), { numeric: "auto" });
  if (diff < 3600) return relative.format(-Math.floor(diff / 60), "minute");
  if (diff < 86400) return relative.format(-Math.floor(diff / 3600), "hour");
  const days = Math.floor(diff / 86400);
  if (days < 30) return relative.format(-days, "day");
  return new Date(unixSecs * 1000).toLocaleDateString(locale());
}

export function playTime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return t("time.playHours", { h, m });
  return t("time.playMinutes", { m });
}

/** A calendar date such as 1 listopada 2026; empty when the value does not parse. */
export function longDate(value: string): string {
  const parsed = new Date(/^\d{4}-\d{2}-\d{2}$/.test(value) ? `${value}T12:00:00` : value);
  if (Number.isNaN(parsed.getTime())) return "";
  return parsed.toLocaleDateString(locale(), { day: "numeric", month: "long", year: "numeric" });
}

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function loaderLabel(l: string): string {
  return l === "fabric" ? "Fabric" : l === "quilt" ? "Quilt" : l === "rust" ? "Rust" : "Vanilla";
}

export function compact(n: number): string {
  return new Intl.NumberFormat(locale(), { notation: "compact", maximumFractionDigits: 1 }).format(n);
}

export function kindLabel(k: string, plural = true): string {
  if (k === "shader") return t(plural ? "content.shaders" : "content.shader");
  if (k === "resourcepack") return t(plural ? "content.resourcepacks" : "content.resourcepack");
  return t(plural ? "content.mods" : "content.mod");
}
