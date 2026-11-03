//! The Patina Client screen, as the Patina Fabric mod draws it
//! (`PatinaSettingsScreen`): a small panel with each mod and its switch, the
//! FPS counter's options and the way into the HUD editor
//! (`PatinaHudEditorScreen`), where the mods are placed on the game itself, at
//! its real size, by the code that draws them in game. Drawing and clicks run
//! the same layout, so they never disagree.
use crate::{
    client_settings::{self, FpsAccuracy, FpsStyle, MAX_SCALE, MIN_SCALE, Mod, Mods},
    extras::FpsReading,
    extras_hud::{self, ModReadings},
    hud::{HudAtlas, HudBatch, rgba},
    lang,
    text::Text,
    ui, widgets,
};

const PANEL_WIDTH: i32 = 280;
const PADDING: i32 = 8;
const TITLE: i32 = 14;
const ROW: i32 = 20;
const ROW_GAP: i32 = 2;
const CAPTION: i32 = 11;
const SECTION_GAP: i32 = 6;
const PILL_WIDTH: f32 = 24.0;
const PILL_HEIGHT: f32 = 12.0;
const KNOB: f32 = 10.0;
const PANEL: u32 = rgba(12, 16, 18, 0xB8);
const LINE: u32 = rgba(255, 255, 255, 0x30);
const HEADING: u32 = rgba(0x6F, 0xDC, 0xC3, 255);
const HINT: u32 = rgba(0x9F, 0xB0, 0xAC, 255);
const FILL: u32 = rgba(12, 16, 18, 160);
const FILL_HOVER: u32 = rgba(20, 28, 30, 192);
const BORDER: u32 = rgba(255, 255, 255, 96);
const PILL_OFF: u32 = rgba(255, 255, 255, 0x40);
const KNOB_COLOR: u32 = rgba(0xF0, 0xF3, 0xF3, 255);
const GUIDE: u32 = rgba(0x3F, 0xB8, 0xA0, 0x80);
const HOVER_EDGE: u32 = rgba(255, 255, 255, 0x80);
const EDITOR_BUTTON_WIDTH: f32 = 100.0;
/// Half the side of the resize handle.
const HANDLE: f32 = 2.0;
const SCALE_STEP: f32 = 0.1;
const FAR_HILLS: [i32; 16] = [
    62, 58, 58, 54, 54, 58, 62, 62, 66, 62, 58, 58, 62, 66, 66, 62,
];
const NEAR_HILLS: [i32; 16] = [
    74, 74, 70, 70, 74, 78, 78, 74, 74, 70, 70, 74, 74, 78, 74, 74,
];

const NAMES: [(Mod, &str, &str); 3] = [
    (Mod::Fps, "patina.client.fps", "FPS Counter"),
    (Mod::Keystrokes, "patina.client.keystrokes", "Keystrokes"),
    (Mod::Coordinates, "patina.client.coordinates", "Coordinates"),
];

fn tr(key: &str, fallback: &str) -> String {
    lang::get_or(key, fallback)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Act {
    Toggle(Mod),
    FpsAccuracy,
    FpsStyle,
    Edit,
    Done,
    ResetAll,
    EditorDone,
}
/// What a click did, for the options screens to carry out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    None,
    /// A control took the click: sound it.
    Pressed,
    /// A mod in the editor was taken hold of: no sound, as in the Fabric mod.
    Grabbed,
    Done,
}
struct Control {
    rect: [f32; 4],
    act: Act,
    label: Text,
}
/// How a held mod follows the cursor.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Grab {
    /// Moved, held at this point of its box.
    Move(f32, f32),
    /// Resized by its corner.
    Resize,
}
#[derive(Default)]
pub struct ClientScreen {
    /// The HUD editor is open over the screen.
    editing: bool,
    selected: Option<Mod>,
    held: Option<(Mod, Grab)>,
    /// Whether the last frame was drawn over a world.
    world: std::cell::Cell<bool>,
}

/// The made-up values the editor shows without a world.
fn sample() -> ModReadings {
    ModReadings {
        fps: FpsReading {
            fps: 144,
            exact: 143.6,
            min: 91,
            low: 91,
        },
        keys: [true, false, false, false, true, false],
        position: [128.0, 64.0, -256.0],
        yaw: 180.0,
    }
}

