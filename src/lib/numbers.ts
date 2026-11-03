/** Committed text of a whole-number field: blank or junk keeps `fallback`, the rest is clamped. */
export function parseWhole(text: string, min: number, max: number, fallback: number): number {
  const value = Number(text.trim());
  if (text.trim() === "" || !Number.isFinite(value)) return fallback;
  return Math.min(max, Math.max(min, Math.round(value)));
}
