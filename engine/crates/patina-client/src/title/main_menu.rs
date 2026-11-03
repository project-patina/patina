//! The main title screen as the Patina Fabric mod draws it (`PatinaTitle`):
//! the logo slides up and left and shrinks, the buttons arrive from the left
//! one after another on a plane turned away to the right, and the player walks
//! in from the right. Buttons keep flat rectangles (`Layout`, `Title::buttons`);
//! `Plane` maps them onto the screen and cursors back, so what looks hit is
//! what is hit.
use super::{Button, Title};
use crate::{
    hud::{HudAtlas, HudBatch, rgba},
    skins::{self, Kind, NameOrId, Resolve},
    title_player, ui,
};
use std::{sync::OnceLock, time::Instant};

const COLUMN_WIDTH: i32 = 170;
const PITCH: i32 = 24;
const BUTTON_HEIGHT: i32 = 20;
pub(super) const ICON_SIZE: f32 = 20.0;
const ICON_GAP: f32 = 4.0;
const LOGO_SCALE: f32 = 0.72;
const LOGO_WIDTH: f32 = 256.0;
const LOGO_HEIGHT: f32 = 51.0;
const VANILLA_LOGO_Y: f32 = 30.0;
/// What the bottom left corner keeps free for the version lines.
const BOTTOM_RESERVED: i32 = 26;
/// Singleplayer, Multiplayer, Patina Client, Options and Quit, one per row.
pub(super) const COLUMN: usize = 5;

/// The flat layout of the left column.
pub(super) struct Layout {
    pub logo_left: f32,
    pub logo_top: f32,
    pub top: f32,
    pub pitch: f32,
    /// The buttons' height, under 20 only where the screen is too short.
    pub height: f32,
    pub width: f32,
    center_y: f32,
}
impl Layout {
    pub(super) fn new(w: f32, h: f32) -> Self {
        // In whole GUI pixels, as the screen's int fields are.
        let (w, h) = (w as i32, h as i32);
        let logo_left = (w / 18).max(20);
        let logo_top = (h / 14).max(8);
        let top = logo_top + (LOGO_HEIGHT * LOGO_SCALE).round() as i32 + 12;
        let rows = COLUMN as i32 + 1;
        let pitch = PITCH.min((h - BOTTOM_RESERVED - top) / rows);
        Self {
            logo_left: logo_left as f32,
            logo_top: logo_top as f32,
            top: top as f32,
            pitch: pitch as f32,
            height: BUTTON_HEIGHT.min(pitch - 2) as f32,
            width: COLUMN_WIDTH.min(w * 2 / 5) as f32,
            center_y: top as f32 + (pitch * rows) as f32 / 2.0,
        }
    }
    pub(super) fn row(&self, n: usize) -> f32 {
        self.top + n as f32 * self.pitch
    }
    /// The x of the `n`th icon button, under the column.
    pub(super) fn icon_x(&self, n: usize) -> f32 {
        self.logo_left + n as f32 * (ICON_SIZE + ICON_GAP)
    }
    pub(super) fn icon_y(&self) -> f32 {
        self.row(COLUMN) + 1.0
    }
    pub(super) fn plane(&self) -> Plane {
        Plane {
            origin_x: self.logo_left,
            center_y: self.center_y,
        }
    }
}

/// The plane the column is drawn on: turned away to the right about its left
/// edge, seen from `FOCAL` GUI pixels.
#[derive(Clone, Copy)]
pub(super) struct Plane {
    origin_x: f32,
    center_y: f32,
}
const TURN: f32 = 17.0;
/// Distance to the plane in GUI units: smaller is a stronger perspective.
const FOCAL: f32 = 440.0;
impl Plane {
    fn scale_at(u: f32) -> f32 {
        FOCAL / (FOCAL + u * TURN.to_radians().sin())
    }
    fn screen_x(&self, u: f32) -> f32 {
        self.origin_x + u * TURN.to_radians().cos() * Self::scale_at(u)
    }
    fn screen_y(&self, u: f32, y: f32) -> f32 {
        self.center_y + (y - self.center_y) * Self::scale_at(u)
    }
    /// Where the flat point `(x, y)` shows.
    pub(super) fn apply(&self, (x, y): (f32, f32)) -> (f32, f32) {
        let u = x - self.origin_x;
        (self.screen_x(u), self.screen_y(u, y))
    }
    /// The flat point a cursor at `(x, y)` is over.
    pub(super) fn undo(&self, (x, y): (f32, f32)) -> (f32, f32) {
        let (sin, cos) = TURN.to_radians().sin_cos();
        let x = x - self.origin_x;
        let u = x * FOCAL / (cos * FOCAL - x * sin);
        (
            u + self.origin_x,
            self.center_y + (y - self.center_y) / Self::scale_at(u),
        )
    }
}