impl ClientScreen {
    /// Whether the HUD editor is open, so Escape closes it first.
    pub fn editing(&self) -> bool {
        self.editing
    }
    /// Escape in the editor: back to the panel.
    pub fn leave_editor(&mut self) {
        self.editing = false;
        self.held = None;
        client_settings::update(|_| {});
    }
    fn panel(w: f32, h: f32) -> [i32; 4] {
        let (width, height) = (w.ceil() as i32, h.ceil() as i32);
        let pw = (width - 24).min(PANEL_WIDTH);
        let ph = PADDING + TITLE + NAMES.len() as i32 * (ROW + ROW_GAP) - ROW_GAP
            + SECTION_GAP
            + CAPTION
            + ROW
            + SECTION_GAP
            + ROW
            + ROW_GAP
            + ROW
            + PADDING;
        [(width - pw) / 2, (height - ph) / 2, pw, ph]
    }
    fn row_y(py: i32, row: usize) -> i32 {
        py + PADDING + TITLE + row as i32 * (ROW + ROW_GAP)
    }
    fn caption_y(py: i32) -> i32 {
        Self::row_y(py, NAMES.len()) - ROW_GAP + SECTION_GAP
    }
    fn controls(&self, mods: &Mods, w: f32, h: f32) -> Vec<Control> {
        let rect = |x: i32, y: i32, w: i32, h: i32| [x as f32, y as f32, w as f32, h as f32];
        if self.editing {
            let y = (h.ceil() / 2.0).floor() - 10.0;
            let middle = (w.ceil() / 2.0).floor();
            return vec![
                Control {
                    rect: [
                        middle - EDITOR_BUTTON_WIDTH - 2.0,
                        y,
                        EDITOR_BUTTON_WIDTH,
                        20.0,
                    ],
                    act: Act::ResetAll,
                    label: Text::literal(tr("patina.client.editor_reset", "Reset")),
                },
                Control {
                    rect: [middle + 2.0, y, EDITOR_BUTTON_WIDTH, 20.0],
                    act: Act::EditorDone,
                    label: Text::literal(tr("gui.done", "Done")),
                },
            ];
        }
        let [px, py, pw, _] = Self::panel(w, h);
        let x = px + PADDING;
        let inner = pw - PADDING * 2;
        let half = (inner - 4) / 2;
        let mut controls: Vec<Control> = NAMES
            .iter()
            .enumerate()
            .map(|(i, (m, key, plain))| Control {
                rect: rect(x, Self::row_y(py, i), inner, ROW),
                act: Act::Toggle(*m),
                label: Text::literal(tr(key, plain)),
            })
            .collect();
        let option = |name: String, value: String| Text::literal(format!("{name}: {value}"));
        let accuracy = match mods.fps.accuracy {
            FpsAccuracy::Whole => tr("patina.client.accuracy.whole", "Whole"),
            FpsAccuracy::Exact => tr("patina.client.accuracy.exact", "Precise"),
        };
        let style = match mods.fps.style {
            FpsStyle::Plain => tr("patina.client.style.plain", "Text"),
            FpsStyle::Panel => tr("patina.client.style.panel", "Panel"),
        };
        let mut y = Self::caption_y(py) + CAPTION;
        controls.push(Control {
            rect: rect(x, y, half, ROW),
            act: Act::FpsAccuracy,
            label: option(tr("patina.client.accuracy", "Accuracy"), accuracy),
        });
        controls.push(Control {
            rect: rect(x + half + 4, y, half, ROW),
            act: Act::FpsStyle,
            label: option(tr("patina.client.style", "Style"), style),
        });
        y += ROW + SECTION_GAP;
        controls.push(Control {
            rect: rect(x, y, inner, ROW),
            act: Act::Edit,
            label: Text::literal(tr("patina.client.edit", "Edit HUD layout")),
        });
        y += ROW + ROW_GAP;
        controls.push(Control {
            rect: rect(x, y, inner, ROW),
            act: Act::Done,
            label: Text::literal(tr("gui.done", "Done")),
        });
        controls
    }

