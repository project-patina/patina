//! HUD boxes in the corners: the minimap's place and mods' panels, small boxes
//! in GUI pixels stacked per corner, and the Patina client's built-in mods,
//! each where it was dragged to, drawn as the Patina Fabric mod draws them
//! (`PatinaHud`).
use crate::{
    client_settings::{Anchor, FpsAccuracy, FpsStyle, Mod, Mods, Place},
    extras::{Corner, FpsReading},
    hud::{HudAtlas, HudBatch, Rect, rgba},
    lang,
};

/// The gap between an overlay and the screen edge or the next overlay.
const MARGIN: f32 = 2.0;
const BACKGROUND: u32 = 0x7800_0000;

/// Where the next overlay goes in each corner.
pub struct Corners {
    width: f32,
    height: f32,
    /// Distance from the corner's top or bottom edge, per `Corner::ALL`.
    used: [f32; 4],
}
impl Corners {
    /// `top_right` reserves the status effect icons' rows.
    pub fn new(width: f32, height: f32, top_right: f32) -> Self {
        Self {
            width,
            height,
            used: [0.0, top_right, 0.0, 0.0],
        }
    }
    /// The top-left position of a `w` x `h` box in `corner`, below (or
    /// above) what the corner already holds.
    pub fn place(&mut self, corner: Corner, w: f32, h: f32) -> (f32, f32) {
        let i = Corner::ALL.iter().position(|c| *c == corner).unwrap_or(0);
        let edge = self.used[i] + MARGIN;
        self.used[i] = edge + h;
        let x = if corner.left() {
            MARGIN
        } else {
            self.width - MARGIN - w
        };
        let y = if corner.top() {
            edge
        } else {
            self.height - edge - h
        };
        (x, y)
    }
}

/// How the minimap shows: its corner and side in GUI pixels, its image (an
/// atlas rect), the player arrow (the map item's decoration sprite) turned
/// `arrow` degrees clockwise from up, and the rim's N toward `north` when the
/// map turns.
pub struct MinimapLook {
    pub corner: Corner,
    pub size: f32,
    pub image: Rect,
    pub arrow: f32,
    pub north: Option<(f32, f32)>,
}
pub fn draw_minimap(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    look: &MinimapLook,
) {
    let MinimapLook {
        corner,
        size,
        image,
        arrow,
        north,
    } = *look;
    let (x, y) = corners.place(corner, size, size);
    batch.rect_uv(x, y, size, size, image, u32::MAX);
    let (cx, cy) = (x + size / 2.0, y + size / 2.0);
    if let Some(r) = atlas.sprites.get("map/decorations/player").copied() {
        // MapRenderer's decorations draw the sprite flipped: its arrow
        // points down the texture, so up on screen is half a turn.
        let (s, c) = (arrow + 180.0).to_radians().sin_cos();
        let p = |dx: f32, dy: f32| [cx + dx * c - dy * s, cy + dx * s + dy * c];
        let h = 4.0;
        batch.quad(
            [p(-h, h), p(h, h), p(h, -h), p(-h, -h)],
            [[r.u0, r.v0], [r.u1, r.v0], [r.u1, r.v1], [r.u0, r.v1]],
            u32::MAX,
        );
    }
    if let Some((nx, ny)) = north {
        let reach = size / 2.0 - 6.0;
        let tw = HudBatch::text_width(atlas, "N");
        let (tx, ty) = (cx + nx * reach - tw / 2.0, cy + ny * reach - 4.0);
        batch.text(
            atlas,
            "N",
            tx.round(),
            ty.round(),
            rgba(255, 255, 255, 255),
            true,
        );
    }
}

/// A box of text lines (a mod's HUD panel): `(text, 0xRRGGBB)` each.
pub fn draw_lines(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    corners: &mut Corners,
    corner: Corner,
    lines: &[(&str, u32)],
) {
    if lines.is_empty() {
        return;
    }
    let w = lines
        .iter()
        .map(|(text, _)| HudBatch::text_width(atlas, text))
        .fold(0.0, f32::max)
        + 4.0;
    let h = 10.0 * lines.len() as f32 + 2.0;
    let (x, y) = corners.place(corner, w, h);
    batch.fill(atlas, x, y, w, h, BACKGROUND);
    for (i, (text, color)) in lines.iter().enumerate() {
        let [_, r, g, b] = color.to_be_bytes();
        let y = y + 2.0 + 10.0 * i as f32;
        batch.text(atlas, text, x + 2.0, y, rgba(r, g, b, 255), true);
    }
}

