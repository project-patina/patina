//! The Patina Client screen, as the Patina Fabric mod draws it
//! (`PatinaSettingsScreen`): module tiles with separate switches, the
//! FPS counter's options behind its tile, and the way into the HUD editor
//! (`PatinaHudEditorScreen`), where the mods are placed on a Minecraft scene
//! at their real size by the code that draws them in game. Drawing and clicks run
//! the same layout, so they never disagree.
use crate::{
    client_settings::{self, MAX_SCALE, MIN_SCALE, Mod, Mods, Utility},
    extras::FpsReading,
    extras_hud::{self, ModReadings},
    hud::{HudAtlas, HudBatch, HUD_BACKDROP_SPRITE, MENU_BACKDROP_SPRITE, rgba},
    lang,
    text::Text,
    ui, widgets,
};

const PADDING: i32 = 12;
const CARD_GAP: i32 = 10;
const SWITCH_HEIGHT: i32 = 14;
const OPTIONS_TOP: i32 = 32;
const OPTIONS_BOTTOM: i32 = 33;
const CONTROL_HEIGHT: i32 = 14;
const ROW_STEP: i32 = 22;
const GROUP_GAP: i32 = 6;
const THUMB_WIDTH: i32 = 96;
const THUMB_HEIGHT: i32 = 70;
const OPTION_TEXT_SCALE: f32 = 0.82;
const PANEL: u32 = rgba(12, 16, 18, 237);
const LINE: u32 = rgba(255, 255, 255, 0x30);
const HEADING: u32 = rgba(0x6F, 0xDC, 0xC3, 255);
const HINT: u32 = rgba(0x9F, 0xB0, 0xAC, 255);
const GUIDE: u32 = rgba(0x3F, 0xB8, 0xA0, 0x80);
const HOVER_EDGE: u32 = rgba(255, 255, 255, 0x80);
const RESET_WIDTH: f32 = 51.0;
const RESET_HEIGHT: f32 = 15.0;
const RESET_Y: f32 = 20.0;
const EDITOR_BACK_WIDTH: f32 = 52.0;
const EDITOR_BUTTON_GAP: f32 = 6.0;
const RESET_ICON: [u16; 9] = [
    0b100011100, 0b100100010, 0b101000001,
    0b111100001, 0b000000001, 0b000000001,
    0b010000010, 0b001000100, 0b000111000,
];
/// Half the side of the resize handle.
const HANDLE: f32 = 2.0;
const SCALE_STEP: f32 = 0.1;

/// Catalogue entries also include rendering modules, which never enter the HUD editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Module {
    Hud(Mod),
    FullBright,
    Utility(Utility),
}
impl Module {
    fn enabled(self, mods: &Mods) -> bool {
        match self {
            Self::Hud(m) => mods.enabled(m),
            Self::FullBright => mods.fullbright.enabled,
            Self::Utility(m) => mods.utility(m).enabled,
        }
    }
    fn tile(self) -> Act {
        match self {
            Self::Hud(m) => Act::Tile(m),
            Self::FullBright => Act::FullBrightTile,
            Self::Utility(m) => Act::UtilityTile(m),
        }
    }
    fn toggle(self) -> Act {
        match self {
            Self::Hud(m) => Act::Toggle(m),
            Self::FullBright => Act::FullBrightToggle,
            Self::Utility(m) => Act::UtilityToggle(m),
        }
    }
}
const NAMES: [(Module, &str, &str); 16] = [
    (Module::Hud(Mod::Fps), "patina.client.fps", "FPS Counter"),
    (Module::Hud(Mod::Keystrokes), "patina.client.keystrokes", "Keystrokes"),
    (Module::Hud(Mod::Coordinates), "patina.client.coordinates", "Coordinates"),
    (Module::Hud(Mod::Armor), "patina.mod.armor", "Armor Durability"),
    (Module::FullBright, "patina.mod.fullbright", "Full Bright"),
    (Module::Utility(Utility::AppleSkin), "patina.mod.appleskin", "AppleSkin"),
    (Module::Utility(Utility::ShulkerTooltip), "patina.mod.shulker_tooltip", "Shulker Tooltip"),
    (Module::Hud(Mod::Speedometer), "patina.mod.speedometer", "Speedometer"),
    (Module::Utility(Utility::Zoom), "patina.mod.zoom", "Zoom"),
    (Module::Hud(Mod::Ping), "patina.mod.ping", "Ping"),
    (Module::Utility(Utility::Perspective), "patina.mod.perspective", "Perspective"),
    (Module::Utility(Utility::HorseStats), "patina.mod.horse_stats", "Horse Stats"),
    (Module::Hud(Mod::Effects), "patina.mod.status_effects", "Status Effects"),
    (Module::Utility(Utility::Weather), "patina.mod.weather", "Weather Changer"),
    (Module::Utility(Utility::DeathInfo), "patina.mod.death_info", "Death Info"),
    (Module::Utility(Utility::NoFog), "patina.mod.no_fog", "No Fog"),
];

