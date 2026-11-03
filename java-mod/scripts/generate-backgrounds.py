#!/usr/bin/env python3
"""Draws the built-in title backgrounds of the Patina Fabric mod.

Everything here is original art made from code, with the Python standard library only.
The landscapes are the launcher banners of src/lib/presets.ts: same colours, same seeds and
the same stepped ridges. The output is committed, so this only runs when the art changes:

    python3 java-mod/scripts/generate-backgrounds.py
"""
import math
import pathlib
import random
import struct
import zlib

OUT = pathlib.Path(__file__).resolve().parent.parent / "src/main/resources/assets/patina/textures/gui/background"

# The banner scene is 960x384 with 24 unit blocks. Half of that keeps every block edge on a pixel.
SCENE_W, SCENE_H, BLOCK, SCALE = 960, 384, 24, 2

# In the order of BANNERS, so landscape-<n> means the same scene in Rust and Java.
BANNERS = [
    ("aurora", "#1c2a2e", "#27414a", "#2c5a5c", "#2f7a6e", 11),
    ("sunset", "#3a2a2c", "#5a3a33", "#7c4e38", "#a7683f", 23),
    ("forest", "#1e2b24", "#2a4034", "#335740", "#3f734b", 37),
    ("ocean", "#1a2836", "#23405a", "#29597c", "#2e7499", 41),
    ("nether", "#2c1a1a", "#4a2422", "#6a2e26", "#8f3d2a", 53),
    ("end", "#221e30", "#352e4d", "#4a3f6c", "#655590", 67),
    ("candy", "#2e2230", "#4a3050", "#6a4070", "#8c5490", 71),
    ("mono", "#1f2122", "#2c2f30", "#3a3e40", "#4b5053", 83),
]

NIGHT_W, NIGHT_H = 640, 360


def rgb(text):
    return tuple(int(text[i:i + 2], 16) for i in (1, 3, 5))


def chunk(tag, data):
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))


def png(width, height, color_type, rows, palette=None):
    head = struct.pack(">IIBBBBB", width, height, 8, color_type, 0, 0, 0)
    body = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", head)
    if palette:
        body += chunk(b"PLTE", bytes(v for color in palette for v in color))
    return body + chunk(b"IDAT", zlib.compress(b"".join(rows), 9)) + chunk(b"IEND", b"")


def ridge(seed, base, swing):
    """The height of the skyline per scene unit: presets.ts `ridge` step for step."""
    heights = [0] * SCENE_W
    x, y, state = 0, base, seed
    while x < SCENE_W:
        state = (state * 1103515245 + 12345) % 2147483648
        run = BLOCK * (2 + (state >> 8) % 5)
        step = BLOCK * ((state >> 16) % 3 - 1)
        end = min(SCENE_W, x + run)
        # SVG: H then V, so the new height starts where this run ends.
        for i in range(x, end):
            heights[i] = y
        x = end
        y = max(base - swing, min(base + swing, y + step))
    return heights


def landscape(index, banner):
    _, sky, far, mid, near, seed = banner
    layers = [ridge(seed, 216, 48), ridge(seed + 7, 264, 48), ridge(seed + 13, 312, 24)]
    palette = [rgb(sky), rgb(far), rgb(mid), rgb(near)]
    width, height = SCENE_W // SCALE, SCENE_H // SCALE
    rows = []
    for py in range(height):
        row = bytearray([0])
        for px in range(width):
            x, y, color = px * SCALE, py * SCALE, 0
            for level, heights in enumerate(layers):
                if y >= heights[x]:
                    color = level + 1
            row.append(color)
        rows.append(bytes(row))
    (OUT / f"landscape-{index}.png").write_bytes(png(width, height, 3, rows, palette))


def mix(a, b, t):
    t = max(0.0, min(1.0, t))
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def add(color, light, amount):
    return tuple(c + l * amount for c, l in zip(color, light))