    /// The game's readings over a world, else made-up ones.
    fn readings(in_world: bool) -> ModReadings {
        extras_hud::live()
            .filter(|_| in_world)
            .unwrap_or_else(sample)
    }
    fn screen((w, h): (f32, f32)) -> (f32, f32) {
        (w.ceil(), h.ceil())
    }
    fn place(
        atlas: &HudAtlas,
        mods: &Mods,
        readings: &ModReadings,
        screen: (f32, f32),
        m: Mod,
    ) -> [f32; 4] {
        extras_hud::place_mod(atlas, mods, readings, m, screen)
    }
    /// The topmost shown mod under the cursor.
    fn mod_at(
        atlas: &HudAtlas,
        mods: &Mods,
        r: &ModReadings,
        screen: (f32, f32),
        (x, y): (f32, f32),
    ) -> Option<Mod> {
        Mod::ALL.into_iter().rev().find(|&m| {
            let [bx, by, bw, bh] = Self::place(atlas, mods, r, screen, m);
            mods.enabled(m) && x >= bx && x < bx + bw && y >= by && y < by + bh
        })
    }
    fn on_handle(
        &self,
        atlas: &HudAtlas,
        mods: &Mods,
        r: &ModReadings,
        screen: (f32, f32),
        (x, y): (f32, f32),
    ) -> bool {
        let Some(m) = self.selected.filter(|m| mods.enabled(*m)) else {
            return false;
        };
        let [bx, by, bw, bh] = Self::place(atlas, mods, r, screen, m);
        (x - (bx + bw)).abs() <= HANDLE + 1.0 && (y - (by + bh)).abs() <= HANDLE + 1.0
    }
    /// A new size with the top left corner where it was, moved in at a screen edge.
    fn resize(
        atlas: &HudAtlas,
        mods: &mut Mods,
        r: &ModReadings,
        screen: (f32, f32),
        m: Mod,
        scale: f32,
    ) {
        let [x, y, ..] = Self::place(atlas, mods, r, screen, m);
        let (w, h) = extras_hud::mod_size(atlas, mods, r, m);
        let scale = (scale.clamp(MIN_SCALE, MAX_SCALE) * 20.0).round() / 20.0;
        let (w, h) = (w * scale, h * scale);
        let (sw, sh) = screen;
        let (x, y) = (x.min((sw - w).max(0.0)), y.min((sh - h).max(0.0)));
        let place = mods.place_mut(m);
        place.scale = scale;
        extras_hud::move_mod(place, [x, y, w, h], screen);
    }