/// What the built-in mods show this frame. The HUD editor passes the game's
/// (`remember`) or made-up values through the same drawing.
#[derive(Clone, Debug)]
pub struct ModReadings {
    pub fps: FpsReading,
    /// Forward, left, back, right, attack and use held.
    pub keys: [bool; 8],
    pub position: [f64; 3],
    /// Degrees, as the player's yaw: 0 faces south.
    pub yaw: f32,
    pub pitch: f32,
    pub biome: String,
    pub left_offhand: bool,
    /// Helmet, chest, leggings, boots; hands and shields are never included.
    pub armor: [ArmorReading; 4],
    pub speed: f64, pub ping: i32, pub effects: Vec<EffectReading>,
}
#[derive(Clone, Debug)]
pub struct EffectReading { pub id: String, pub amplifier: i32, pub ticks: i32 }
pub fn sample_effects() -> Vec<EffectReading> { ["regeneration", "speed", "strength", "jump_boost", "fire_resistance", "night_vision"].into_iter().enumerate().map(|(i,id)| EffectReading { id:id.into(), amplifier: if i<4 {2}else{0}, ticks: if i%2==0 {5400}else{1160} }).collect() }
pub fn effect_duration(ticks:i32)->String { if ticks<0 {"∞".into()}else if ticks>=1200{format!("{}M",(ticks+1199)/1200)}else{format!("{}S",(ticks+19)/20)} }
pub fn effect_roman(level:i32)->String { if (1..=10).contains(&level) {["I","II","III","IV","V","VI","VII","VIII","IX","X"][(level-1) as usize].into()}else{level.to_string()} }
fn info_text(m:Mod,mods:&Mods,r:&ModReadings)->String { if m==Mod::Ping {format!("{} ms",if r.ping<0 {"—".into()}else{r.ping.to_string()})}else{format!("{:.1} {}",r.speed*if mods.speedometer.unit=="km/h"{3.6}else{1.0},mods.speedometer.unit)} }
#[derive(Clone, Debug, Default)]
pub struct ArmorReading {
    pub item: Option<String>,
    pub remaining: i32,
    pub maximum: i32,
}
pub fn sample_armor() -> [ArmorReading; 4] {
    [ArmorReading { item: Some("minecraft:turtle_helmet".into()), remaining: 8, maximum: 275 },
     ArmorReading::default(),
     ArmorReading { item: Some("minecraft:iron_leggings".into()), remaining: 225, maximum: 225 },
     ArmorReading { item: Some("minecraft:golden_boots".into()), remaining: 65, maximum: 91 }]
}
/// The FPS mod's line.
pub fn fps_text(reading: &FpsReading, accuracy: FpsAccuracy) -> String {
    match accuracy {
        FpsAccuracy::Whole => format!("{} FPS", reading.fps),
        FpsAccuracy::Exact => format!("{:.1} FPS  1% {}", reading.exact, reading.low),
    }
}
/// `Direction.fromYRot`: the facing's lang key and the axis it runs along.
fn facing(yaw: f32) -> (&'static str, &'static str, &'static str) {
    match ((yaw / 90.0).round() as i32).rem_euclid(4) {
        0 => ("patina.client.south", "South", "+Z"),
        1 => ("patina.client.west", "West", "-X"),
        2 => ("patina.client.north", "North", "-Z"),
        _ => ("patina.client.east", "East", "+X"),
    }
}
/// The coordinates mod's two lines.
pub fn coordinate_lines(position: [f64; 3], yaw: f32) -> [String; 2] {
    let [x, y, z] = position.map(|v| v.floor() as i64);
    let (key, plain, axis) = facing(yaw);
    [
        format!("{x} {y} {z}"),
        format!("{} {axis}", lang::get_or(key, plain)),
    ]
}

/// The gap between a mod and the screen edge it keeps to.
pub const MOD_MARGIN: f32 = 4.0;
const PANEL_PADDING: f32 = 3.0;
const LINE_HEIGHT: f32 = 9.0;
const PANEL: u32 = rgba(0, 0, 0, 0x70);
const TEXT: u32 = rgba(0xE8, 0xF0, 0xEE, 255);
const DIM_TEXT: u32 = rgba(0x9F, 0xB0, 0xAC, 255);
const KEY: f32 = 18.0;
const KEY_GAP: f32 = 2.0;
const KEY_IDLE: u32 = rgba(0, 0, 0, 0x70);
const KEY_DOWN: u32 = rgba(0x3F, 0xB8, 0xA0, 0xC0);