def smooth(edge0, edge1, x):
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3 - 2 * t)


def night():
    rng = random.Random(26)
    top, middle, low = (5, 8, 20), (10, 18, 40), (20, 52, 66)
    aurora_near, aurora_far = (63, 184, 160), (56, 92, 170)
    moon_x, moon_y, moon_r = NIGHT_W * 0.60, NIGHT_H * 0.19, 15.0

    craters = [(-5.0, -4.0, 4.5), (5.0, 3.0, 5.5), (-3.0, 7.0, 3.0), (6.0, -7.0, 2.5)]

    stars = {}
    for _ in range(420):
        x, y = rng.randrange(NIGHT_W), rng.randrange(int(NIGHT_H * 0.72))
        stars[x, y] = rng.random() ** 2.4 * 0.9 + 0.1 * (1 - y / NIGHT_H)

    # The same stepped ridges as the landscapes, as three layers of hills under the moon.
    layers = [
        (ridge(131, 228, 48), (12, 26, 40), 0.55),
        (ridge(138, 272, 40), (8, 18, 30), 0.3),
        (ridge(144, 320, 24), (4, 9, 16), 0.1),
    ]
    rows = []
    for py in range(NIGHT_H):
        row = bytearray([0])
        v = py / NIGHT_H
        for px in range(NIGHT_W):
            u = px / NIGHT_W
            sky = mix(top, middle, v / 0.55) if v < 0.55 else mix(middle, low, (v - 0.55) / 0.3)

            # Curtains of light that hang above the hills and thin out towards the top.
            wave = math.sin(u * 7.0 + 1.3) * 0.055 + math.sin(u * 17.0 + 0.4) * 0.018
            band = math.exp(-(((v - 0.40 - wave) / 0.135) ** 2))
            rays = 0.62 + 0.38 * math.sin(u * 58.0 + math.sin(u * 13.0) * 3.0) * math.sin(u * 23.0 + 2.0)
            reach = smooth(0.08, 0.34, u) * (1 - smooth(0.76, 0.98, u) * 0.75)
            sky = add(sky, mix(aurora_near, aurora_far, u * 1.15 + (0.44 - v) * 1.6), 0.58 * band * rays * reach)

            dist = math.hypot(px - moon_x, py - moon_y)
            sky = add(sky, (150, 180, 200), 0.28 * math.exp(-dist / 34.0) + 0.09 * math.exp(-dist / 120.0))
            if dist < moon_r + 1:
                disc = smooth(moon_r + 1, moon_r - 1, dist)
                shade = sum(0.12 * math.exp(-(math.hypot(px - moon_x - cx, py - moon_y - cy) / r) ** 2) for cx, cy, r in craters)
                sky = mix(sky, (226, 235, 232), disc * (0.97 - shade))

            star = stars.get((px, py))
            if star and dist > moon_r + 3:
                sky = add(sky, (235, 240, 255), star * (1 - band * 0.6))

            color = sky
            scene_x, scene_y = u * SCENE_W, v * SCENE_H
            for heights, shade, haze in layers:
                edge = heights[int(scene_x)]
                if scene_y >= edge:
                    depth = (scene_y - edge) / 96.0
                    color = mix(shade, low, haze * (1 - min(1.0, depth)) * 0.6)
                    rim = 1 - smooth(0.0, 5.0, scene_y - edge)
                    color = add(color, (70, 150, 150), 0.10 * rim)

            # A little noise keeps the long gradients from banding when scaled up.
            grain = rng.random() + rng.random() - 1
            row += bytes(max(0, min(255, round(c + grain * 0.8))) for c in color)
        rows.append(bytes(row))
    (OUT / "night.png").write_bytes(png(NIGHT_W, NIGHT_H, 2, rows))


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for index, banner in enumerate(BANNERS):
        landscape(index, banner)
    night()


if __name__ == "__main__":
    main()