    pub fn draw(&self, batch: &mut HudBatch, atlas: &HudAtlas, cursor: (f32, f32), in_world: bool) {
        let mods = client_settings::get().mods;
        self.world.set(in_world);
        if self.editing {
            self.draw_editor(batch, atlas, &mods, cursor, in_world);
            return;
        }
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, in_world);
        let [px, py, pw, ph] = Self::panel(w, h).map(|v| v as f32);
        batch.fill(atlas, px, py, pw, ph, PANEL);
        outline(batch, atlas, [px, py, pw, ph], LINE);
        let padding = PADDING as f32;
        batch.text(
            atlas,
            "Patina Client",
            px + padding,
            py + padding + 1.0,
            HEADING,
            true,
        );
        for control in self.controls(&mods, w, h) {
            let [x, y, cw, ch] = control.rect;
            let hover = ui::hovered(cursor, x, y, cw, ch);
            match control.act {
                Act::Toggle(m) => {
                    widgets::record(&control.label.plain(), control.rect, true);
                    widgets::widget_cursor(hover, true);
                    let on = mods.enabled(m);
                    let fill = if hover { FILL_HOVER } else { FILL };
                    batch.fill(atlas, x, y, cw, ch, fill);
                    let edge = if hover { ui::ACCENT } else { BORDER };
                    outline(batch, atlas, control.rect, edge);
                    let ty = y + ((ROW - 8) / 2) as f32;
                    batch.text_styled(atlas, &control.label, x + 6.0, ty, u32::MAX, true);
                    let pill_x = x + cw - 6.0 - PILL_WIDTH;
                    let pill_y = y + ((ROW as f32 - PILL_HEIGHT) / 2.0).floor();
                    let track = if on { ui::ACCENT } else { PILL_OFF };
                    batch.fill(atlas, pill_x, pill_y, PILL_WIDTH, PILL_HEIGHT, track);
                    let knob_x = if on {
                        pill_x + PILL_WIDTH - 1.0 - KNOB
                    } else {
                        pill_x + 1.0
                    };
                    batch.fill(atlas, knob_x, pill_y + 1.0, KNOB, KNOB, KNOB_COLOR);
                }
                _ => ui::button(batch, atlas, x, y, cw, ch, &control.label, true, hover),
            }
        }
        let caption = tr(NAMES[0].1, NAMES[0].2);
        let caption_y = Self::caption_y(py as i32) as f32 + 1.0;
        batch.text(atlas, &caption, px + padding, caption_y, HINT, true);
    }

    fn draw_editor(
        &self,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
        mods: &Mods,
        cursor: (f32, f32),
        in_world: bool,
    ) {
        extras_hud::editor_drawn();
        let screen = Self::screen((batch.width, batch.height));
        let (w, h) = screen;
        // Over a world the world itself shows; without one, a stand-in of the game.
        if !in_world {
            scene(batch, atlas, screen);
        }
        for control in self.controls(mods, w, h) {
            let [x, y, cw, ch] = control.rect;
            let hover = self.held.is_none() && ui::hovered(cursor, x, y, cw, ch);
            ui::button(batch, atlas, x, y, cw, ch, &control.label, true, hover);
        }
        let hint = tr(
            "patina.client.editor_hint",
            "Drag to move; wheel or corner to resize",
        );
        let tw = HudBatch::text_width(atlas, &hint).ceil();
        let hint_x = ((w - tw) / 2.0).floor();
        batch.text(atlas, &hint, hint_x, (h / 2.0).floor() + 14.0, HINT, true);
        let readings = Self::readings(in_world);
        for m in Mod::ALL {
            if mods.enabled(m) {
                let rect = Self::place(atlas, mods, &readings, screen, m);
                extras_hud::draw_mod(batch, atlas, mods, &readings, m, rect, 1.0);
            }
        }
        if let Some((m, Grab::Move(..))) = self.held {
            let place = mods.place(m);
            if place.position.horizontal() == 0 && place.x == 0.0 {
                batch.fill(atlas, (w / 2.0).floor(), 0.0, 1.0, h, GUIDE);
            }
            if place.position.vertical() == 0 && place.y == 0.0 {
                batch.fill(atlas, 0.0, (h / 2.0).floor(), w, 1.0, GUIDE);
            }
        }
        let held = self.held.map(|(m, _)| m);
        let hovered = held.or_else(|| Self::mod_at(atlas, mods, &readings, screen, cursor));
        if let Some(m) = hovered.filter(|m| Some(*m) != self.selected) {
            let rect = Self::place(atlas, mods, &readings, screen, m);
            frame(batch, atlas, rect, HOVER_EDGE, false);
        }
        if let Some(m) = self.selected.filter(|m| mods.enabled(*m)) {
            let rect = Self::place(atlas, mods, &readings, screen, m);
            frame(batch, atlas, rect, ui::ACCENT, true);
            let size = format!("{}%", (mods.place(m).scale * 100.0).round());
            let [bx, by, _, bh] = rect;
            let y = if by >= 11.0 {
                by.floor() - 11.0
            } else {
                (by + bh).ceil() + 3.0
            };
            batch.text(atlas, &size, bx.floor(), y, HINT, true);
        }
        let resizing = matches!(self.held, Some((_, Grab::Resize)));
        let on_handle = self.on_handle(atlas, mods, &readings, screen, cursor);
        if resizing || self.held.is_none() && on_handle {
            widgets::request_cursor(widgets::Cursor::Crosshair);
        } else if hovered.is_some() {
            widgets::request_cursor(widgets::Cursor::Move);
        }
    }

    /// A press at `cursor`; `back` is the right button.
    pub fn click(
        &mut self,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        size: (f32, f32),
        back: bool,
    ) -> Outcome {
        let mods = client_settings::get().mods;
        if self.editing && !back {
            let screen = Self::screen(size);
            let readings = Self::readings(self.world.get());
            if self.on_handle(atlas, &mods, &readings, screen, cursor) {
                self.held = self.selected.map(|m| (m, Grab::Resize));
                return Outcome::Grabbed;
            }
            if let Some(m) = Self::mod_at(atlas, &mods, &readings, screen, cursor) {
                let [bx, by, ..] = Self::place(atlas, &mods, &readings, screen, m);
                self.selected = Some(m);
                self.held = Some((m, Grab::Move(cursor.0 - bx, cursor.1 - by)));
                return Outcome::Grabbed;
            }
            self.selected = None;
        }
        let Some(act) = self
            .controls(&mods, size.0, size.1)
            .iter()
            .find(|c| ui::hovered(cursor, c.rect[0], c.rect[1], c.rect[2], c.rect[3]))
            .map(|c| c.act)
        else {
            return Outcome::None;
        };
        self.act(act);
        match act {
            Act::Done => Outcome::Done,
            _ => Outcome::Pressed,
        }
    }
    fn act(&mut self, act: Act) {
        match act {
            Act::Done => {}
            Act::Edit => {
                self.editing = true;
                self.selected = None;
            }
            Act::EditorDone => self.leave_editor(),
            Act::Toggle(m) => client_settings::update(|s| {
                let on = s.mods.enabled_mut(m);
                *on = !*on;
            }),
            Act::FpsAccuracy => client_settings::update(|s| {
                s.mods.fps.accuracy = match s.mods.fps.accuracy {
                    FpsAccuracy::Whole => FpsAccuracy::Exact,
                    FpsAccuracy::Exact => FpsAccuracy::Whole,
                };
            }),
            Act::FpsStyle => client_settings::update(|s| {
                s.mods.fps.style = match s.mods.fps.style {
                    FpsStyle::Plain => FpsStyle::Panel,
                    FpsStyle::Panel => FpsStyle::Plain,
                };
            }),
            Act::ResetAll => client_settings::update(|s| {
                let defaults = Mods::default();
                for m in Mod::ALL {
                    *s.mods.place_mut(m) = defaults.place(m);
                }
            }),
        }
    }
    /// The cursor moved with the button held: a held mod follows it.
    pub fn drag(&mut self, atlas: &HudAtlas, (x, y): (f32, f32), size: (f32, f32)) {
        let Some((m, grab)) = self.held else {
            return;
        };
        let screen = Self::screen(size);
        let readings = Self::readings(self.world.get());
        client_settings::update_unsaved(|s| match grab {
            Grab::Resize => {
                let [bx, by, ..] = Self::place(atlas, &s.mods, &readings, screen, m);
                let (bw, bh) = extras_hud::mod_size(atlas, &s.mods, &readings, m);
                let scale = ((x - bx) / bw).max((y - by) / bh);
                Self::resize(atlas, &mut s.mods, &readings, screen, m, scale);
            }
            Grab::Move(gx, gy) => {
                let (bw, bh) = extras_hud::mod_size(atlas, &s.mods, &readings, m);
                let scale = s.mods.place(m).scale;
                let (bw, bh) = (bw * scale, bh * scale);
                let (sw, sh) = screen;
                let left = (x - gx).clamp(0.0, (sw - bw).max(0.0));
                let top = (y - gy).clamp(0.0, (sh - bh).max(0.0));
                extras_hud::move_mod(s.mods.place_mut(m), [left, top, bw, bh], screen);
            }
        });
    }
    /// The button came up: a held mod stays where it was let go, and is saved.
    pub fn release(&mut self) {
        if self.held.take().is_some() {
            client_settings::update(|_| {});
        }
    }
    /// The mouse wheel: in the editor it resizes the mod under the cursor;
    /// on the panel it steps the FPS counter's options.
    pub fn wheel(
        &mut self,
        atlas: &HudAtlas,
        cursor: (f32, f32),
        size: (f32, f32),
        amount: f32,
    ) -> Outcome {
        if amount == 0.0 {
            return Outcome::None;
        }
        let mods = client_settings::get().mods;
        if self.editing {
            let screen = Self::screen(size);
            let readings = Self::readings(self.world.get());
            if let Some(m) = Self::mod_at(atlas, &mods, &readings, screen, cursor) {
                self.selected = Some(m);
                client_settings::update(|s| {
                    let scale = s.mods.place(m).scale + amount.signum() * SCALE_STEP;
                    Self::resize(atlas, &mut s.mods, &readings, screen, m, scale);
                });
            }
            return Outcome::None;
        }
        // CycleButton.mouseScrolled: the options with values step.
        let act = self
            .controls(&mods, size.0, size.1)
            .iter()
            .find(|c| ui::hovered(cursor, c.rect[0], c.rect[1], c.rect[2], c.rect[3]))
            .map(|c| c.act);
        if let Some(act @ (Act::FpsAccuracy | Act::FpsStyle)) = act {
            self.act(act);
        }
        Outcome::None
    }
}