/// A mod's size at a scale of 1.
pub fn mod_size(atlas: &HudAtlas, mods: &Mods, readings: &ModReadings, m: Mod) -> (f32, f32) {
    let width = |text: &str| HudBatch::text_width(atlas, text).ceil();
    match m {
        Mod::Speedometer | Mod::Ping => { let pad=if if m==Mod::Speedometer {mods.speedometer.background}else{mods.ping.background}{3.0}else{0.0}; (width(&info_text(m,mods,readings))+pad*2.0,8.0+pad*2.0) }
        Mod::Effects => { let count=readings.effects.len().max(1);let columns=(mods.status_effects.columns as usize).min(count);(columns as f32*30.0-2.0,count.div_ceil(columns) as f32*30.0-2.0) }
        Mod::Fps => {
            let pad = if mods.fps.style == FpsStyle::Panel {
                PANEL_PADDING
            } else {
                0.0
            };
            let text = fps_text(&readings.fps, mods.fps.accuracy);
            (width(&text) + pad * 2.0, LINE_HEIGHT - 1.0 + pad * 2.0)
        }
        Mod::Keystrokes => {
            let k = &mods.keystrokes;
            let rows = k.show_mouse as u8 + (k.show_space || k.show_shift) as u8;
            (KEY * 3.0 + KEY_GAP * 2.0, KEY * 2.0 + KEY_GAP + (KEY + KEY_GAP) * rows as f32)
        },
        Mod::Armor => {
            let count = readings.armor.iter().filter(|a| !mods.armor.hide_empty || a.item.is_some()).count().max(1) as f32;
            armor_size(&mods.armor, count)
        },
        Mod::Coordinates => {
            let lines = configured_coordinate_lines(readings, &mods.coordinates);
            let pad = if mods.coordinates.background { PANEL_PADDING } else { 0.0 };
            (lines.iter().map(|line| width(line)).fold(0.0, f32::max) + pad * 2.0, LINE_HEIGHT * lines.len() as f32 + pad * 2.0)
        }
    }
}
/// Where a mod sits on a `width` x `height` screen, `[x, y, w, h]` with its
/// scale applied, kept on the screen whatever its place says.
pub fn place_mod(
    atlas: &HudAtlas,
    mods: &Mods,
    readings: &ModReadings,
    m: Mod,
    (width, height): (f32, f32),
) -> [f32; 4] {
    let place = mods.place(m);
    let (w, h) = mod_size(atlas, mods, readings, m);
    let (w, h) = (w * place.scale, h * place.scale);
    if m == Mod::Armor && place.position == Anchor::HotbarLeft {
        let offhand = if readings.left_offhand { 29.0 } else { 0.0 };
        let x = (width / 2.0).floor() - 91.0 - offhand - 4.0 - w;
        // Leave the vanilla bars clear when the screen is too narrow.
        let bottom = if x < 0.0 { height - 50.0 } else { height };
        return [x.max(0.0), (bottom - h).max(0.0), w, h];
    }
    let x = along(place.position.horizontal(), place.x, w, width);
    let y = along(place.position.vertical(), place.y, h, height);
    [
        x.clamp(0.0, (width - w).max(0.0)),
        y.clamp(0.0, (height - h).max(0.0)),
        w,
        h,
    ]
}
/// Where a box of `size` starts on a `room` long side, kept to `side`.
fn along(side: i8, offset: f32, size: f32, room: f32) -> f32 {
    match side {
        -1 => MOD_MARGIN + offset,
        1 => room - MOD_MARGIN - size - offset,
        _ => room / 2.0 + offset - size / 2.0,
    }
}
/// The distance from `side` of a box at `start`: what `along` takes back.
fn offset(side: i8, start: f32, size: f32, room: f32) -> f32 {
    match side {
        -1 => start - MOD_MARGIN,
        1 => room - MOD_MARGIN - size - start,
        _ => start + size / 2.0 - room / 2.0,
    }
}
/// Which side a box keeps to: the third of the screen its middle is in.
fn side(start: f32, size: f32, room: f32) -> i8 {
    let middle = start + size / 2.0;
    if middle < room / 3.0 {
        -1
    } else if middle > room * 2.0 / 3.0 {
        1
    } else {
        0
    }
}
/// Gives a mod whose box is at `[x, y, w, h]` what is nearest to it of the
/// corners, the edges' middles and the middle, and its distance from there.
pub fn move_mod(place: &mut Place, [x, y, w, h]: [f32; 4], (width, height): (f32, f32)) {
    let (horizontal, vertical) = (side(x, w, width), side(y, h, height));
    place.position = Anchor::from_sides(horizontal, vertical);
    // A box let go next to its margin, or the middle, lines up with it.
    let snap = |offset: f32| {
        if offset.abs() < 3.0 {
            0.0
        } else {
            (offset * 2.0).round() / 2.0
        }
    };
    place.x = snap(offset(horizontal, x, w, width));
    place.y = snap(offset(vertical, y, h, height));
}