/// `eased`: a quartic ease out of `time` ms, starting `delay` ms in.
fn eased(elapsed: f32, delay: f32, time: f32) -> f32 {
    let rest = 1.0 - ((elapsed - delay) / time).clamp(0.0, 1.0);
    1.0 - rest.powi(4)
}
const LOGO_DELAY: f32 = 450.0;
const LOGO_TIME: f32 = 1000.0;
const BUTTON_DELAY: f32 = 750.0;
const BUTTON_STAGGER: f32 = 55.0;
const BUTTON_TIME: f32 = 650.0;
const PLAYER_DELAY: f32 = 900.0;
const PLAYER_TIME: f32 = 800.0;
const HOVER_PUSH: f32 = 5.0;
/// What the hover glow closes of its distance in a sixtieth of a second.
const HOVER_EASE: f32 = 0.3;

/// When the title first showed in this run. Like the game's `TitleScreen`
/// fading in once after loading, only the first one plays the motion.
static INTRO: OnceLock<Instant> = OnceLock::new();
/// Milliseconds into the motion; long past it once it has played.
fn elapsed() -> f32 {
    INTRO.get_or_init(Instant::now).elapsed().as_secs_f32() * 1000.0
}
/// `TitleScreen`'s fade in: the widgets, logo and texts come in over the second
/// half of its two seconds.
fn widget_alpha(elapsed: f32) -> f32 {
    ((elapsed / 2000.0).clamp(0.0, 1.0) - 0.5).max(0.0) * 2.0
}

/// The hover glow of each button, eased frame by frame.
#[derive(Default)]
pub(super) struct Hover {
    glow: Vec<f32>,
    last: Option<Instant>,
}
impl Hover {
    fn step(&mut self, count: usize) -> f32 {
        let now = Instant::now();
        let frames = self
            .last
            .map_or(1.0, |last| (now - last).as_secs_f32() * 60.0)
            .min(60.0);
        self.last = Some(now);
        self.glow.resize(count, 0.0);
        1.0 - (1.0 - HOVER_EASE).powf(frames)
    }
}

fn mix(from: u32, to: u32, t: f32) -> u32 {
    let lerp = |shift: u32| {
        let (a, b) = ((from >> shift) & 255, (to >> shift) & 255);
        (a as f32 + (b as f32 - a as f32) * t).round() as u32
    };
    lerp(24) << 24 | lerp(16) << 16 | lerp(8) << 8 | lerp(0)
}
/// `PatinaClient.fade`.
fn fade(color: u32, alpha: f32) -> u32 {
    let a = ((color >> 24) as f32 * alpha.clamp(0.0, 1.0)).round() as u32;
    a << 24 | color & 0x00ff_ffff
}
const ACCENT: u32 = 0xFF3F_B8A0;

/// A strip of a panel from `left` to `right` along the plane, coloured from
/// its near to its far end.
#[allow(clippy::too_many_arguments)]
fn strip(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    plane: &Plane,
    [left, top, right, bottom]: [f32; 4],
    near: u32,
    far: u32,
    shift: f32,
    alpha: f32,
) {
    let (near, far) = (fade(near, alpha), fade(far, alpha));
    let near_x = plane.screen_x(left) + shift;
    let far_x = plane.screen_x(right) + shift;
    batch.fill_quad_colors(
        atlas,
        [
            [near_x, plane.screen_y(left, top)],
            [near_x, plane.screen_y(left, bottom)],
            [far_x, plane.screen_y(right, bottom)],
            [far_x, plane.screen_y(right, top)],
        ],
        [near, near, far, far].map(argb_to_rgba),
    );
}
/// `0xAARRGGBB` to the batch's colour.
fn argb_to_rgba(c: u32) -> u32 {
    rgba((c >> 16) as u8, (c >> 8) as u8, c as u8, (c >> 24) as u8)
}

