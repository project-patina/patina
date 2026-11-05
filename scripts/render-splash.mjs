// Renders the loading animation of src/lib/blocks.ts into sprite sheets: a smooth one for both
// game clients and a lighter one, of the same frames at a lower rate, for the launcher's update
// screen (BlocksLoader.svelte).
// Run with: node --experimental-strip-types scripts/render-splash.mjs
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { blockTracks } from "../src/lib/blocks.ts";

const SIZE = 160, UNIT = SIZE / 7;
// The clients step a frame every 16 ms; the launcher every 80 ms, without the wordmark.
const SHEETS = [
  { columns: 16, frames: 320, wordmark: true, out: ["engine/crates/patina-theme/splash/patina-splash.png", "java-mod/src/main/resources/assets/patina/textures/gui/splash.png"] },
  { columns: 8, frames: 64, wordmark: false, out: ["src/assets/patina-splash.png"] },
];
const COLOURS = [
  { top: "#e9a06c", left: "#c47a47", right: "#7c4526", hidden: "#7c4526" },
  { top: "#6fdcc3", left: "#3fb8a0", right: "#2a8c7c", hidden: "#2a8c7c" },
];
const FACES = {
  front: [[0, 0, 1], [1, 0, 0], [0, 1, 0]], back: [[0, 0, -1], [-1, 0, 0], [0, 1, 0]],
  right: [[1, 0, 0], [0, 0, -1], [0, 1, 0]], left: [[-1, 0, 0], [0, 0, 1], [0, 1, 0]],
  top: [[0, -1, 0], [1, 0, 0], [0, 0, 1]], bottom: [[0, 1, 0], [1, 0, 0], [0, 0, -1]],
};
const apply = (m, [x, y, z]) => [0, 1, 2].map((i) => m[i] * x + m[4 + i] * y + m[8 + i] * z + m[12 + i]);
const yaw = Math.PI / 4, pitch = -Math.asin(1 / Math.sqrt(3));
// rotateX(pitch) rotateY(yaw) translate(-1, 1, -1): the scene transform of BlocksLoader.svelte.
const view = ([x, y, z]) => {
  const [a, b, c] = [x - 1, y + 1, z - 1];
  const rx = a * Math.cos(yaw) + c * Math.sin(yaw), rz = -a * Math.sin(yaw) + c * Math.cos(yaw);
  return [rx, b * Math.cos(pitch) - rz * Math.sin(pitch), b * Math.sin(pitch) + rz * Math.cos(pitch)];
};
const at = (frames, offset) => {
  const next = frames.findIndex((f) => f.offset >= offset);
  if (next <= 0) return frames[0].matrix;
  const [a, b] = [frames[next - 1], frames[next]], t = b.offset === a.offset ? 0 : (offset - a.offset) / (b.offset - a.offset);
  return a.matrix.map((v, i) => v + (b.matrix[i] - v) * t);
};

const { tracks, duration } = blockTracks();
// A small pixel wordmark is baked into the clients' sheet: the first loading screen has no font atlas.
const WORDMARK = [
  ["11110", "10001", "10001", "11110", "10000", "10000", "10000"],
  ["01110", "10001", "10001", "11111", "10001", "10001", "10001"],
  ["11111", "00100", "00100", "00100", "00100", "00100", "00100"],
  ["11111", "00100", "00100", "00100", "00100", "00100", "11111"],
  ["10001", "11001", "11001", "10101", "10011", "10011", "10001"],
  ["01110", "10001", "10001", "11111", "10001", "10001", "10001"],
];
const polygons = (offset) => {
  const out = [];
  tracks.forEach((track, i) => {
    const matrix = at(track.frames, offset);
    for (const { name, shade } of track.faces) {
      const [n, u, v] = FACES[name];
      const corners = [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([a, b]) => view(apply(matrix, n.map((c, k) => c / 2 + (u[k] * a + v[k] * b) / 2))));
      const [p, q, r] = corners;
      if ((q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0]) <= 0) continue;
      out.push({ depth: corners.reduce((s, c) => s + c[2], 0) / 4, colour: COLOURS[track.layer][shade], corners });
    }
  });
  return out.sort((a, b) => a.depth - b.depth);
};

const render = ({ columns, frames, wordmark, out }) => {
  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${columns * SIZE}" height="${Math.ceil(frames / columns) * SIZE}">`;
  for (let f = 0; f < frames; f++) {
    const ox = (f % columns) * SIZE + SIZE / 2, oy = Math.floor(f / columns) * SIZE + SIZE / 2;
    for (const { colour, corners } of polygons(f / frames)) {
      const points = corners.map(([x, y]) => `${(ox + x * UNIT).toFixed(2)},${(oy + y * UNIT).toFixed(2)}`).join(" ");
      svg += `<polygon points="${points}" fill="${colour}" stroke="${colour}" stroke-width="0.5"/>`;
    }
    const ms = f / frames * duration;
    const opacity = Math.min(1, Math.max(0, (ms - 2030) / 180)) * Math.min(1, Math.max(0, (3000 - ms) / 180));
    if (wordmark && opacity > 0) {
      WORDMARK.forEach((glyph, letter) => glyph.forEach((row, y) => [...row].forEach((pixel, x) => {
        if (pixel === "1") svg += `<rect x="${ox - 26.25 + (letter * 6 + x) * 1.5}" y="${oy + 48 + y * 1.5}" width="1.5" height="1.5" fill="#e8f0ee" opacity="${opacity.toFixed(3)}"/>`;
      })));
    }
  }
  svg += "</svg>";
  const work = mkdtempSync(join(tmpdir(), "patina-splash-"));
  writeFileSync(join(work, "sheet.svg"), svg);
  execFileSync("rsvg-convert", [join(work, "sheet.svg"), "-o", join(work, "sheet.png")]);
  for (const path of out) { mkdirSync(join(path, ".."), { recursive: true }); copyFileSync(join(work, "sheet.png"), path); }
  rmSync(work, { recursive: true });
  console.log(`${frames} frames of ${SIZE}px, ${columns} columns`, out);
};
for (const sheet of SHEETS) render(sheet);
console.log("duration ms:", duration);