/// The game's readings of the last frame, for the HUD editor over a world.
static LIVE: std::sync::Mutex<Option<ModReadings>> = std::sync::Mutex::new(None);
/// When the HUD editor last drew: the game leaves the mods to it meanwhile.
static EDITOR: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
/// Keep this frame's readings for the editor.
pub fn remember(readings: &ModReadings) {
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = Some(readings.clone());
}
/// The last frame's readings, while a world is open; the game clears them.
pub fn live() -> Option<ModReadings> {
    LIVE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}
pub fn forget_live() {
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = None;
}
/// The editor drew this frame.
pub fn editor_drawn() {
    *EDITOR.lock().unwrap_or_else(|e| e.into_inner()) = Some(std::time::Instant::now());
}
/// Whether the editor is on screen: it drew within the last few frames.
pub fn editor_open() -> bool {
    EDITOR
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_some_and(|at| at.elapsed() < std::time::Duration::from_millis(200))
}
fn draw_key(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    (x, y, w): (f32, f32, f32),
    label: &str,
    down: bool,
) {
    batch.fill(atlas, x, y, w, KEY, if down { KEY_DOWN } else { KEY_IDLE });
    let tw = HudBatch::text_width(atlas, label).ceil();
    let scale = ((w - 4.0) / tw.max(1.0)).min(1.0);
    let tx = x + ((w - tw * scale + 1.0) / 2.0).floor();
    let ty = y + ((KEY - LINE_HEIGHT * scale + 2.0) / 2.0).floor();
    let start = batch.vertices.len();
    batch.text(atlas, label, 0.0, 0.0, TEXT, false);
    for vertex in &mut batch.vertices[start..] {
        vertex.pos = [tx + vertex.pos[0] * scale, ty + vertex.pos[1] * scale];
    }
}
/// W, then A S D, then the mouse buttons.
fn draw_keystrokes(batch: &mut HudBatch, atlas: &HudAtlas, keys: [bool; 8], options: &crate::client_settings::KeysMod) {
    let step = KEY + KEY_GAP;
    let label = |s: &str| if options.show_labels { s.to_owned() } else { String::new() };
    draw_key(batch, atlas, (step, 0.0, KEY), &label("W"), keys[0]);
    for (i, name) in ["A", "S", "D"].iter().enumerate() { draw_key(batch, atlas, (step * i as f32, step, KEY), &label(name), keys[i + 1]); }
    let mut y = step * 2.0;
    if options.show_mouse {
        let half = ((KEY * 3.0 + KEY_GAP) / 2.0).floor();
        draw_key(batch, atlas, (0.0, y, half), &label(&lang::get_or("patina.client.lmb", "LMB")), keys[4]);
        draw_key(batch, atlas, (half + KEY_GAP, y, half), &label(&lang::get_or("patina.client.rmb", "RMB")), keys[5]);
        y += step;
    }
    let width = KEY * 3.0 + KEY_GAP * 2.0;
    let half = (width - KEY_GAP) / 2.0;
    if options.show_shift {
        draw_key(batch, atlas, (0.0, y, if options.show_space { half } else { width }), &label("SHIFT"), keys[7]);
    }
    if options.show_space {
        let (x, w) = if options.show_shift { (half + KEY_GAP, half) } else { (0.0, width) };
        draw_key(batch, atlas, (x, y, w), &label(&lang::get_or("patina.key.space", "SPACE")), keys[6]);
    }
}

pub fn configured_coordinate_lines(r: &ModReadings, options: &crate::client_settings::CoordinatesMod) -> Vec<String> {
    let [position, direction] = coordinate_lines(r.position, r.yaw);
    let mut lines = vec![position];
    let yaw = (r.yaw + 180.0).rem_euclid(360.0) - 180.0;
    let mut orientation = if options.show_angle { format!("{yaw:.1}° / {:.1}°", r.pitch) } else { String::new() };
    if options.show_direction {
        if !orientation.is_empty() { orientation.push_str("  "); }
        orientation.push_str(&direction);
    }
    if !orientation.is_empty() { lines.push(orientation); }
    if options.show_biome { lines.push(format!("{}: {}", lang::get_or("patina.option.biome", "Biome"), r.biome)); }
    lines
}