/// `PatinaTitle.drawPanel`: the slab of a button, its edges lit in the accent
/// as it glows.
fn panel(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    plane: &Plane,
    b: &Button,
    glow: f32,
    shift: f32,
    alpha: f32,
) {
    let left = b.x - plane.origin_x;
    let right = left + b.w;
    let (top, bottom) = (b.y, b.y + b.h);
    let mut strip = |rect: [f32; 4], near: u32, far: u32| {
        strip(batch, atlas, plane, rect, near, far, shift, alpha);
    };
    let active = b.enabled;
    let near = if active {
        mix(0xD012_1A1C, 0xE017_4A44, glow)
    } else {
        0x8010_1416
    };
    let far = if active {
        mix(0xA80A_0F11, 0xC00F_2B28, glow)
    } else {
        0x6010_1214
    };
    strip([left, top, right, bottom], near, far);
    // The underside gives the panel its thickness.
    strip(
        [left, bottom, right, bottom + 2.0],
        0xD004_0607,
        0xB004_0607,
    );
    let edge = if active {
        mix(0x38FF_FFFF, ACCENT, glow)
    } else {
        0x18FF_FFFF
    };
    strip([left, top, right, top + 1.0], edge, edge);
    let under = mix(0x14FF_FFFF, ACCENT, glow);
    strip([left, bottom - 1.0, right, bottom], under, under);
    strip([left, top, left + 1.0 + glow, bottom], edge, edge);
    strip([right - 1.0, top, right, bottom], edge, edge);
}

/// `PatinaTitle.beginWidget`: the label laid on the plane, an affine fit of
/// the projection at the button's middle.
fn label(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    plane: &Plane,
    b: &Button,
    shift: f32,
    alpha: f32,
) {
    let start = batch.vertices.len();
    let white = fade(if b.enabled { 0xFFFF_FFFF } else { 0xFFA0_A0A0 }, alpha);
    if let Some((icon, size)) = b.icon {
        // CenteredIcon: a 15px icon in a 20px button starts at 10 - 7 = 3.
        batch.sprite_scaled(
            atlas,
            &format!("gui/sprites/{icon}"),
            b.x + (b.w / 2.0).floor() - (size / 2.0).floor(),
            b.y + (b.h / 2.0).floor() - (size / 2.0).floor(),
            size,
            size,
            argb_to_rgba(fade(0xFFFF_FFFF, alpha)),
        );
    } else {
        // AbstractButton's label: centred between 2px insets, a pixel under the middle.
        let width = HudBatch::text_width(atlas, &b.label).ceil();
        let x = ((2.0 * b.x + b.w) / 2.0).floor() - (width / 2.0).floor();
        let y = ((2.0 * b.y + b.h - 9.0) / 2.0).floor() + 1.0;
        batch.text(atlas, &b.label, x, y, argb_to_rgba(white), true);
    }
    let u = b.x + b.w / 2.0 - plane.origin_x;
    let y = b.y + b.h / 2.0;
    let across = plane.screen_x(u + 1.0) - plane.screen_x(u);
    let slope = plane.screen_y(u + 1.0, y) - plane.screen_y(u, y);
    let down = Plane::scale_at(u);
    let center_x = u + plane.origin_x;
    let (sx, sy) = (plane.screen_x(u) + shift, plane.screen_y(u, y));
    for v in &mut batch.vertices[start..] {
        let [px, py] = v.pos;
        v.pos = [
            sx + across * (px - center_x),
            sy + slope * (px - center_x) + down * (py - y),
        ];
    }
}

