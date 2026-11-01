export function relativeTime(unixSecs: number | null | undefined): string {
  if (!unixSecs) return "nigdy";
  const diff = Date.now() / 1000 - unixSecs;
  if (diff < 60) return "przed chwilą";
  if (diff < 3600) return `${Math.floor(diff / 60)} min temu`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} godz. temu`;
  const days = Math.floor(diff / 86400);
  if (days === 1) return "wczoraj";
  if (days < 30) return `${days} dni temu`;
  return new Date(unixSecs * 1000).toLocaleDateString("pl-PL");
}

export function playTime(secs: number): string {
  if (secs < 60) return "0 min";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  return h > 0 ? `${h} godz. ${m} min` : `${m} min`;
}

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export function loaderLabel(l: string): string {
  return l === "fabric" ? "Fabric" : l === "quilt" ? "Quilt" : l === "rust" ? "Rust" : "Vanilla";
}

export function greeting(): string {
  const h = new Date().getHours();
  if (h < 5) return "Dobrej nocy";
  if (h < 12) return "Dzień dobry";
  if (h < 18) return "Cześć";
  return "Dobry wieczór";
}

export function compact(n: number): string {
  if (n >= 1e6) return `${(n / 1e6).toFixed(n >= 1e7 ? 0 : 1)} mln`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(n >= 1e4 ? 0 : 1)} tys.`;
  return String(n);
}

export function kindLabel(k: string, plural = true): string {
  if (k === "shader") return plural ? "Shadery" : "Shader";
  if (k === "resourcepack") return plural ? "Paczki zasobów" : "Paczka zasobów";
  return plural ? "Mody" : "Mod";
}