/// `graphics.outline`: one GUI pixel inside the rectangle.
fn outline(batch: &mut HudBatch, atlas: &HudAtlas, [x, y, w, h]: [f32; 4], color: u32) {
    batch.fill(atlas, x, y, w, 1.0, color);
    batch.fill(atlas, x, y + h - 1.0, w, 1.0, color);
    batch.fill(atlas, x, y + 1.0, 1.0, h - 2.0, color);
    batch.fill(atlas, x + w - 1.0, y + 1.0, 1.0, h - 2.0, color);
}

/// A mod's outline, with the resize handle on the selected one.
fn frame(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    [bx, by, bw, bh]: [f32; 4],
    color: u32,
    handle: bool,
) {
    let left = bx.floor() - 1.0;
    let top = by.floor() - 1.0;
    let right = (bx + bw).ceil() + 1.0;
    let bottom = (by + bh).ceil() + 1.0;
    outline(batch, atlas, [left, top, right - left, bottom - top], color);
    if handle {
        let side = HANDLE * 2.0;
        let (x, y) = (right - 1.0 - HANDLE, bottom - 1.0 - HANDLE);
        batch.fill(atlas, x, y, side, side, color);
    }
}

/// A stand-in for the game at its real size: sky, hills and ground, and the
/// HUD where the game puts it (crosshair, hotbar, hearts, food and the
/// experience bar).
fn scene(batch: &mut HudBatch, atlas: &HudAtlas, (w, h): (f32, f32)) {
    let (width, height) = (w as i32, h as i32);
    let fill = |batch: &mut HudBatch, x0: i32, y0: i32, x1: i32, y1: i32, color: u32| {
        let (x, y) = (x0 as f32, y0 as f32);
        batch.fill(atlas, x, y, (x1 - x0) as f32, (y1 - y0) as f32, color);
    };
    batch.fill_gradient(
        atlas,
        (0.0, 0.0, w, (height * 3 / 4) as f32),
        rgba(0x7F, 0xB2, 0xDA, 255),
        rgba(0xC4, 0xE0, 0xEC, 255),
    );
    for (tops, color) in [
        (FAR_HILLS, rgba(0x7D, 0xA6, 0x89, 255)),
        (NEAR_HILLS, rgba(0x4F, 0x8A, 0x45, 255)),
    ] {
        let columns = tops.len() as i32;
        for (i, top) in tops.into_iter().enumerate() {
            let i = i as i32;
            let left = width * i / columns;
            let right = width * (i + 1) / columns;
            fill(batch, left, height * top / 100, right, height, color);
        }
    }
    let ground = height * 84 / 100;
    fill(batch, 0, ground, width, height, rgba(0x6B, 0x4A, 0x32, 255));
    fill(
        batch,
        0,
        ground,
        width,
        ground + 3,
        rgba(0x5E, 0x9A, 0x50, 255),
    );
    let sprite = |batch: &mut HudBatch, name: &str, x: i32, y: i32, sw: i32, sh: i32| {
        let name = format!("gui/sprites/{name}");
        let (x, y, sw, sh) = (x as f32, y as f32, sw as f32, sh as f32);
        ui::blit_sprite(batch, atlas, &name, x, y, sw, sh, u32::MAX);
    };
    sprite(
        batch,
        "hud/crosshair",
        (width - 15) / 2,
        (height - 15) / 2,
        15,
        15,
    );
    let bar = width / 2 - 91;
    sprite(batch, "hud/hotbar", bar, height - 22, 182, 22);
    sprite(batch, "hud/hotbar_selection", bar - 1, height - 23, 24, 23);
    sprite(
        batch,
        "hud/experience_bar_background",
        bar,
        height - 29,
        182,
        5,
    );
    for i in 0..10 {
        let heart = bar + i * 8;
        sprite(batch, "hud/heart/container", heart, height - 39, 9, 9);
        sprite(batch, "hud/heart/full", heart, height - 39, 9, 9);
        let food = width / 2 + 91 - i * 8 - 9;
        sprite(batch, "hud/food_empty", food, height - 39, 9, 9);
        sprite(batch, "hud/food_full", food, height - 39, 9, 9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> HudAtlas {
        HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap()
    }
    #[test]
    fn the_panel_is_the_fabric_mods_and_fits_the_smallest_screen() {
        // PatinaSettingsScreen.init at 480 x 270: 280 wide, 179 high, centred.
        assert_eq!(ClientScreen::panel(480.0, 270.0), [100, 45, 280, 179]);
        let screen = ClientScreen::default();
        for (w, h) in [(320.0, 240.0), (427.0, 240.0), (960.0, 540.0)] {
            let [px, py, pw, ph] = ClientScreen::panel(w, h).map(|v| v as f32);
            assert!(px >= 0.0 && py >= 0.0 && px + pw <= w && py + ph <= h);
            for c in screen.controls(&Mods::default(), w, h) {
                let [x, y, cw, ch] = c.rect;
                let inside = x >= px && y >= py && x + cw <= px + pw && y + ch <= py + ph;
                assert!(inside, "{:?}", c.act);
            }
        }
        let controls = screen.controls(&Mods::default(), 480.0, 270.0);
        assert_eq!(controls[0].rect, [108.0, 67.0, 264.0, 20.0]);
        // The FPS counter's options under its caption, then Edit and Done.
        assert_eq!(controls[3].rect, [108.0, 148.0, 130.0, 20.0]);
        assert_eq!(controls[5].rect, [108.0, 174.0, 264.0, 20.0]);
    }
    #[test]
    fn a_row_switches_its_mod_and_edit_opens_the_editor() {
        let atlas = atlas();
        let size = (480.0, 270.0);
        let mut screen = ClientScreen::default();
        client_settings::update_unsaved(|s| *s = client_settings::ClientSettings::default());
        let at = |screen: &ClientScreen, act: Act| {
            let controls = screen.controls(&Mods::default(), size.0, size.1);
            let c = controls.into_iter().find(|c| c.act == act).unwrap();
            (c.rect[0] + 1.0, c.rect[1] + 1.0)
        };
        let row = at(&screen, Act::Toggle(Mod::Keystrokes));
        assert_eq!(screen.click(&atlas, row, size, false), Outcome::Pressed);
        assert!(client_settings::get().mods.keystrokes.enabled);
        let edit = at(&screen, Act::Edit);
        screen.click(&atlas, edit, size, false);
        assert!(screen.editing());
        // In the editor the mods are where they are in game: grab the
        // keystrokes and drop them in the middle of the screen.
        let mods = client_settings::get().mods;
        let r = sample();
        let [bx, by, bw, bh] = ClientScreen::place(&atlas, &mods, &r, size, Mod::Keystrokes);
        let grab = (bx + bw / 2.0, by + bh / 2.0);
        assert_eq!(screen.click(&atlas, grab, size, false), Outcome::Grabbed);
        screen.drag(&atlas, (241.0, 136.0), size);
        screen.release();
        let place = client_settings::get().mods.keystrokes.place;
        let centre = (client_settings::Anchor::Center, 0.0, 0.0);
        assert_eq!((place.position, place.x, place.y), centre);
        // The wheel resizes it; Reset puts everything back.
        screen.wheel(&atlas, (240.0, 135.0), size, 1.0);
        assert_eq!(client_settings::get().mods.keystrokes.place.scale, 1.1);
        let reset = at(&screen, Act::ResetAll);
        screen.click(&atlas, reset, size, false);
        let default = Mods::default().keystrokes.place;
        assert_eq!(client_settings::get().mods.keystrokes.place, default);
        screen.leave_editor();
        assert!(!screen.editing());
    }
}