/// The account's skin, looked up from the services once the title is up.
#[derive(Default)]
pub(super) struct Skin {
    profiles: Option<skins::Resolver>,
    downloads: Option<skins::Downloader>,
    url: Option<String>,
    pixels: Option<Vec<u8>>,
    slim: bool,
    /// Nothing more to look for: no account, or the account has no skin.
    done: bool,
}
impl Skin {
    fn poll(&mut self) {
        if self.pixels.is_some() || self.done {
            return;
        }
        let Some(auth) = crate::auth::current() else {
            self.done = true;
            return;
        };
        if self.url.is_none() {
            let profiles = self.profiles.get_or_insert_with(skins::Resolver::new);
            match profiles.state(&NameOrId::Id(auth.uuid)) {
                Resolve::Done(Some(profile)) => {
                    let unpacked = profile
                        .textures()
                        .map(|(value, signature)| {
                            skins::unpack(&value, signature, &skins::TEXTURE_URIS)
                        })
                        .filter(|t| t.skin.is_some());
                    let Some(textures) = unpacked else {
                        self.done = true;
                        return;
                    };
                    self.slim = textures.slim;
                    self.url = textures.skin;
                }
                Resolve::Done(None) => self.done = true,
                _ => {}
            }
        }
        let Some(url) = self.url.clone() else {
            return;
        };
        let downloads = self.downloads.get_or_insert_with(|| {
            skins::Downloader::new(
                std::env::current_dir()
                    .unwrap_or_default()
                    .join("assets/skins"),
            )
        });
        match downloads.state(Kind::Skin, &url) {
            skins::Download::Failed => self.done = true,
            _ => {
                let wanted = skins::texture_hash(&url).map(|hash| skins::skin_location(&hash));
                self.pixels = downloads
                    .take_ready()
                    .into_iter()
                    .find(|(location, _)| Some(location) == wanted.as_ref())
                    .map(|(_, rgba)| rgba);
            }
        }
    }
}

