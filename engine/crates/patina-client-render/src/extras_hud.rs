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
    pub keys: [bool; 6],
    pub position: [f64; 3],
    /// Degrees, as the player's yaw: 0 faces south.
    pub yaw: f32,
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
        Mod::Fps => {
            let pad = if mods.fps.style == FpsStyle::Panel {
                PANEL_PADDING
            } else {
                0.0
            };
            let text = fps_text(&readings.fps, mods.fps.accuracy);
            (width(&text) + pad * 2.0, LINE_HEIGHT - 1.0 + pad * 2.0)
        }
        Mod::Keystrokes => (KEY * 3.0 + KEY_GAP * 2.0, KEY * 3.0 + KEY_GAP * 2.0),
        Mod::Coordinates => {
            let [a, b] = coordinate_lines(readings.position, readings.yaw);
            (width(&a).max(width(&b)), LINE_HEIGHT * 2.0)
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
    let tx = x + ((w - tw + 1.0) / 2.0).floor();
    let ty = y + ((KEY - LINE_HEIGHT + 2.0) / 2.0).floor();
    batch.text(atlas, label, tx, ty, TEXT, false);
}
/// W, then A S D, then the mouse buttons.
fn draw_keystrokes(batch: &mut HudBatch, atlas: &HudAtlas, keys: [bool; 6]) {
    let step = KEY + KEY_GAP;
    draw_key(batch, atlas, (step, 0.0, KEY), "W", keys[0]);
    draw_key(batch, atlas, (0.0, step, KEY), "A", keys[1]);
    draw_key(batch, atlas, (step, step, KEY), "S", keys[2]);
    draw_key(batch, atlas, (step * 2.0, step, KEY), "D", keys[3]);
    let half = ((KEY * 3.0 + KEY_GAP) / 2.0).floor();
    let left = lang::get_or("patina.client.lmb", "LMB");
    let right = lang::get_or("patina.client.rmb", "RMB");
    draw_key(batch, atlas, (0.0, step * 2.0, half), &left, keys[4]);
    draw_key(
        batch,
        atlas,
        (half + KEY_GAP, step * 2.0, half),
        &right,
        keys[5],
    );
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
        Mod::Keystrokes => draw_keystrokes(batch, atlas, readings.keys),
        Mod::Coordinates => {
            let [a, b] = coordinate_lines(readings.position, readings.yaw);
            batch.text(atlas, &a, 0.0, 0.0, TEXT, true);
            batch.text(atlas, &b, 0.0, LINE_HEIGHT, DIM_TEXT, true);
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
            keys: [false; 6],
            position: [0.0; 3],
            yaw: 0.0,
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
