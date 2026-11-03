type Rgb = [number, number, number];

export function parseHex(hex: string): Rgb | null {
  const short = /^#([\da-f])([\da-f])([\da-f])$/i.exec(hex);
  const full = short ? [short[1] + short[1], short[2] + short[2], short[3] + short[3]] : /^#([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i.exec(hex)?.slice(1);
  return full ? (full.map((part) => parseInt(part, 16)) as Rgb) : null;
}

const toHex = (rgb: Rgb) => `#${rgb.map((c) => Math.round(c).toString(16).padStart(2, "0")).join("")}`;

export function luminance([r, g, b]: Rgb): number {
  const [lr, lg, lb] = [r, g, b].map((c) => (c /= 255) <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  return lr * 0.2126 + lg * 0.7152 + lb * 0.0722;
}

export function contrast(a: Rgb, b: Rgb): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

/** Selection marks must stay visible: pull an accent that vanishes into the surface towards black or white. */
export function usableAccent(hex: string, surface: string, fallback = "#3fb8a0"): string {
  const color = parseHex(hex) ?? parseHex(fallback)!;
  const base = parseHex(surface)!;
  const target = luminance(base) > 0.18 ? 0 : 255;
  for (let mix = 0; mix <= 0.9; mix += 0.05) {
    const shaded = color.map((c) => c + (target - c) * mix) as Rgb;
    if (contrast(shaded, base) >= 3) return toHex(shaded);
  }
  return fallback;
}

/** Foreground for text on an accent fill. */
export function accentForeground(hex: string): string {
  const color = parseHex(hex);
  return color && luminance(color) <= 0.179 ? "#f5f6f6" : "#101214";
}
