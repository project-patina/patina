// Renders Patina's logo: the cube the loading animation builds (src/lib/blocks.ts), drawn
// from the same geometry and colours, and writes every copy of it the project ships.
// Run with: node --experimental-strip-types scripts/render-logo.mjs
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { blockTracks } from "../src/lib/blocks.ts";

const COLOURS = [
  { top: "#e9a06c", left: "#c47a47", right: "#7c4526" },
  { top: "#6fdcc3", left: "#3fb8a0", right: "#2a8c7c" },
];
const FACES = {
  front: [[0, 0, 1], [1, 0, 0], [0, 1, 0]], back: [[0, 0, -1], [-1, 0, 0], [0, 1, 0]],
  right: [[1, 0, 0], [0, 0, -1], [0, 1, 0]], left: [[-1, 0, 0], [0, 0, 1], [0, 1, 0]],
  top: [[0, -1, 0], [1, 0, 0], [0, 0, 1]], bottom: [[0, 1, 0], [1, 0, 0], [0, 0, -1]],
};
const SIZE = 512, MARGIN = 24;
const apply = (m, [x, y, z]) => [0, 1, 2].map((i) => m[i] * x + m[4 + i] * y + m[8 + i] * z + m[12 + i]);
const yaw = Math.PI / 4, pitch = -Math.asin(1 / Math.sqrt(3));
// The scene transform of the animation (see scripts/render-splash.mjs).
const view = ([x, y, z]) => {
  const [a, b, c] = [x - 1, y + 1, z - 1];
  const rx = a * Math.cos(yaw) + c * Math.sin(yaw), rz = -a * Math.sin(yaw) + c * Math.cos(yaw);
  return [rx, b * Math.cos(pitch) - rz * Math.sin(pitch), b * Math.sin(pitch) + rz * Math.cos(pitch)];
};

// Every block at home: the finished cube. Only the faces turned to the viewer are drawn.
const faces = [];
for (const track of blockTracks().tracks) {
  for (const { name, shade } of track.faces) {
    if (shade === "hidden") continue;
    const [n, u, v] = FACES[name];
    const corners = [[-1, -1], [1, -1], [1, 1], [-1, 1]].map(([a, b]) => view(apply(track.home, n.map((c, k) => c / 2 + (u[k] * a + v[k] * b) / 2))));
    faces.push({ depth: corners.reduce((s, c) => s + c[2], 0) / 4, colour: COLOURS[track.layer][shade], corners });
  }
}
faces.sort((a, b) => a.depth - b.depth);

const xs = faces.flatMap((f) => f.corners.map((c) => c[0])), ys = faces.flatMap((f) => f.corners.map((c) => c[1]));
const [minX, maxX, minY, maxY] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
const scale = (SIZE - MARGIN * 2) / Math.max(maxX - minX, maxY - minY);
const ox = SIZE / 2 - (minX + maxX) / 2 * scale, oy = SIZE / 2 - (minY + maxY) / 2 * scale;
// The stroke in the face's own colour closes the hairlines between neighbouring blocks.
const polygons = faces.map(({ colour, corners }) => {
  const points = corners.map(([x, y]) => `${(ox + x * scale).toFixed(1)},${(oy + y * scale).toFixed(1)}`).join(" ");
  return `  <polygon points="${points}" fill="${colour}" stroke="${colour}" stroke-width="1.5" stroke-linejoin="round"/>`;
});
const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${SIZE} ${SIZE}">\n${polygons.join("\n")}\n</svg>\n`;

for (const path of ["public/icon.svg", "site/assets/icon.svg"]) writeFileSync(path, svg);
const work = mkdtempSync(join(tmpdir(), "patina-logo-"));
const png = (size, path) => execFileSync("rsvg-convert", ["-w", String(size), "-h", String(size), "public/icon.svg", "-o", path]);
png(1024, join(work, "icon-1024.png"));
// The app icons for every platform, from one large picture.
execFileSync("npx", ["tauri", "icon", join(work, "icon-1024.png"), "-o", join(work, "icons")], { stdio: "ignore" });
execFileSync("sh", ["-c", `cp "${join(work, "icons")}"/*.png "${join(work, "icons")}"/icon.ico "${join(work, "icons")}"/icon.icns src-tauri/icons/`]);
png(64, "engine/crates/patina-theme/icons/patina-64.png");
png(160, "java-mod/src/main/resources/assets/patina/icon.png");
png(512, "site/assets/discord-icon.png");
rmSync(work, { recursive: true });
console.log(`${faces.length} faces; wrote public/icon.svg, site/assets, src-tauri/icons, the Rust window icon and the Fabric mod icon`);