const ARMOR_SLOT: f32 = 22.0;
const ARMOR_ALERT: f32 = 12.0;
const EMPTY_ARMOR: [&str; 4] = [
    "gui/sprites/container/slot/helmet",
    "gui/sprites/container/slot/chestplate",
    "gui/sprites/container/slot/leggings",
    "gui/sprites/container/slot/boots",
];

fn armor_size(options: &crate::client_settings::ArmorMod, count: f32) -> (f32, f32) {
    let alert = if options.warning { ARMOR_ALERT } else { 0.0 };
    let numbers = options.display != "bar";
    if options.vertical {
        (alert + ARMOR_SLOT + if numbers { 28.0 } else { 0.0 }, count * ARMOR_SLOT)
    } else {
        (count * 20.0 + 2.0, alert + ARMOR_SLOT + if numbers { 10.0 } else { 0.0 })
    }
}

/// The armor slots are the interactive part of the editor.
pub fn armor_slots([x, y, w, h]: [f32; 4], armor: &crate::client_settings::ArmorMod) -> [f32; 4] {
    let slot = ARMOR_SLOT * armor.place.scale;
    if armor.vertical {
        [x + if armor.warning { ARMOR_ALERT * armor.place.scale } else { 0.0 }, y, slot, h]
    } else {
        [x, y + h - slot, w, slot]
    }
}

fn draw_armor_frame(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, count: usize, color: u32) {
    let Some(sprite) = atlas.sprites.get("gui/sprites/hud/hotbar").copied() else { return };
    let step = (sprite.u1 - sprite.u0) / 182.0;
    let body = count as f32 * 20.0;
    let left = Rect { u1: sprite.u0 + step * body, ..sprite };
    let right = Rect { u0: sprite.u0 + step * 180.0, ..sprite };
    batch.rect_uv(x, y, body, ARMOR_SLOT, left, color);
    batch.rect_uv(x + body, y, 2.0, ARMOR_SLOT, right, color);
    let right = x + body;
    let copper_light = rgba(217, 157, 116, 255);
    let copper = rgba(166, 108, 76, 255);
    let copper_shade = rgba(121, 81, 59, 255);
    batch.fill(atlas, x + 1.0, y + 1.0, body, 1.0, copper_light);
    batch.fill(atlas, x + 1.0, y + 2.0, body, 1.0, copper);
    batch.fill(atlas, x + 1.0, y + 3.0, 1.0, 17.0, copper);
    batch.fill(atlas, right, y + 3.0, 1.0, 17.0, copper_shade);
    batch.fill(atlas, x + 2.0, y + 3.0, 1.0, 16.0, copper_shade);
    batch.fill(atlas, right - 1.0, y + 3.0, 1.0, 16.0, copper);
    for slot in 1..count {
        let seam = x + slot as f32 * 20.0;
        batch.fill(atlas, seam - 1.0, y + 3.0, 4.0, 16.0, copper);
        batch.fill(atlas, seam, y + 3.0, 1.0, 16.0, copper_shade);
        batch.fill(atlas, seam + 1.0, y + 3.0, 1.0, 16.0, copper_light);
    }
    batch.fill(atlas, x + 2.0, y + 19.0, body - 2.0, 1.0, copper);
    batch.fill(atlas, x + 1.0, y + 20.0, body, 1.0, copper_shade);
}

fn draw_armor_alert(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32) {
    let outline = rgba(101, 28, 25, 255);
    let red = rgba(227, 75, 61, 255);
    let white = rgba(255, 244, 233, 255);
    for row in 0..9 {
        let inset = ((8 - row) / 2) as f32;
        let top = y + row as f32;
        batch.fill(atlas, x + inset, top, 9.0 - 2.0 * inset, 1.0, outline);
        if (2..8).contains(&row) {
            batch.fill(atlas, x + inset + 1.0, top, 7.0 - 2.0 * inset, 1.0, red);
        }
    }
    batch.fill(atlas, x + 4.0, y + 3.0, 1.0, 3.0, white);
    batch.fill(atlas, x + 4.0, y + 7.0, 1.0, 1.0, white);
}