fn tr(key: &str, fallback: &str) -> String {
    lang::get_or(key, fallback)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Setting {
    FpsAccuracy, FpsStyle, Mouse, Space, Shift, Labels, Direction, Angle, Biome,
    Background, Vertical, HideEmpty, Theme, Durability, DynamicColor, Warning, Threshold,
    SpeedUnit, SpeedBackground, PingBackground, EffectColumns, EffectDuration, EffectAmplifier,
}
impl Setting {
    fn key(self) -> &'static str {
        use Setting::*;
        match self {
            FpsAccuracy => "accuracy", FpsStyle => "style", Mouse => "mouse", Space => "space",
            Shift => "shift", Labels => "labels", Direction => "direction", Angle => "angle",
            Biome => "biome", Background => "background", Vertical => "vertical",
            HideEmpty => "hide_empty", Theme => "theme", Durability => "durability",
            DynamicColor => "dynamic_color", Warning => "warning", Threshold => "threshold",
            SpeedUnit => "unit", SpeedBackground | PingBackground => "background", EffectColumns => "columns", EffectDuration => "duration", EffectAmplifier => "amplifier",
        }
    }

    fn label(self) -> String {
        let key = self.key();
        tr(&format!("patina.option.{key}"), key)
    }

    fn choices(self) -> &'static [&'static str] {
        match self {
            Self::FpsAccuracy => &["whole", "precise"],
            Self::FpsStyle => &["text", "panel"],
            Self::Durability => &["bar", "remaining"],
            Self::SpeedUnit => &["m/s", "km/h"], Self::EffectColumns => &["1", "2", "3", "4"],
            _ => &[],
        }
    }

    fn choice_label(self, value: &str) -> String {
        tr(&format!("patina.option.{}.{value}", self.key()), value)
    }

    fn value(self, mods: &Mods) -> String {
        use Setting::*;
        match self {
            FpsAccuracy => self.choice_label(if mods.fps.accuracy == client_settings::FpsAccuracy::Whole { "whole" } else { "precise" }),
            FpsStyle => self.choice_label(if mods.fps.style == client_settings::FpsStyle::Plain { "text" } else { "panel" }),
            Theme => self.choice_label("patina"),
            Durability => self.choice_label(&mods.armor.display),
            Threshold => format!("{}%", mods.armor.warning_threshold),
            SpeedUnit => mods.speedometer.unit.clone(), EffectColumns => mods.status_effects.columns.to_string(),
            _ => switch_label(self.enabled(mods)),
        }
    }

    fn enabled(self, mods: &Mods) -> bool {
        use Setting::*;
        match self {
            Mouse => mods.keystrokes.show_mouse, Space => mods.keystrokes.show_space,
            Shift => mods.keystrokes.show_shift, Labels => mods.keystrokes.show_labels,
            Direction => mods.coordinates.show_direction, Angle => mods.coordinates.show_angle,
            Biome => mods.coordinates.show_biome, Background => mods.coordinates.background,
            Vertical => mods.armor.vertical, HideEmpty => mods.armor.hide_empty,
            DynamicColor => mods.armor.dynamic_color, Warning => mods.armor.warning,
            SpeedBackground => mods.speedometer.background, PingBackground => mods.ping.background, EffectDuration => mods.status_effects.show_duration, EffectAmplifier => mods.status_effects.show_amplifier,
            _ => false,
        }
    }

    fn control_width(self, left: i32) -> i32 {
        match self {
            Self::FpsAccuracy | Self::FpsStyle | Self::Durability => 91.min(left * 3 / 5),
            Self::Threshold => 68,
            Self::Theme => 52,
            _ => 56,
        }
    }

    fn apply(self, mods: &mut Mods) {
        use Setting::*;
        let enabled = !self.enabled(mods);
        match self {
            Mouse => mods.keystrokes.show_mouse = enabled,
            Space => mods.keystrokes.show_space = enabled,
            Shift => mods.keystrokes.show_shift = enabled,
            Labels => mods.keystrokes.show_labels = enabled,
            Direction => mods.coordinates.show_direction = enabled,
            Angle => mods.coordinates.show_angle = enabled,
            Biome => mods.coordinates.show_biome = enabled,
            Background => mods.coordinates.background = enabled,
            Vertical => mods.armor.vertical = enabled,
            HideEmpty => mods.armor.hide_empty = enabled,
            DynamicColor => mods.armor.dynamic_color = enabled,
            Warning => mods.armor.warning = enabled,
            SpeedBackground => mods.speedometer.background = enabled, PingBackground => mods.ping.background = enabled, EffectDuration => mods.status_effects.show_duration = enabled, EffectAmplifier => mods.status_effects.show_amplifier = enabled,
            _ => {}
        }
    }

    fn pick(self, mods: &mut Mods, index: usize) {
        use client_settings::{FpsAccuracy, FpsStyle};
        match self {
            Self::FpsAccuracy => mods.fps.accuracy = [FpsAccuracy::Whole, FpsAccuracy::Exact][index],
            Self::FpsStyle => mods.fps.style = [FpsStyle::Plain, FpsStyle::Panel][index],
            Self::Durability => mods.armor.display = self.choices()[index].into(),
            Self::SpeedUnit => mods.speedometer.unit = self.choices()[index].into(), Self::EffectColumns => mods.status_effects.columns = index as u8 + 1,
            _ => {}
        }
    }

}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Act {
    Tile(Mod),
    Toggle(Mod),
    FullBrightTile,
    FullBrightToggle,
    UtilityTile(Utility), UtilityToggle(Utility),
    CloseOptions,
    Option(Setting),
    Category(&'static str),
    Pick(Setting, usize),
    Edit,
    Done,
    ResetAll,
    ResetModule(Mod),
    BackFromEditor,
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
pub struct ClientScreen {
    /// The HUD editor is open over the screen.
    editing: bool,
    options: Option<Mod>,
    utility: Option<crate::utility_screen::UtilityScreen>,
    catalogue_scroll: usize,
    options_scroll: i32,
    collapsed: std::collections::HashSet<&'static str>,
    dropdown: Option<Setting>,
    slider: Option<[f32; 4]>,
    search: ui::EditBox,
    selected: Option<Mod>,
    held: Option<(Mod, Grab)>,
    /// Whether the last frame was drawn over a world.
    world: std::cell::Cell<bool>,
}

impl Default for ClientScreen {
    fn default() -> Self {
        let mut search = ui::EditBox::new(0.0, 0.0, 0.0, 14.0);
        search.max_length = 64;
        search.focused = false;
        Self {
            editing: false,
            options: None,
            utility: None, catalogue_scroll: 0,
            options_scroll: 0,
            collapsed: Default::default(),
            dropdown: None,
            slider: None,
            search,
            selected: None,
            held: None,
            world: Default::default(),
        }
    }
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
        keys: [true, false, false, false, true, false, true, false],
        position: [128.0, 64.0, -256.0],
        yaw: 180.0,
        pitch: 12.0, biome: tr("patina.preview.biome", "Plains"), left_offhand: false, armor: extras_hud::sample_armor(), speed: 4.3, ping: 42, effects: extras_hud::sample_effects(),
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
    /// Escape dismisses an inner view before the catalogue itself.
    pub fn close_subpage(&mut self) -> bool {
        if let Some(utility) = &mut self.utility {
            if !utility.escape() { self.utility = None; }
            return true;
        }
        if self.editing {
            self.leave_editor();
            true
        } else if self.dropdown.take().is_some() {
            true
        } else if self.options.take().is_some() {
            self.release();
            true
        } else {
            self.search.focused = false;
            self.search.set_value("");
            false
        }
    }
    fn panel_width(width: i32) -> i32 {
        312.min(width - 16)
    }
    fn panel_height(height: i32, minimum: i32) -> i32 {
        minimum.min(height - 16)
    }
    fn catalogue_height(width: i32) -> i32 {
        let _ = width; 288
    }
    fn catalogue_frame(w: f32, h: f32) -> [i32; 4] {
        let (width, height) = (w.ceil() as i32, h.ceil() as i32);
        let pw = 396.min(width - 16);
        let ph = Self::panel_height(height, Self::catalogue_height(Self::panel_width(width)) + 44);
        [(width - pw) / 2, (height - ph) / 2, pw, ph]
    }
    fn panel(w: f32, h: f32) -> [i32; 4] {
        let width = w.ceil() as i32;
        let pw = Self::panel_width(width);
        let ph = Self::panel_height(h.ceil() as i32, Self::catalogue_height(pw));
        let [_, frame_y, _, frame_height] = Self::catalogue_frame(w, h);
        [(width - pw) / 2, frame_y + 8.min((frame_height - ph) / 2), pw, ph]
    }
    fn tile(w: f32, h: f32, index: usize) -> [i32; 4] {
        let [px, py, pw, ph] = Self::panel(w, h);
        let columns = if pw >= 264 { 4 } else { 2 };
        let tw = (pw - PADDING * 2 - CARD_GAP * (columns - 1)) / columns;
        let _ = ph;
        let top = Self::tile_top(pw);
        let th = 53;
        [px + PADDING + index as i32 % columns * (tw + CARD_GAP), py + top + index as i32 / columns * (th + CARD_GAP), tw, th]
    }
    fn tile_top(panel_width: i32) -> i32 {
        if panel_width >= 252 { 36 } else { 50 }
    }
    fn has_thumbnail(width: i32, height: i32) -> bool {
        width >= 284 && height - OPTIONS_TOP - OPTIONS_BOTTOM >= THUMB_HEIGHT
    }
    fn option_list_width(width: i32, height: i32) -> i32 {
        width - 28 - if Self::has_thumbnail(width, height) { THUMB_WIDTH + 14 } else { 0 }
    }
    fn dialog(w: f32, h: f32) -> [i32; 4] {
        let (width, height) = (w.ceil() as i32, h.ceil() as i32);
        let pw = 386.min(width - 16);
        let ph = Self::panel_height(height, 211);
        [(width - pw) / 2, (height - ph) / 2, pw, ph]
    }
    fn matching_mods(&self) -> impl Iterator<Item = &(Module, &'static str, &'static str)> {
        let query = self.search.value.trim().to_lowercase();
        NAMES.iter().filter(move |(module, key, plain)| {
            tr(key, plain).to_lowercase().contains(&query)
                || plain.to_lowercase().contains(&query)
                || key.rsplit('.').next().is_some_and(|name| name.contains(&query))
                || (*module == Module::FullBright && "fullbright brightness jasnosc jasność".contains(&query))
        })
    }

    fn search_rect(w: f32, h: f32) -> [f32; 4] {
        let [px, py, pw, _] = Self::panel(w, h).map(|v| v as f32);
        if pw >= 252.0 {
            let [fx, _, fw, _] = Self::catalogue_frame(w, h).map(|v| v as f32);
            let search_width = pw - 198.0;
            [fx + fw - 32.0 - search_width, py + 6.0, search_width, 14.0]
        } else {
            [px + 12.0, py + 26.0, pw - 24.0, 14.0]
        }
    }

    fn max_catalogue_scroll(&self, w: f32, h: f32) -> usize {
        let columns = if Self::panel_width(w.ceil() as i32) >= 264 {4} else {2};
        let [_,y,_,fh] = Self::catalogue_frame(w,h); let [_,ty,_,_] = Self::tile(w,h,0);
        let visible = ((y+fh-8-ty+CARD_GAP)/(53+CARD_GAP)).max(1) as usize;
        self.matching_mods().count().div_ceil(columns).saturating_sub(visible)
    }
    pub fn bind_key(&mut self, key: winit::keyboard::KeyCode) -> bool { self.utility.as_mut().is_some_and(|s| s.bind(key)) }
    fn catalogue_controls(&self, w: f32, h: f32) -> Vec<Control> {
        let mut controls = Vec::new();
        for (i, (m, key, plain)) in self.matching_mods().enumerate() {
            let columns = if Self::panel_width(w.ceil() as i32) >= 264 { 4 } else { 2 };
            let start = self.catalogue_scroll.min(self.max_catalogue_scroll(w,h)) * columns;
            if i < start { continue; }
            let i = i - start;
            let [_, frame_y, _, frame_h] = Self::catalogue_frame(w,h);
            let [_, tile_y, _, tile_h] = Self::tile(w,h,i);
            if tile_y + tile_h > frame_y + frame_h - 8 { continue; }
            let [x, y, tw, th] = Self::tile(w, h, i).map(|v| v as f32);
            controls.push(Control { rect: [x, y, tw, th - SWITCH_HEIGHT as f32], act: m.tile(), label: Text::literal(tr(key, plain)) });
            controls.push(Control { rect: [x, y + th - SWITCH_HEIGHT as f32, tw, SWITCH_HEIGHT as f32], act: m.toggle(), label: Text::literal(tr(key, plain)) });
        }
        let [px, py, pw, _] = Self::panel(w, h).map(|v| v as f32);
        let [fx, _, fw, _] = Self::catalogue_frame(w, h).map(|v| v as f32);
        let layout_x = if pw >= 252.0 { Self::search_rect(w, h)[0] - 82.0 } else { px + 84.0 };
        controls.push(Control { rect: [layout_x, py + 6.0, 71.0, 14.0], act: Act::Edit, label: Text::literal(tr("patina.client.edit", "HUD Layout")) });
        controls.push(Control { rect: [fx + fw - 26.0, py + 6.0, 14.0, 14.0], act: Act::Done, label: Text::literal("×") });
        controls
    }
    fn option_rows(&self) -> Vec<(i32, Option<Setting>, &'static str)> {
        use Setting::*;
        let groups: Vec<(&str, Vec<Setting>)> = match self.options {
            Some(Mod::Fps) => vec![("appearance", vec![FpsAccuracy, FpsStyle])],
            Some(Mod::Keystrokes) => vec![("keys", vec![Mouse, Space, Shift]), ("appearance", vec![Labels])],
            Some(Mod::Coordinates) => vec![("information", vec![Direction, Angle, Biome]), ("appearance", vec![Background])],
            Some(Mod::Speedometer) => vec![("appearance", vec![SpeedUnit, SpeedBackground])],
            Some(Mod::Ping) => vec![("appearance", vec![PingBackground])],
            Some(Mod::Effects) => vec![("layout", vec![EffectColumns]), ("appearance", vec![EffectDuration, EffectAmplifier])],
            Some(Mod::Armor) => vec![("layout", vec![Vertical, HideEmpty, Durability, DynamicColor]), ("appearance", vec![Theme]), ("warnings", vec![Warning, Threshold])],
            None => vec![],
        };
        let mut rows = Vec::new();
        let mut offset = 0;
        for (index, (group, settings)) in groups.into_iter().enumerate() {
            if index > 0 {
                offset += GROUP_GAP;
            }
            rows.push((offset, None, group));
            offset += ROW_STEP;
            if !self.collapsed.contains(group) {
                for setting in settings {
                    rows.push((offset, Some(setting), ""));
                    offset += ROW_STEP;
                }
            }
        }
        rows
    }
    fn content_height(&self) -> i32 { self.option_rows().last().map_or(0, |(y, _, _)| y + ROW_STEP) }
    fn max_options_scroll(&self, view_height: i32) -> i32 {
        (self.content_height() - view_height).max(0)
    }
    fn controls(&self, mods: &Mods, w: f32, h: f32) -> Vec<Control> {
        if self.editing {
            let x = ((w.ceil() - RESET_WIDTH - EDITOR_BUTTON_GAP - EDITOR_BACK_WIDTH) / 2.0).floor();
            return vec![
                Control { rect: [x, RESET_Y, RESET_WIDTH, RESET_HEIGHT], act: Act::ResetAll, label: Text::literal(tr("patina.client.editor_reset", "Reset")) },
                Control { rect: [x + RESET_WIDTH + EDITOR_BUTTON_GAP, RESET_Y, EDITOR_BACK_WIDTH, RESET_HEIGHT], act: Act::BackFromEditor, label: Text::literal(tr("patina.editor.back", "Back")) },
            ];
        }
        let Some(module) = self.options else { return self.catalogue_controls(w, h); };
        let [x, y, pw, ph] = Self::dialog(w, h);
        let left = Self::option_list_width(pw, ph);
        let top = y + OPTIONS_TOP;
        let bottom = y + ph - OPTIONS_BOTTOM;
        let scroll = self.options_scroll.min(self.max_options_scroll(bottom - top));
        let mut controls = Vec::new();
        for (offset, setting, group) in self.option_rows() {
            let cy = top + offset - scroll;
            if cy < top || cy + CONTROL_HEIGHT > bottom { continue; }
            let (act, label, width) = if let Some(setting) = setting {
                (Act::Option(setting), format!("{}: {}", setting.label(), setting.value(mods)), setting.control_width(left))
            } else {
                (Act::Category(group), tr(&format!("patina.category.{group}"), group), left)
            };
            controls.push(Control {
                rect: [(x + PADDING + left - width) as f32, cy as f32, width as f32, CONTROL_HEIGHT as f32],
                act, label: Text::literal(label),
            });
        }
        controls.push(Control { rect: [(x + pw - 132) as f32, (y + 6) as f32, 64.0, CONTROL_HEIGHT as f32], act: Act::Edit, label: Text::literal(tr("patina.settings.preview", "Preview")) });
        controls.push(Control { rect: [(x + pw - 62) as f32, (y + 6) as f32, 52.0, CONTROL_HEIGHT as f32], act: Act::CloseOptions, label: Text::literal(tr("patina.client.back", "Back")) });
        controls.push(Control { rect: [(x + PADDING) as f32, (y + ph - 23) as f32, 96.0, RESET_HEIGHT], act: Act::ResetModule(module), label: Text::literal(tr("patina.settings.reset", "Reset settings")) });
        if let Some(setting) = self.dropdown
            && let Some(control) = controls.iter().find(|control| control.act == Act::Option(setting))
        {
            let [cx, cy, cw, ch] = control.rect;
            let values = setting.choices();
            let top = (cy - ch * values.len() as f32).max(2.0);
            let popup_width = cw.max(94.0);
            for (i, value) in values.iter().enumerate() {
                controls.push(Control {
                    rect: [cx + cw - popup_width, top + ch * i as f32, popup_width, ch],
                    act: Act::Pick(setting, i), label: Text::literal(setting.choice_label(value)),
                });
            }
        }
        controls
    }

    /// The game's readings over a world, else made-up ones.
    fn readings(&self, in_world: bool) -> ModReadings {
        (in_world && self.options.is_none())
            .then(extras_hud::live)
            .flatten()
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
    fn hit_box(rect: [f32; 4], mods: &Mods, m: Mod) -> [f32; 4] {
        if m == Mod::Armor { extras_hud::armor_slots(rect, &mods.armor) } else { rect }
    }

    /// The topmost shown mod under the cursor.
    fn mod_at(
        &self,
        atlas: &HudAtlas,
        mods: &Mods,
        r: &ModReadings,
        screen: (f32, f32),
        (x, y): (f32, f32),
    ) -> Option<Mod> {
        Mod::ALL.into_iter().rev().find(|&m| {
            let [bx, by, bw, bh] = Self::hit_box(Self::place(atlas, mods, r, screen, m), mods, m);
            self.shown(mods, m) && x >= bx && x < bx + bw && y >= by && y < by + bh
        })
    }
    fn shown(&self, mods: &Mods, m: Mod) -> bool {
        mods.enabled(m) || self.options == Some(m)
    }

    fn on_handle(
        &self,
        atlas: &HudAtlas,
        mods: &Mods,
        r: &ModReadings,
        screen: (f32, f32),
        (x, y): (f32, f32),
    ) -> bool {
        let Some(m) = self.selected.filter(|m| self.shown(mods, *m)) else {
            return false;
        };
        let [bx, by, bw, bh] = Self::hit_box(Self::place(atlas, mods, r, screen, m), mods, m);
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

    pub fn draw(&mut self, batch: &mut HudBatch, atlas: &HudAtlas, cursor: (f32, f32), in_world: bool) {
        let mods = client_settings::get().mods;
        self.world.set(in_world);
        if let Some(utility) = &self.utility {
            let (w,h) = (batch.width,batch.height); ui::menu_background(batch,atlas,0.0,0.0,w,h,in_world); utility.draw(batch,atlas,&mods,cursor); return;
        }
        if self.editing { self.draw_editor(batch, atlas, &mods, cursor, in_world); return; }
        let (w, h) = (batch.width, batch.height);
        ui::menu_background(batch, atlas, 0.0, 0.0, w, h, in_world);
        if let Some(m) = self.options { self.draw_options(batch, atlas, &mods, cursor, m); }
        else { self.draw_catalogue(batch, atlas, &mods, cursor); }
    }
    fn draw_options(&self, batch: &mut HudBatch, atlas: &HudAtlas, mods: &Mods, cursor: (f32, f32), m: Mod) {
        let (w, h) = (batch.width, batch.height);
        let [x, y, pw, ph] = Self::dialog(w, h).map(|v| v as f32);
        let left = Self::option_list_width(pw as i32, ph as i32) as f32;
        let (top, bottom) = (y + OPTIONS_TOP as f32, y + ph - OPTIONS_BOTTOM as f32);
        let scroll = self.options_scroll.min(self.max_options_scroll((bottom - top) as i32)) as f32;
        ui::plate(batch, atlas, [x, y, pw, ph], rgba(12, 16, 18, 245), LINE);
        batch.fill(atlas, x, y, pw, 2.0, ui::ACCENT);
        let (_, key, fallback) = NAMES.iter().find(|(which, _, _)| *which == Module::Hud(m)).unwrap();
        draw_label(batch, atlas, &Text::literal(tr(key, fallback)), [x + 12.0, y + 6.0, pw - 148.0, 14.0], HEADING, OPTION_TEXT_SCALE, false);
        if Self::has_thumbnail(pw as i32, ph as i32) {
            draw_thumbnail(batch, atlas, mods, m, x + pw - 12.0 - THUMB_WIDTH as f32, top);
        }
        if self.content_height() as f32 > bottom - top {
            let track = bottom - top;
            let thumb = (track * track / self.content_height() as f32).max(12.0);
            let sy = top + (track - thumb) * scroll / (self.content_height() as f32 - track);
            batch.fill(atlas, x + left + 15.0, top, 2.0, track, LINE);
            batch.fill(atlas, x + left + 15.0, sy, 2.0, thumb, ui::ACCENT);
        }
        for control in self.controls(mods, w, h) {
            let [cx, cy, cw, ch] = control.rect;
            let hover = ui::hovered(cursor, cx, cy, cw, ch);
            if let Act::Option(setting) = control.act {
                let (label, value) = (setting.label(), setting.value(mods));
                let label_x = x + PADDING as f32;
                let label_width = cx - label_x - 8.0;
                draw_label(batch, atlas, &Text::literal(label), [label_x, cy, label_width, ch], u32::MAX, OPTION_TEXT_SCALE, false);
                let active = setting != Setting::Theme;
                if self.dropdown.is_none() || self.dropdown == Some(setting) {
                    let kind = if setting == Setting::Threshold {
                        widgets::Kind::Slider {
                            value: (mods.armor.warning_threshold as f32 - 5.0) / 45.0,
                            track: [cx + 4.0, cw - 32.0],
                        }
                    } else {
                        widgets::Kind::Button
                    };
                    widgets::record_as(&control.label.plain(), control.rect, active, kind);
                    widgets::widget_cursor(hover, active);
                }
                if setting == Setting::Threshold {
                    let start = cx + 4.0;
                    let end = cx + cw - 28.0;
                    let y = cy + ch / 2.0;
                    let knob = start + (end - start) * (mods.armor.warning_threshold as f32 - 5.0) / 45.0;
                    batch.fill(atlas, start, y - 1.0, end - start, 2.0, rgba(84, 94, 94, 255));
                    batch.fill(atlas, start, y - 1.0, knob - start, 2.0, ui::ACCENT);
                    batch.fill(atlas, knob - 2.0, y - 3.0, 5.0, 6.0, if hover { u32::MAX } else { HEADING });
                    draw_label(batch, atlas, &Text::literal(value), [end + 5.0, cy, 23.0, ch], u32::MAX, OPTION_TEXT_SCALE, false);
                } else if !setting.choices().is_empty() {
                    let clipped: String = value.chars().take(atlas.font.fit(&value, (cw - 20.0) / OPTION_TEXT_SCALE)).collect();
                    option_button(batch, atlas, control.rect, &Text::literal(clipped), true, hover);
                    chevron(batch, atlas, cx + cw - 8.0, cy + 7.0, if self.dropdown == Some(setting) { 1 } else { -1 });
                } else if setting == Setting::Theme {
                    option_button(batch, atlas, control.rect, &Text::literal(value), false, false);
                } else {
                    draw_switch(batch, atlas, control.rect, setting.enabled(mods), hover, OPTION_TEXT_SCALE, false);
                }
            } else if matches!(control.act, Act::Pick(..)) {
                let fill = if hover { rgba(31, 57, 53, 255) } else { rgba(18, 26, 29, 255) };
                ui::plate(batch, atlas, control.rect, fill, if hover { ui::ACCENT } else { LINE });
                draw_label(batch, atlas, &control.label, control.rect, u32::MAX, OPTION_TEXT_SCALE, true);
                widgets::record(&control.label.plain(), control.rect, true);
                widgets::widget_cursor(hover, true);
            } else {
                let hover = hover && self.dropdown.is_none();
                if self.dropdown.is_none() {
                    widgets::record(&control.label.plain(), control.rect, true);
                    widgets::widget_cursor(hover, true);
                }
                if matches!(control.act, Act::ResetModule(_)) {
                    reset_face(batch, atlas, control.rect, &control.label, OPTION_TEXT_SCALE, hover);
                } else {
                    option_button(batch, atlas, control.rect, &control.label, true, hover);
                    if let Act::Category(group) = control.act {
                        chevron(batch, atlas, cx + 10.0, cy + 7.0, if self.collapsed.contains(group) { 0 } else { 1 });
                    }
                }
            }
        }
    }

    fn draw_catalogue(&mut self, batch: &mut HudBatch, atlas: &HudAtlas, mods: &Mods, cursor: (f32, f32)) {
        let (w, h) = (batch.width, batch.height);
        let [px, py, pw, _] = Self::panel(w, h).map(|v| v as f32);
        let [fx, fy, fw, fh] = Self::catalogue_frame(w, h).map(|v| v as f32);
        ui::plate(batch, atlas, [fx, fy, fw, fh], PANEL, LINE);
        batch.fill(atlas, fx, fy, fw, 2.0, ui::ACCENT);
        let title_scale = if fw >= 330.0 { 1.15 } else { 1.0 };
        draw_label(batch, atlas, &Text::literal("Patina Client"), [fx + 12.0, py + 6.0, 100.0, 14.0], HEADING, title_scale, false);
        [self.search.x, self.search.y, self.search.width, self.search.height] = Self::search_rect(w, h);
        self.search.draw(batch, atlas, None);
        if self.search.value.is_empty() && !self.search.focused {
            batch.text(atlas, &tr("patina.settings.search", "Search"), self.search.text_x(), self.search.text_y(), HINT, true);
        }
        if self.matching_mods().next().is_none() {
            batch.text(atlas, &tr("patina.settings.no_results", "No matching mods"), px + 12.0, py + Self::tile_top(pw as i32) as f32 + 6.0, HINT, true);
        }
        for (i, (m, key, plain)) in self.matching_mods().enumerate() {
            let columns = if Self::panel_width(w.ceil() as i32) >= 264 { 4 } else { 2 };
            let start = self.catalogue_scroll.min(self.max_catalogue_scroll(w,h)) * columns;
            if i < start { continue; }
            let i = i - start;
            let [_, frame_y, _, frame_h] = Self::catalogue_frame(w,h);
            let [_, tile_y, _, tile_h] = Self::tile(w,h,i);
            if tile_y + tile_h > frame_y + frame_h - 8 { continue; }
            let [x, y, tw, th] = Self::tile(w, h, i).map(|v| v as f32);
            let hover = ui::hovered(cursor, x, y, tw, th - SWITCH_HEIGHT as f32);
            let on = m.enabled(mods);
            let fill = if hover { rgba(26, 37, 39, 240) } else { rgba(16, 24, 27, 192) };
            rounded_plate(batch, atlas, [x, y, tw, th - SWITCH_HEIGHT as f32], fill, if hover { ui::ACCENT } else { LINE }, true, false);
            let stripe = if on { ui::ACCENT } else { rgba(92, 113, 108, 64) };
            batch.fill(atlas, x + 2.0, y + 1.0, tw - 4.0, 1.0, stripe);
            batch.fill(atlas, x + 1.0, y + 2.0, tw - 2.0, 1.0, stripe);
            let start = batch.vertices.len();
            mod_icon(batch, atlas, *m, 0.0, 0.0, if on { HEADING } else { HINT });
            for vertex in &mut batch.vertices[start..] {
                vertex.pos = [x + (tw / 2.0).floor() - 7.0 + vertex.pos[0] * 0.58,
                    y + 6.0 + vertex.pos[1] * 0.58];
            }
            let name = Text::literal(tr(key, plain));
            let text_scale = 0.86;
            let lines = atlas.font.split(&name, (tw - 4.0) / text_scale);
            let count = lines.len().min(2);
            for (i, line) in lines.iter().take(count).enumerate() {
                let label_x = x + (tw - atlas.font.width_text(line) * text_scale) / 2.0;
                let label_y = y + th - SWITCH_HEIGHT as f32 - 9.0 - (count - 1 - i) as f32 * 8.0;
                let start = batch.vertices.len();
                batch.text_styled(atlas, line, 0.0, 0.0, u32::MAX, true);
                for vertex in &mut batch.vertices[start..] {
                    vertex.pos = [label_x + vertex.pos[0] * text_scale,
                        label_y + vertex.pos[1] * text_scale];
                }
            }
        }
        // Only the active layer contributes controls to keyboard navigation.
        for control in self.catalogue_controls(w, h) {
            let [x, y, cw, ch] = control.rect;
            let hover = ui::hovered(cursor, x, y, cw, ch);
            match control.act {
                Act::Tile(_) | Act::FullBrightTile | Act::UtilityTile(_) => {
                    if self.options.is_none() {
                        widgets::record(&control.label.plain(), control.rect, true);
                        widgets::widget_cursor(hover, true);
                    }
                }
                Act::Toggle(_) | Act::FullBrightToggle | Act::UtilityToggle(_) => {
                    let on = match control.act {
                        Act::Toggle(m) => mods.enabled(m),
                        Act::UtilityToggle(m) => mods.utility(m).enabled,
                        _ => mods.fullbright.enabled,
                    };
                    draw_switch(batch, atlas, control.rect, on, hover, 1.0, true);
                    widgets::record(&format!("{}: {}", control.label.plain(), switch_label(on)), control.rect, true);
                    widgets::widget_cursor(hover, true);
                }
                _ if self.options.is_none() => ui::button(batch, atlas, x, y, cw, ch, &control.label, true, hover),
                _ => {
                    // Same decoration, without registering covered buttons.
                    let (fill, edge, color) = ui::button_colors(true, false);
                    ui::plate(batch, atlas, control.rect, fill, edge);
                    batch.text_styled(atlas, &control.label, x + ((cw - HudBatch::text_width(atlas, &control.label.plain())) / 2.0).floor(), y + 6.0, color, true);
                }
            }
        }
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
        scene(batch, atlas, screen);
        let hint = tr(
            "patina.client.editor_hint",
            "Drag to move; wheel or corner to resize",
        );
        let tw = HudBatch::text_width(atlas, &hint).ceil();
        let hint_x = ((w - tw) / 2.0).floor();
        batch.fill(atlas, hint_x - 5.0, 4.0, tw + 10.0, 13.0, rgba(17, 24, 27, 208));
        batch.text(atlas, &hint, hint_x, 7.0, rgba(233, 243, 240, 255), true);
        let readings = self.readings(in_world);
        for m in Mod::ALL {
            if self.shown(mods, m) {
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
        let hovered = held.or_else(|| self.mod_at(atlas, mods, &readings, screen, cursor));
        if let Some(m) = hovered.filter(|m| Some(*m) != self.selected) {
            let rect = Self::place(atlas, mods, &readings, screen, m);
            frame(batch, atlas, Self::hit_box(rect, mods, m), HOVER_EDGE, false);
        }
        if let Some(m) = self.selected.filter(|m| self.shown(mods, *m)) {
            let rect = Self::place(atlas, mods, &readings, screen, m);
            frame(batch, atlas, Self::hit_box(rect, mods, m), ui::ACCENT, true);
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
        for control in self.controls(mods, w, h) {
            let [x, y, cw, ch] = control.rect;
            let hover = self.held.is_none() && ui::hovered(cursor, x, y, cw, ch);
            if control.act == Act::ResetAll {
                reset_face(batch, atlas, control.rect, &control.label, 1.0, hover);
            } else {
                let edge = if hover { HEADING } else { rgba(71, 91, 89, 255) };
                let fill = if hover { rgba(36, 57, 55, 247) } else { rgba(17, 24, 27, 240) };
                rounded_plate(batch, atlas, control.rect, fill, edge, true, true);
                ui::centered_text(batch, atlas, &control.label, x, y + 4.0, cw, rgba(233, 243, 240, 255));
            }
            widgets::widget_cursor(hover, true);
            widgets::record(&control.label.plain(), control.rect, true);
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
        if let Some(utility) = &mut self.utility {
            if let Some(close) = utility.click(cursor, size) { if close {self.utility=None;} return Outcome::Pressed; }
            return Outcome::None;
        }
        let mods = client_settings::get().mods;
        if self.editing {
            if back {
                return Outcome::None;
            }
            // Editor controls have priority over HUD elements dragged under them.
            if let Some(control) = self.controls(&mods, size.0, size.1).iter().find(|c| ui::hovered(cursor, c.rect[0], c.rect[1], c.rect[2], c.rect[3])) {
                self.held = None;
                self.selected = None;
                self.act(control.act);
                return Outcome::Pressed;
            }
            let screen = Self::screen(size);
            let readings = self.readings(self.world.get());
            if self.on_handle(atlas, &mods, &readings, screen, cursor) {
                self.held = self.selected.map(|m| (m, Grab::Resize));
                return Outcome::Grabbed;
            }
            if let Some(m) = self.mod_at(atlas, &mods, &readings, screen, cursor) {
                let [bx, by, ..] = Self::place(atlas, &mods, &readings, screen, m);
                self.selected = Some(m);
                self.held = Some((m, Grab::Move(cursor.0 - bx, cursor.1 - by)));
                return Outcome::Grabbed;
            }
            self.selected = None;
            return Outcome::None;
        }
        if back { return Outcome::None; }
        if self.options.is_none() {
            let [x, y, w, h] = Self::search_rect(size.0, size.1);
            if ui::hovered(cursor, x, y, w, h) {
                [self.search.x, self.search.y, self.search.width, self.search.height] = [x, y, w, h];
                self.search.focused = true;
                self.search.reset_blink();
                self.search.click(atlas, cursor.0, false, false);
                return Outcome::Grabbed;
            }
            self.search.focused = false;
        }
        let controls = self.controls(&mods, size.0, size.1);
        let target = controls.iter().rev().find(|c| ui::hovered(cursor, c.rect[0], c.rect[1], c.rect[2], c.rect[3]));
        if let Some(open) = self.dropdown
            && !target.is_some_and(|c| matches!(c.act, Act::Pick(..)) || c.act == Act::Option(open))
        {
            self.dropdown = None;
            return Outcome::Grabbed;
        }
        let Some(control) = target else { return Outcome::None; };
        let act = control.act;
        if act == Act::Option(Setting::Theme) { return Outcome::None; }
        if act == Act::Option(Setting::Threshold) {
            self.slider = Some(control.rect);
            Self::set_threshold(control.rect, cursor.0);
            return Outcome::Pressed;
        }
        self.act(act);
        match act {
            Act::Done => Outcome::Done,
            _ => Outcome::Pressed,
        }
    }
    fn act(&mut self, act: Act) {
        match act {
            Act::UtilityTile(m) => { self.utility = Some(crate::utility_screen::UtilityScreen::new(m)); self.search.focused = false; },
            Act::UtilityToggle(m) => client_settings::update(|s| {let utility=s.mods.utility_mut(m);utility.enabled=!utility.enabled;}),
            Act::Done => {}
            Act::FullBrightTile | Act::FullBrightToggle => client_settings::update(|s| {
                s.mods.fullbright.enabled = !s.mods.fullbright.enabled;
            }),
            Act::CloseOptions => { self.options = None; self.release(); },
            Act::Tile(m) => {
                self.options = Some(m);
                self.options_scroll = 0;
                self.collapsed.clear();
                if m == Mod::Armor {
                    self.collapsed.extend(["layout", "appearance", "warnings"]);
                }
            }
            Act::Category(key) => {
                if !self.collapsed.remove(key) { self.collapsed.insert(key); }
            }
            Act::Pick(setting, index) => {
                client_settings::update(|s| setting.pick(&mut s.mods, index));
                self.dropdown = None;
            }
            Act::Edit => {
                self.editing = true;
                self.selected = self.options;
            }
            Act::BackFromEditor => self.leave_editor(),
            Act::Toggle(m) => client_settings::update(|s| {
                let on = s.mods.enabled_mut(m);
                *on = !*on;
            }),
            Act::Option(setting) => {
                if setting.choices().is_empty() {
                    client_settings::update(|s| setting.apply(&mut s.mods));
                } else {
                    self.dropdown = (self.dropdown != Some(setting)).then_some(setting);
                }
            }
            Act::ResetModule(m) => client_settings::update(|s| s.mods.reset_options(m)),
            Act::ResetAll => client_settings::update(|s| {
                let defaults = Mods::default();
                for m in Mod::ALL {
                    *s.mods.place_mut(m) = defaults.place(m);
                }
            }),
        }
    }
    pub fn search_mut(&mut self) -> Option<&mut ui::EditBox> {
        (!self.editing && self.utility.is_none() && self.options.is_none() && self.search.focused).then_some(&mut self.search)
    }

    fn set_threshold([x, _, w, _]: [f32; 4], cursor: f32) {
        let ratio = ((cursor - x - 4.0) / (w - 32.0)).clamp(0.0, 1.0);
        client_settings::update_unsaved(|s| s.mods.armor.warning_threshold = 5 + (ratio * 45.0).round() as u8);
    }

    /// The cursor moved with the button held: a held mod follows it.
    pub fn drag(&mut self, atlas: &HudAtlas, (x, y): (f32, f32), size: (f32, f32)) {
        if let Some(utility) = &mut self.utility { utility.drag((x,y)); return; }
        if let Some(rect) = self.slider {
            Self::set_threshold(rect, x);
            return;
        }
        let Some((m, grab)) = self.held else {
            return;
        };
        let screen = Self::screen(size);
        let readings = self.readings(self.world.get());
        client_settings::update_unsaved(|s| match grab {
            Grab::Resize => {
                let rect = Self::place(atlas, &s.mods, &readings, screen, m);
                let [bx, by, ..] = rect;
                let [sx, sy, sw, sh] = Self::hit_box(rect, &s.mods, m);
                let scale = s.mods.place(m).scale * ((x - bx) / (sx + sw - bx)).max((y - by) / (sy + sh - by));
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
        if let Some(utility) = &mut self.utility {utility.release();}
        if (self.held.take(), self.slider.take()) != (None, None) {
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
        if self.utility.is_some() {return Outcome::None;}
        if self.options.is_none() && !self.editing {
            self.catalogue_scroll=(self.catalogue_scroll as isize-amount.signum() as isize).clamp(0,self.max_catalogue_scroll(size.0,size.1) as isize) as usize;
            return Outcome::None;
        }
        let mods = client_settings::get().mods;
        if self.editing {
            let screen = Self::screen(size);
            let readings = self.readings(self.world.get());
            if let Some(m) = self.mod_at(atlas, &mods, &readings, screen, cursor) {
                self.selected = Some(m);
                client_settings::update(|s| {
                    let scale = s.mods.place(m).scale + amount.signum() * SCALE_STEP;
                    Self::resize(atlas, &mut s.mods, &readings, screen, m, scale);
                });
            }
            return Outcome::None;
        }
        if self.options.is_some() {
            let [x, y, pw, ph] = Self::dialog(size.0, size.1);
            let left = Self::option_list_width(pw, ph);
            let view_height = ph - OPTIONS_TOP - OPTIONS_BOTTOM;
            if ui::hovered(cursor, x as f32, (y + OPTIONS_TOP) as f32, (left + 24) as f32, view_height as f32) {
                self.dropdown = None;
                let max = self.max_options_scroll(view_height);
                self.options_scroll = (self.options_scroll.min(max) - (amount.signum() * ROW_STEP as f32) as i32).clamp(0, max);
            }
        }
        Outcome::None
    }
}

fn switch_label(on: bool) -> String {
    if on { tr("patina.value.on", "On") } else { tr("patina.value.off", "Off") }
}

pub(crate) fn draw_label(batch: &mut HudBatch, atlas: &HudAtlas, label: &Text, rect: [f32; 4], color: u32, scale: f32, centered: bool) {
    let [x, y, w, h] = rect;
    let start = batch.vertices.len();
    if centered {
        ui::centered_text(batch, atlas, label, 0.0, 0.0, w / scale, color);
    } else {
        let plain = label.plain();
        let count = atlas.font.fit(&plain, w / scale);
        batch.text_styled(atlas, &label.slice(0, count), 0.0, 0.0, color, true);
    }
    let y = y + (h - 8.0 * scale) / 2.0;
    for v in &mut batch.vertices[start..] {
        v.pos = [x + v.pos[0] * scale, y + v.pos[1] * scale];
    }
}

fn option_button(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], label: &Text, active: bool, hover: bool) {
    let [x, y, w, h] = rect;
    ui::button_face(batch, atlas, x, y, w, h, &Text::empty(), active, hover);
    let color = if active { u32::MAX } else { rgba(160, 160, 160, 255) };
    draw_label(batch, atlas, label, [x + 2.0, y, w - 4.0, h], color, OPTION_TEXT_SCALE, true);
}

fn draw_thumbnail(batch: &mut HudBatch, atlas: &HudAtlas, mods: &Mods, m: Mod, x: f32, y: f32) {
    let (w, h) = (THUMB_WIDTH as f32, THUMB_HEIGHT as f32);
    ui::plate(batch, atlas, [x, y, w, h], rgba(31, 33, 34, 255), LINE);
    if let Some(mut backdrop) = atlas.sprites.get(MENU_BACKDROP_SPRITE).copied() {
        let scale = ((w - 2.0) / backdrop.w as f32).max((h - 2.0) / backdrop.h as f32);
        let crop_x = (1.0 - (w - 2.0) / (scale * backdrop.w as f32)) * 0.5;
        let crop_y = 1.0 - (h - 2.0) / (scale * backdrop.h as f32);
        let inset = (backdrop.u1 - backdrop.u0) * crop_x;
        backdrop.u0 += inset;
        backdrop.u1 -= inset;
        backdrop.v0 += (backdrop.v1 - backdrop.v0) * crop_y;
        batch.rect_uv(x + 1.0, y + 1.0, w - 2.0, h - 2.0, backdrop, u32::MAX);
    }
    let readings = sample();
    let (mw, mh) = extras_hud::mod_size(atlas, mods, &readings, m);
    let zoom = 0.95_f32.min((w - 12.0) / mw.max(1.0)).min((h - 12.0) / mh.max(1.0));
    let mut preview = mods.clone();
    preview.place_mut(m).scale = zoom;
    let rect = [x + (w - mw * zoom) / 2.0, y + (h - mh * zoom) / 2.0, mw * zoom, mh * zoom];
    extras_hud::draw_mod(batch, atlas, &preview, &readings, m, rect, 1.0);
}

fn rounded_fill(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], color: u32, top: u8, bottom: u8) {
    let [x, y, w, h] = rect;
    batch.fill(atlas, x, y + f32::from(top), w, h - f32::from(top + bottom), color);
    for inset in 1..=top {
        let dx = f32::from(inset);
        batch.fill(atlas, x + dx, y + f32::from(top - inset), w - 2.0 * dx, 1.0, color);
    }
    for inset in 1..=bottom {
        let dx = f32::from(inset);
        batch.fill(atlas, x + dx, y + h - f32::from(bottom - inset + 1), w - 2.0 * dx, 1.0, color);
    }
}

fn rounded_plate(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], fill: u32, edge: u32, top: bool, bottom: bool) {
    let [x, y, w, h] = rect;
    rounded_fill(batch, atlas, rect, edge, 2 * u8::from(top), 2 * u8::from(bottom));
    rounded_fill(batch, atlas, [x + 1.0, y + 1.0, w - 2.0, h - 2.0], fill, u8::from(top), u8::from(bottom));
}

fn chevron(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, direction: i32) {
    for d in -2_i32..=2 {
        let offset = 1 - d.abs();
        let (dx, dy) = if direction == 0 { (offset, d) } else { (d, direction * offset) };
        batch.fill(atlas, x + dx as f32, y + dy as f32, 1.0, 1.0, HEADING);
    }
}

pub(crate) fn reset_face(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], label: &Text, scale: f32, hover: bool) {
    let [x, y, w, h] = rect;
    let edge = if hover { HEADING } else { rgba(71, 91, 89, 255) };
    let fill = if hover { rgba(36, 57, 55, 247) } else { rgba(17, 24, 27, 240) };
    rounded_plate(batch, atlas, rect, fill, edge, true, true);
    let text_width = atlas.font.width_text(label) * scale;
    let icon_x = x + ((w - 13.0 - text_width) / 2.0).floor();
    reset_icon(batch, atlas, icon_x, y + 3.0, if hover { HEADING } else { rgba(216, 238, 234, 255) });
    draw_label(batch, atlas, label, [icon_x + 13.0, y, w - 13.0, h], rgba(233, 243, 240, 255), scale, false);
}

pub(crate) fn draw_switch(batch: &mut HudBatch, atlas: &HudAtlas, rect: [f32; 4], on: bool, hover: bool, scale: f32, rounded: bool) {
    let [x, y, w, h] = rect;
    let half = (w / 2.0).floor();
    let (left, width, color) = if on { (x + half, w - half, rgba(38, 117, 99, 255)) }
        else { (x, half, rgba(74, 80, 84, 255)) };
    let edge = if hover { HEADING } else { LINE };
    if rounded {
        rounded_plate(batch, atlas, rect, rgba(25, 30, 33, 255), edge, false, true);
        batch.fill(atlas, left + 1.0, y + 1.0, width - 2.0, h - 3.0, color);
        batch.fill(atlas, left + if on { 1.0 } else { 2.0 }, y + h - 2.0, width - 3.0, 1.0, color);
    } else {
        ui::plate(batch, atlas, rect, rgba(25, 30, 33, 255), edge);
        batch.fill(atlas, left + 1.0, y + 1.0, width - 2.0, h - 2.0, color);
    }
    draw_label(batch, atlas, &Text::literal(switch_label(false)), [x, y, half, h], if on { HINT } else { u32::MAX }, scale, true);
    draw_label(batch, atlas, &Text::literal(switch_label(true)), [x + half, y, w - half, h], if on { u32::MAX } else { HINT }, scale, true);
}

fn mod_icon(batch: &mut HudBatch, atlas: &HudAtlas, module: Module, x: f32, y: f32, color: u32) {
    let m = match module {
        Module::Hud(m) => m,
        Module::Utility(utility) => {
            outline(batch,atlas,[x+3.0,y+3.0,18.0,18.0],color);
            let text=utility.key().chars().next().unwrap().to_ascii_uppercase().to_string();batch.text(atlas,&text,x+9.0,y+8.0,color,true);return;
        }
        Module::FullBright => {
            batch.fill(atlas, x + 7.0, y + 7.0, 10.0, 10.0, color);
            for i in 0..3 {
                let offset = 2.0 + i as f32 * 9.0;
                batch.fill(atlas, x + offset, y + 2.0, 2.0, 3.0, color);
                batch.fill(atlas, x + offset, y + 19.0, 2.0, 3.0, color);
            }
            batch.fill(atlas, x + 1.0, y + 11.0, 4.0, 2.0, color);
            batch.fill(atlas, x + 19.0, y + 11.0, 4.0, 2.0, color);
            return;
        }
    };
    match m {
        Mod::Fps => for i in 0..3 {
            batch.fill(atlas, x + 3.0 + i as f32 * 7.0, y + 16.0 - i as f32 * 6.0, 4.0, 7.0 + i as f32 * 6.0, color);
        },
        Mod::Keystrokes => {
            outline(batch, atlas, [x + 8.0, y, 8.0, 8.0], color);
            for i in 0..3 { outline(batch, atlas, [x + i as f32 * 8.0, y + 9.0, 8.0, 8.0], color); }
            batch.fill(atlas, x + 1.0, y + 20.0, 22.0, 2.0, color);
        }
        Mod::Coordinates => {
            outline(batch, atlas, [x + 7.0, y + 7.0, 10.0, 10.0], color);
            batch.fill(atlas, x + 11.0, y, 2.0, 9.0, color);
            batch.fill(atlas, x + 11.0, y + 15.0, 2.0, 9.0, color);
            batch.fill(atlas, x, y + 11.0, 9.0, 2.0, color);
            batch.fill(atlas, x + 15.0, y + 11.0, 9.0, 2.0, color);
        }
        Mod::Speedometer | Mod::Ping | Mod::Effects => {
            outline(batch,atlas,[x+3.0,y+3.0,18.0,18.0],color);let label=match m {Mod::Speedometer=>"S",Mod::Ping=>"P",_=>"E"};batch.text(atlas,label,x+9.0,y+8.0,color,true);
        }
        Mod::Armor => {
            batch.fill(atlas, x + 5.0, y + 3.0, 14.0, 4.0, color);
            batch.fill(atlas, x + 1.0, y + 5.0, 22.0, 6.0, color);
            batch.fill(atlas, x + 6.0, y + 11.0, 12.0, 11.0, color);
            batch.fill(atlas, x + 10.0, y + 2.0, 4.0, 4.0, PANEL);
        }
    }
}

fn reset_icon(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, color: u32) {
    for (row, bits) in RESET_ICON.iter().enumerate() {
        for column in 0..9 {
            if bits & (1 << (8 - column)) != 0 {
                batch.fill(atlas, x + column as f32, y + row as f32, 1.0, 1.0, color);
            }
        }
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
    if let Some(mut photo) = atlas.sprites.get(HUD_BACKDROP_SPRITE).copied() {
        // The supplied 1920×1140 image has a black strip below the 1080-pixel scene.
        let source_height = (photo.h as f32).min(1080.0);
        let scale = (w / photo.w as f32).max(h / source_height);
        let shown_w = w / scale;
        let shown_h = h / scale;
        let du = (photo.u1 - photo.u0) / photo.w as f32;
        let dv = (photo.v1 - photo.v0) / photo.h as f32;
        photo.u0 += (photo.w as f32 - shown_w) * du / 2.0;
        photo.u1 = photo.u0 + shown_w * du;
        photo.v0 += (source_height - shown_h) * dv / 2.0;
        photo.v1 = photo.v0 + shown_h * dv;
        batch.rect_uv(0.0, 0.0, w, h, photo, u32::MAX);
    } else {
        batch.fill(atlas, 0.0, 0.0, w, h, PANEL);
    }
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

    #[test]
    fn fullbright_is_searchable_and_its_second_row_fits_the_frame() {
        let mut screen = ClientScreen::default();
        for size in [(320.0, 240.0), (640.0, 360.0)] {
            let [fx, fy, fw, fh] = ClientScreen::catalogue_frame(size.0, size.1);
            let [x, y, w, h] = ClientScreen::tile(size.0, size.1, 4);
            assert!(x >= fx && y >= fy && x + w <= fx + fw && y + h <= fy + fh);
        }
        screen.search.set_value("fullbright");
        assert_eq!(screen.matching_mods().count(), 1);
        assert_eq!(screen.matching_mods().next().unwrap().0, Module::FullBright);
        let controls = screen.catalogue_controls(640.0, 360.0);
        assert!(controls.iter().any(|c| c.act == Act::FullBrightToggle));
        assert!(controls.iter().any(|c| c.act == Act::FullBrightTile));
        assert_eq!(Mod::ALL.len(), 7, "Rendering modules have no HUD position");
    }

    fn atlas() -> HudAtlas {
        HudAtlas::fonts(std::path::Path::new("/nonexistent/patina-test")).unwrap()
    }
    #[test]
    fn four_tiles_share_a_row_and_armor_options_scroll_into_view() {
        let size = (640.0, 360.0);
        let tiles: Vec<_> = (0..4).map(|i| ClientScreen::tile(size.0, size.1, i)).collect();
        assert!(tiles.windows(2).all(|pair| pair[0][1] == pair[1][1] && pair[0][0] + pair[0][2] < pair[1][0]));
        let mut screen = ClientScreen::default();
        screen.act(Act::Tile(Mod::Armor));
        let atlas = atlas();
        let small = (320.0, 240.0);
        let before = screen.controls(&Mods::default(), small.0, small.1);
        assert!(before.iter().any(|c| c.act == Act::Category("warnings")));
        assert!(!before.iter().any(|c| matches!(c.act, Act::Option(_))));
        for group in ["layout", "appearance", "warnings"] {
            screen.act(Act::Category(group));
        }
        for _ in 0..8 {
            screen.wheel(&atlas, (40.0, 100.0), small, -1.0);
        }
        let after = screen.controls(&Mods::default(), small.0, small.1);
        assert!(after.iter().any(|c| c.act == Act::Option(Setting::Threshold)));
        let mut mods = Mods::default();
        Setting::Vertical.apply(&mut mods);
        Setting::Durability.pick(&mut mods, 1);
        assert!(mods.armor.vertical);
        assert_eq!(mods.armor.display, "remaining");
        assert!(screen.close_subpage());
    }
    #[test]
    fn the_panel_is_the_fabric_mods_and_fits_the_smallest_screen() {
        assert_eq!(ClientScreen::panel(640.0, 360.0), [164, 22, 312, 288]);
        let screen = ClientScreen::default();
        for (w, h) in [(320.0, 240.0), (427.0, 240.0), (960.0, 540.0)] {
            let [px, py, pw, ph] = ClientScreen::catalogue_frame(w, h).map(|v| v as f32);
            assert!(px >= 0.0 && py >= 0.0 && px + pw <= w && py + ph <= h);
            for c in screen.controls(&Mods::default(), w, h) {
                let [x, y, cw, ch] = c.rect;
                let inside = x >= px && y >= py && x + cw <= px + pw && y + ch <= py + ph;
                assert!(inside, "{:?}", c.act);
            }
        }
        let controls = screen.controls(&Mods::default(), 480.0, 270.0);
        assert_eq!(controls[0].rect, [96.0, 44.0, 64.0, 39.0]);
        assert_eq!(controls[1].rect, [96.0, 83.0, 64.0, 14.0]);
        assert!(!controls.iter().any(|c| matches!(c.act, Act::Option(Setting::FpsAccuracy) | Act::Option(Setting::FpsStyle))));
    }
    #[test]
    fn fps_options_belong_to_the_tile_and_do_not_expose_the_catalogue() {
        let atlas = atlas();
        let size = (480.0, 270.0);
        let mut screen = ClientScreen::default();
        client_settings::update_unsaved(|s| *s = client_settings::ClientSettings::default());
        let initial = client_settings::get().mods;
        let point = |c: &Control| (c.rect[0] + 2.0, c.rect[1] + 2.0);
        let catalogue = screen.controls(&initial, size.0, size.1);
        let fps_tile = catalogue.iter().find(|c| c.act == Act::Tile(Mod::Fps)).unwrap();
        let edit = (size.0 - 30.0, size.1 / 2.0);
        assert_eq!(screen.click(&atlas, point(fps_tile), size, false), Outcome::Pressed);
        assert_eq!(screen.options, Some(Mod::Fps));
        assert_eq!(client_settings::get().mods.fps.enabled, initial.fps.enabled);
        let popup = screen.controls(&initial, size.0, size.1);
        assert_eq!(popup.len(), 6);
        assert!(!popup.iter().any(|c| matches!(c.act, Act::Tile(_) | Act::Toggle(_))));
        assert_eq!(screen.click(&atlas, edit, size, false), Outcome::None);
        assert!(!screen.editing());
        let accuracy = point(popup.iter().find(|c| c.act == Act::Option(Setting::FpsAccuracy)).unwrap());
        screen.click(&atlas, accuracy, size, false);
        assert_eq!(client_settings::get().mods.fps.accuracy, initial.fps.accuracy);
        let popup = screen.controls(&initial, size.0, size.1);
        let precise = point(popup.iter().find(|c| c.act == Act::Pick(Setting::FpsAccuracy, 1)).unwrap());
        screen.click(&atlas, precise, size, false);
        assert_ne!(client_settings::get().mods.fps.accuracy, initial.fps.accuracy);
        assert!(screen.close_subpage());
        assert!(screen.options.is_none());
        assert!(!screen.close_subpage());
        let toggle = point(catalogue.iter().find(|c| c.act == Act::Toggle(Mod::Fps)).unwrap());
        screen.click(&atlas, toggle, size, false);
        assert_ne!(client_settings::get().mods.fps.enabled, initial.fps.enabled);
        assert!(screen.options.is_none());
    }
    #[test]
    fn editor_back_and_escape_return_to_the_open_module() {
        let atlas = atlas();
        let size = (480.0, 270.0);
        let mut screen = ClientScreen::default();
        screen.act(Act::Tile(Mod::Armor));
        screen.act(Act::Category("layout"));
        screen.options_scroll = ROW_STEP;
        let mods = Mods::default();
        let preview = screen.controls(&mods, size.0, size.1).into_iter()
            .find(|c| c.act == Act::Edit).unwrap();
        let point = |c: &Control| (c.rect[0] + 2.0, c.rect[1] + 2.0);
        assert_eq!(screen.click(&atlas, point(&preview), size, false), Outcome::Pressed);
        assert!(screen.editing());
        assert!(!mods.enabled(Mod::Armor));
        assert!(screen.shown(&mods, Mod::Armor));
        let back = screen.controls(&mods, size.0, size.1).into_iter()
            .find(|c| c.act == Act::BackFromEditor).unwrap();
        assert_eq!(screen.click(&atlas, point(&back), size, false), Outcome::Pressed);
        assert!(!screen.editing());
        assert_eq!(screen.options, Some(Mod::Armor));
        assert_eq!(screen.options_scroll, ROW_STEP);
        assert!(!screen.collapsed.contains("layout"));
        screen.act(Act::Edit);
        assert!(screen.close_subpage());
        assert!(!screen.editing());
        assert_eq!(screen.options, Some(Mod::Armor));
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
        // Even a HUD element covering the button cannot steal its reset click.
        client_settings::update_unsaved(|s| {
            let (w, h) = extras_hud::mod_size(&atlas, &s.mods, &r, Mod::Keystrokes);
            extras_hud::move_mod(s.mods.place_mut(Mod::Keystrokes), [reset.0, reset.1, w, h], size);
        });
        assert_eq!(screen.click(&atlas, reset, size, false), Outcome::Pressed);
        let default = Mods::default().keystrokes.place;
        assert_eq!(client_settings::get().mods.keystrokes.place, default);
        screen.leave_editor();
        assert!(!screen.editing());
    }
}