impl Title {
    /// The id of the enabled button of the main screen a cursor at `cursor`
    /// is over (`PatinaTitle.hit`).
    pub(super) fn main_hit(&self, w: f32, h: f32, cursor: (f32, f32)) -> Option<u8> {
        let (x, y) = Layout::new(w.ceil(), h.ceil()).plane().undo(cursor);
        self.buttons(w, h)
            .into_iter()
            .find(|b| b.enabled && ui::hovered((x, y), b.x, b.y, b.w, b.h))
            .map(|b| b.id)
    }
    pub(super) fn draw_main(
        &self,
        atlas: &HudAtlas,
        batch: &mut HudBatch,
        cursor: (f32, f32),
        hide_splash: bool,
    ) {
        let (w, h) = (batch.width.ceil(), batch.height.ceil());
        let layout = Layout::new(w, h);
        let plane = layout.plane();
        let elapsed = elapsed();
        let alpha = widget_alpha(elapsed);
        // Darkens the side the buttons are on, so they read over any picture.
        let right = w * 0.62;
        batch.fill_quad_colors(
            atlas,
            [[0.0, 0.0], [0.0, h], [right, h], [right, 0.0]],
            [rgba(0, 0, 0, 0x88), rgba(0, 0, 0, 0x88), 0, 0],
        );
        // The logo and splash where vanilla draws them, then moved to where
        // the logo is heading (`PatinaTitle.pushLogo`).
        let start = batch.vertices.len();
        let vanilla_left = (w / 2.0).floor() - 128.0;
        // LogoRenderer: the logo's top 44 of 64 rows, then the edition
        // banner's top 14 of 16 rows overlapping its bottom by 7.
        batch.sprite_region(
            atlas,
            "gui/title/minecraft",
            [vanilla_left, VANILLA_LOGO_Y, 256.0, 44.0],
            256.0,
            64.0,
        );
        batch.sprite_region(
            atlas,
            crate::title_edition::SPRITE,
            [
                (w / 2.0).floor() - 64.0,
                VANILLA_LOGO_Y + 44.0 - 7.0,
                128.0,
                14.0,
            ],
            128.0,
            16.0,
        );
        if !hide_splash && let Some(splash) = self.splash.as_deref() {
            let tw = HudBatch::text_width(atlas, splash);
            let start = batch.vertices.len();
            batch.text(
                atlas,
                splash,
                -(tw / 2.0).floor(),
                -8.0,
                rgba(255, 255, 0, 255),
                true,
            );
            let scale = (1.8
                - (self.started.elapsed().as_secs_f32() * std::f32::consts::TAU)
                    .sin()
                    .abs()
                    * 0.1)
                * 100.0
                / (tw + 32.0);
            let (sin, cos) = (-20.0f32).to_radians().sin_cos();
            for v in &mut batch.vertices[start..] {
                let [x, y] = v.pos;
                v.pos = [
                    w / 2.0 + 123.0 + scale * (x * cos - y * sin),
                    69.0 + scale * (x * sin + y * cos),
                ];
            }
        }
        let t = eased(elapsed, LOGO_DELAY, LOGO_TIME);
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        let center = w / 2.0 - LOGO_WIDTH / 2.0;
        let (left, top, scale) = (
            lerp(center, layout.logo_left),
            lerp(VANILLA_LOGO_Y, layout.logo_top),
            lerp(1.0, LOGO_SCALE),
        );
        for v in &mut batch.vertices[start..] {
            let [x, y] = v.pos;
            v.pos = [
                left + scale * (x - center),
                top + scale * (y - VANILLA_LOGO_Y),
            ];
        }
        batch.fade_since(start, alpha);

        let buttons = self.buttons(w, h);
        let mut hover = self.hover.borrow_mut();
        let step = hover.step(buttons.len());
        let flat = plane.undo(cursor);
        for (order, b) in buttons.iter().enumerate() {
            let inside = ui::hovered(flat, b.x, b.y, b.w, b.h);
            crate::widgets::widget_cursor(inside, b.enabled);
            let corners = [
                (b.x, b.y),
                (b.x + b.w, b.y),
                (b.x + b.w, b.y + b.h),
                (b.x, b.y + b.h),
            ]
            .map(|c| plane.apply(c));
            let (x0, y0) = corners
                .iter()
                .fold((f32::MAX, f32::MAX), |m, c| (m.0.min(c.0), m.1.min(c.1)));
            let (x1, y1) = corners
                .iter()
                .fold((f32::MIN, f32::MIN), |m, c| (m.0.max(c.0), m.1.max(c.1)));
            crate::widgets::record(&b.label, [x0, y0, x1 - x0, y1 - y0], b.enabled);
            let glow = &mut hover.glow[order];
            *glow += (f32::from(u8::from(inside)) - *glow) * step;
            let arrived = eased(
                elapsed,
                BUTTON_DELAY + order as f32 * BUTTON_STAGGER,
                BUTTON_TIME,
            );
            let offset = -(1.0 - arrived) * (layout.logo_left + COLUMN_WIDTH as f32 + 24.0);
            let shift = offset + *glow * HOVER_PUSH;
            panel(batch, atlas, &plane, b, *glow, shift, alpha);
            label(batch, atlas, &plane, b, shift, alpha);
        }
        drop(hover);
        self.draw_player(batch, atlas, (w, h), cursor, elapsed);
        // The game's version line, and the client's above it in the Patina teal.
        let white = argb_to_rgba(fade(0xFFFF_FFFF, alpha));
        let minecraft = format!("Minecraft {}", patina_core::VANILLA_VERSION);
        batch.text(atlas, &minecraft, 2.0, h - 10.0, white, true);
        let client = format!("Patina Client {}", patina_core::release::VERSION);
        let teal = argb_to_rgba(fade(0xFF6F_DCC3, alpha));
        batch.text(atlas, &client, 2.0, h - 20.0, teal, true);
        // A nightly names its build in the top-left corner, the one spot no button reaches at any GUI
        // scale, so screenshots and reports say which build they come from.
        if let Some(nightly) = patina_core::release::nightly_label() {
            batch.text(atlas, &nightly, 2.0, 2.0, u32::MAX, true);
        }
        let notice = "Not affiliated with Mojang or Microsoft";
        let tw = HudBatch::text_width(atlas, notice);
        batch.text(atlas, notice, w - tw - 2.0, h - 10.0, white, true);
    }
    /// `PatinaTitle.drawPlayer`: the player's own skin standing still in a box
    /// on the right; only the head moves, towards the cursor.
    fn draw_player(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        (w, h): (f32, f32),
        cursor: (f32, f32),
        elapsed: f32,
    ) {
        let (w, h) = (w as i32, h as i32);
        let box_height = h * 3 / 4;
        let box_width = box_height * 3 / 5;
        let rest = w * 3 / 4 - box_width / 2;
        let away = (1.0 - eased(elapsed, PLAYER_DELAY, PLAYER_TIME)) * (w - rest + 8) as f32;
        let x = rest + away.round() as i32;
        let y = (h - box_height) * 3 / 5;
        if x >= w {
            return;
        }
        let mut skin = self.skin.borrow_mut();
        skin.poll();
        let default = crate::auth::current().map_or("entity/player/wide/steve", |auth| {
            crate::tab_list::default_skin(auth.uuid)
        });
        let (rect, slim) = match (&skin.pixels, atlas.sprites.get(title_player::SKIN_SPRITE)) {
            (Some(pixels), Some(rect)) => {
                batch.patches.push(crate::font::Patch {
                    x: (rect.u0 * atlas.width as f32).round() as u32,
                    y: (rect.v0 * atlas.height as f32).round() as u32,
                    width: 64,
                    height: 64,
                    rgba: pixels.clone(),
                });
                (Some(*rect), skin.slim)
            }
            _ => (
                atlas.sprites.get(default).copied(),
                default.contains("/slim/"),
            ),
        };
        let Some(rect) = rect else {
            return;
        };
        let (bx, by, bw, bh) = (x as f32, y as f32, box_width as f32, box_height as f32);
        let head = (bx + bw / 2.0, by + bh * 0.13);
        // The model's own turn is taken out, so the head faces the cursor
        // rather than a point beside it.
        let yaw = (-((cursor.0 - head.0) / HEAD_REACH).atan()).clamp(-HEAD_YAW, HEAD_YAW)
            + MODEL_TURN.to_radians();
        let pitch = ((cursor.1 - head.1) / HEAD_REACH)
            .atan()
            .clamp(-HEAD_PITCH, HEAD_PITCH);
        let pose = title_player::Pose {
            scale: MODEL_FIT * bh / MODEL_HEIGHT,
            tilt: MODEL_TILT,
            turn: MODEL_TURN,
            pivot: MODEL_PIVOT,
            head_yaw: yaw,
            head_pitch: pitch,
        };
        title_player::draw(batch, rect, slim, [bx, by, bw, bh], pose);
    }
}
const MODEL_HEIGHT: f32 = 2.125;
const MODEL_FIT: f32 = 0.97;
const MODEL_PIVOT: f32 = -1.0625;
const MODEL_TILT: f32 = -3.0;
const MODEL_TURN: f32 = -12.0;
const HEAD_YAW: f32 = 0.8;
const HEAD_PITCH: f32 = 0.42;
const HEAD_REACH: f32 = 90.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_plane_maps_cursors_back_exactly() {
        for (w, h) in [(427.0, 240.0), (640.0, 360.0), (960.0, 540.0)] {
            let plane = Layout::new(w, h).plane();
            for x in [20.0, 60.0, 120.0, 170.0] {
                for y in [70.0, 100.0, 150.0, 190.0] {
                    let (fx, fy) = plane.undo(plane.apply((x, y)));
                    assert!((fx - x).abs() < 0.01 && (fy - y).abs() < 0.01, "{x} {y}");
                }
            }
        }
    }
    #[test]
    fn the_layout_is_the_fabric_mods() {
        // PatinaTitle.arrange at 427 x 240: logo at (23, 17), rows of 24 from 66.
        let l = Layout::new(427.0, 240.0);
        assert_eq!((l.logo_left, l.logo_top, l.top), (23.0, 17.0, 66.0));
        assert_eq!((l.pitch, l.height, l.width), (24.0, 20.0, 170.0));
        assert_eq!(l.icon_y(), 66.0 + 5.0 * 24.0 + 1.0);
        // A short screen packs the rows tighter.
        let short = Layout::new(320.0, 200.0);
        assert!(short.pitch < 24.0 && short.height == short.pitch - 2.0);
    }
    #[test]
    fn the_motion_ends_with_everything_in_place() {
        assert_eq!(eased(0.0, LOGO_DELAY, LOGO_TIME), 0.0);
        assert_eq!(eased(5000.0, LOGO_DELAY, LOGO_TIME), 1.0);
        assert_eq!(widget_alpha(0.0), 0.0);
        assert_eq!(widget_alpha(1000.0), 0.0);
        assert_eq!(widget_alpha(1500.0), 0.5);
        assert_eq!(widget_alpha(9000.0), 1.0);
    }
}