fn draw_armor(batch: &mut HudBatch, atlas: &HudAtlas, readings: &[ArmorReading; 4], options: &crate::client_settings::ArmorMod) {
    let pieces = readings.iter().enumerate().filter(|(_, piece)| !options.hide_empty || piece.item.is_some());
    let count = pieces.clone().count();
    if count == 0 { return; }
    let alert = if options.warning { ARMOR_ALERT } else { 0.0 };
    let tint = rgba(226, 238, 233, 255);
    let frame_y = alert + if options.display == "bar" { 0.0 } else { 10.0 };
    if !options.vertical { draw_armor_frame(batch, atlas, 0.0, frame_y, count, tint); }
    for (shown, (slot, piece)) in pieces.enumerate() {
        let (x, y) = if options.vertical { (alert, shown as f32 * ARMOR_SLOT) } else { (shown as f32 * 20.0, frame_y) };
        if options.vertical { draw_armor_frame(batch, atlas, x, y, 1, tint); }
        let Some(item) = &piece.item else {
            batch.sprite_scaled(atlas, EMPTY_ARMOR[slot], x + 3.0, y + 3.0, 16.0, 16.0, rgba(255, 255, 255, 85));
            continue;
        };
        let (namespace, path) = item.split_once(':').unwrap_or(("minecraft", item.as_str()));
        let sprite = if namespace == "minecraft" { format!("item/{path}") } else { format!("{namespace}:item/{path}") };
        batch.sprite_scaled(atlas, &sprite, x + 3.0, y + 3.0, 16.0, 16.0, u32::MAX);
        if piece.maximum <= 0 { continue; }
        let remaining = piece.remaining.clamp(0, piece.maximum);
        let ratio = remaining as f32 / piece.maximum as f32;
        let damage = piece.maximum - remaining;
        let damage_bar = crate::hud_overlays::damage_bar(damage, piece.maximum);
        let color = if options.dynamic_color {
            let argb = damage_bar.map_or(0x0000ff00, |(_, color)| color);
            let [_, r, g, b] = argb.to_be_bytes();
            rgba(r, g, b, 255)
        } else { u32::MAX };
        if options.display == "bar" {
            if let Some((width, _)) = damage_bar {
                batch.fill(atlas, x + 5.0, y + 16.0, 13.0, 2.0, rgba(0, 0, 0, 255));
                batch.fill(atlas, x + 5.0, y + 16.0, width.max(1) as f32, 1.0, color);
            }
        } else {
            let text = remaining.to_string();
            let width = HudBatch::text_width(atlas, &text);
            let scale = (if options.vertical { 26.0 } else { 18.0 } / width.max(1.0)).min(1.0);
            let (tx, ty) = if options.vertical { (x + 24.0, y + 7.0) }
                else { (x + 11.0 - width * scale / 2.0, y - 9.0) };
            let start = batch.vertices.len();
            batch.text(atlas, &text, 0.0, 0.0, color, true);
            for vertex in &mut batch.vertices[start..] {
                vertex.pos = [tx + vertex.pos[0] * scale, ty + vertex.pos[1] * scale];
            }
        }
        if options.warning && ratio * 100.0 <= options.warning_threshold as f32 {
            let (ax, ay) = if options.vertical { (x - 11.0, y + 7.0) } else { (x + 7.0, alert - 10.0) };
            draw_armor_alert(batch, atlas, ax, ay);
        }
    }
}
/// Draws one mod into its box `rect`; `alpha` fades it, for a mod that is off
/// in the settings preview.
pub fn draw_mod(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    mods: &Mods,
    readings: &ModReadings,
    m: Mod,
    rect: [f32; 4],
    alpha: f32,
) {
    let start = batch.vertices.len();
    match m {
        Mod::Speedometer | Mod::Ping => { let pad=if if m==Mod::Speedometer {mods.speedometer.background}else{mods.ping.background}{3.0}else{0.0}; if pad>0.0 {let(w,h)=mod_size(atlas,mods,readings,m);batch.fill(atlas,0.0,0.0,w,h,PANEL);} batch.text(atlas,&info_text(m,mods,readings),pad,pad,TEXT,true); }
        Mod::Effects => {
            let columns=(mods.status_effects.columns as usize).min(readings.effects.len().max(1));
            for(i,effect)in readings.effects.iter().enumerate(){let(x,y)=((i%columns)as f32*30.0,(i/columns)as f32*30.0);batch.fill(atlas,x+1.0,y,26.0,28.0,rgba(102,104,106,255));batch.fill(atlas,x,y+1.0,28.0,26.0,rgba(102,104,106,255));batch.fill(atlas,x+1.0,y+1.0,26.0,26.0,rgba(16,17,18,176));if let Some(sprite)=atlas.sprites.get(&format!("mob_effect/{}",effect.id)).copied(){batch.rect_uv(x+5.0,y+5.0,18.0,18.0,sprite,u32::MAX);}if mods.status_effects.show_amplifier{let text=effect_roman(effect.amplifier+1);batch.text(atlas,&text,x+(28.0-HudBatch::text_width(atlas,&text))/2.0,y+1.0,u32::MAX,true);}if mods.status_effects.show_duration{let text=effect_duration(effect.ticks);batch.text(atlas,&text,x+(28.0-HudBatch::text_width(atlas,&text))/2.0,y+19.0,u32::MAX,true);}}
        }
        Mod::Fps => {
            let text = fps_text(&readings.fps, mods.fps.accuracy);
            let panel = mods.fps.style == FpsStyle::Panel;
            let pad = if panel { PANEL_PADDING } else { 0.0 };
            if panel {
                let (w, h) = mod_size(atlas, mods, readings, m);
                batch.fill(atlas, 0.0, 0.0, w, h, PANEL);
            }
            batch.text(atlas, &text, pad, pad, TEXT, !panel);
        }
        Mod::Keystrokes => draw_keystrokes(batch, atlas, readings.keys, &mods.keystrokes),
        Mod::Armor => draw_armor(batch, atlas, &readings.armor, &mods.armor),
        Mod::Coordinates => {
            let pad = if mods.coordinates.background { PANEL_PADDING } else { 0.0 };
            if pad > 0.0 { let (w, h) = mod_size(atlas, mods, readings, m); batch.fill(atlas, 0.0, 0.0, w, h, PANEL); }
            for (i, line) in configured_coordinate_lines(readings, &mods.coordinates).iter().enumerate() {
                batch.text(atlas, line, pad, pad + if pad > 0.0 { 1.0 } else { 0.0 } + i as f32 * LINE_HEIGHT, if i == 0 { TEXT } else { DIM_TEXT }, pad == 0.0);
            }
        }
    }
    let scale = mods.place(m).scale;
    for v in &mut batch.vertices[start..] {
        v.pos = [rect[0] + v.pos[0] * scale, rect[1] + v.pos[1] * scale];
    }
    if alpha < 1.0 {
        batch.fade_since(start, alpha);
    }
}
/// The enabled built-in mods, each in its place. The coordinates only show
/// with `position` (no Reduced Debug Info).
pub fn draw_mods(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    mods: &Mods,
    readings: &ModReadings,
    position: bool,
) {
    let screen = (batch.width, batch.height);
    for m in Mod::ALL {
        if mods.enabled(m) && (m != Mod::Coordinates || position) {
            let rect = place_mod(atlas, mods, readings, m, screen);
            draw_mod(batch, atlas, mods, readings, m, rect, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_readings() -> ModReadings {
        ModReadings { fps: FpsReading { fps: 144, exact: 143.6, min: 90, low: 91 }, keys: [false; 8], position: [128.0, 64.0, -256.0], yaw: 450.0, pitch: 12.0, biome: "Plains".into(), left_offhand: false, armor: sample_armor(), speed: 4.3, ping: 42, effects: sample_effects() }
    }
    #[test]
    fn coordinates_include_only_the_selected_information() {
        let r = sample_readings();
        let mut options = crate::client_settings::CoordinatesMod::default();
        options.show_direction = false;
        assert_eq!(configured_coordinate_lines(&r, &options), vec!["128 64 -256"]);
        options.show_biome = true;
        options.show_angle = true;
        let lines = configured_coordinate_lines(&r, &options);
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("90.0° / 12.0°"));
        assert!(lines[2].ends_with("Plains"));
    }
    #[test]
    fn optional_key_rows_change_the_hud_hitbox() {
        let atlas = HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap();
        let mut mods = Mods::default();
        let r = sample_readings();
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Keystrokes), (58.0, 78.0));
        mods.keystrokes.show_space = false;
        mods.keystrokes.show_shift = false;
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Keystrokes), (58.0, 58.0));
        mods.keystrokes.show_mouse = false;
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Keystrokes), (58.0, 38.0));
    }
    #[test]
    fn armor_aligns_with_the_hotbar_and_avoids_the_offhand_slot() {
        let atlas = HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap();
        let mods = Mods::default();
        let mut readings = sample_readings();
        let placed = |r: &ModReadings| place_mod(&atlas, &mods, r, Mod::Armor, (480.0, 300.0));
        assert_eq!(placed(&readings), [63.0, 266.0, 82.0, 34.0]);
        readings.left_offhand = true;
        assert_eq!(placed(&readings), [34.0, 266.0, 82.0, 34.0]);
        assert_eq!(place_mod(&atlas, &mods, &readings, Mod::Armor, (320.0, 240.0)), [0.0, 156.0, 82.0, 34.0]);
    }

    #[test]
    fn armor_layout_hides_empty_slots_and_draws_all_display_modes() {
        let atlas = HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap();
        let mut mods = Mods::default();
        let r = sample_readings();
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Armor), (82.0, 34.0));
        mods.armor.hide_empty = true;
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Armor), (62.0, 34.0));
        mods.armor.vertical = true;
        assert_eq!(mod_size(&atlas, &mods, &r, Mod::Armor), (34.0, 66.0));
        for display in ["bar", "remaining"] {
            mods.armor.display = display.into();
            let mut batch = HudBatch::new([1280, 720]);
            let (width, height) = mod_size(&atlas, &mods, &r, Mod::Armor);
            draw_mod(&mut batch, &atlas, &mods, &r, Mod::Armor, [4.0, 4.0, width, height], 1.0);
            assert!(!batch.vertices.is_empty());
            assert!(batch.vertices.iter().all(|v| v.pos.iter().all(|p| p.is_finite())));
        }
    }

    #[test]
    fn fps_lines_follow_the_accuracy() {
        let r = FpsReading {
            fps: 120,
            exact: 119.84,
            min: 60,
            low: 88,
        };
        assert_eq!(fps_text(&r, FpsAccuracy::Whole), "120 FPS");
        assert_eq!(fps_text(&r, FpsAccuracy::Exact), "119.8 FPS  1% 88");
    }
    #[test]
    fn facing_follows_the_yaw() {
        assert_eq!(facing(0.0).2, "+Z");
        assert_eq!(facing(90.0).2, "-X");
        assert_eq!(facing(-90.0).2, "+X");
        assert_eq!(facing(180.0).2, "-Z");
        assert_eq!(facing(-181.0).2, "-Z");
    }
    #[test]
    fn a_dragged_mod_keeps_to_its_nearest_corner() {
        let mut place = Place::default();
        move_mod(&mut place, [300.0, 250.0, 50.0, 20.0], (400.0, 300.0));
        assert_eq!(place.position, Anchor::BottomRight);
        assert_eq!((place.x, place.y), (46.0, 26.0));
        // Next to the margin it lines up with it.
        move_mod(&mut place, [5.0, 2.0, 50.0, 20.0], (400.0, 300.0));
        assert_eq!(
            (place.position, place.x, place.y),
            (Anchor::TopLeft, 0.0, 0.0)
        );
        // Let go in the middle, it stays in the middle on any screen.
        move_mod(&mut place, [176.0, 141.0, 50.0, 20.0], (400.0, 300.0));
        assert_eq!(
            (place.position, place.x, place.y),
            (Anchor::Center, 0.0, 0.0)
        );
        let mut mods = Mods::default();
        mods.keystrokes.enabled = true;
        mods.keystrokes.place = place;
        let atlas = HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap();
        let readings = ModReadings {
            fps: FpsReading {
                fps: 1,
                exact: 1.0,
                min: 1,
                low: 1,
            },
            keys: [false; 8],
            position: [0.0; 3],
            yaw: 0.0,
            pitch: 0.0, biome: "Plains".into(), left_offhand: false, armor: sample_armor(), speed: 0.0, ping: 0, effects: vec![],
        };
        for screen in [(400.0, 300.0), (640.0, 360.0)] {
            let [x, y, w, h] = place_mod(&atlas, &mods, &readings, Mod::Keystrokes, screen);
            assert_eq!((x + w / 2.0, y + h / 2.0), (screen.0 / 2.0, screen.1 / 2.0));
        }
    }
    #[test]
    fn corners_stack_away_from_their_edge() {
        let mut c = Corners::new(400.0, 300.0, 26.0);
        assert_eq!(c.place(Corner::TopLeft, 50.0, 12.0), (2.0, 2.0));
        assert_eq!(c.place(Corner::TopLeft, 70.0, 20.0), (2.0, 16.0));
        // Top right starts under the effect icons.
        assert_eq!(c.place(Corner::TopRight, 50.0, 12.0), (348.0, 28.0));
        assert_eq!(c.place(Corner::BottomRight, 50.0, 12.0), (348.0, 286.0));
        assert_eq!(c.place(Corner::BottomRight, 50.0, 10.0), (348.0, 274.0));
        assert_eq!(c.place(Corner::BottomLeft, 10.0, 10.0), (2.0, 288.0));
    }
}
