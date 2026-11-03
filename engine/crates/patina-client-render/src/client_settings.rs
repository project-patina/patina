//! The Patina client's own settings, `<game dir>/patina/client.json`: the three
//! built-in HUD mods, each with its place and size, written the way the Patina
//! Fabric mod writes them. Separate from `options.txt`, which stays as vanilla
//! writes it. The title background is fixed: the launcher's black Mono banner.
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
};

/// The launcher's Mono banner (`src/lib/presets.ts`): the sky and three
/// ridges' colours, and the seed of the ridges.
const MONO: ([u32; 4], u32) = ([0x1f2122, 0x2c2f30, 0x3a3e40, 0x4b5053], 83);
/// The banners' size in blocks: the launcher's 960x384 scene in 24 pixel blocks.
pub const SCENE_COLUMNS: usize = 40;
pub const SCENE_ROWS: usize = 16;
/// The smallest and largest size a mod can be given.
pub const MIN_SCALE: f32 = 0.5;
pub const MAX_SCALE: f32 = 2.5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FpsAccuracy {
    #[default]
    Whole,
    #[serde(rename = "precise", alias = "exact")]
    Exact,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FpsStyle {
    #[default]
    #[serde(rename = "text", alias = "plain")]
    Plain,
    Panel,
}
/// What a mod keeps to: a corner, the middle of an edge, or the middle of
/// the screen (`PatinaConfig.POSITIONS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
impl Anchor {
    pub const ALL: [Anchor; 9] = [
        Anchor::TopLeft,
        Anchor::TopCenter,
        Anchor::TopRight,
        Anchor::CenterLeft,
        Anchor::Center,
        Anchor::CenterRight,
        Anchor::BottomLeft,
        Anchor::BottomCenter,
        Anchor::BottomRight,
    ];
    fn index(self) -> i8 {
        Self::ALL.iter().position(|a| *a == self).unwrap_or(0) as i8
    }
    /// -1 keeps to the left edge, 1 to the right one, 0 to the middle.
    pub fn horizontal(self) -> i8 {
        self.index() % 3 - 1
    }
    /// -1 keeps to the top edge, 1 to the bottom one, 0 to the middle.
    pub fn vertical(self) -> i8 {
        self.index() / 3 - 1
    }
    pub fn from_sides(horizontal: i8, vertical: i8) -> Self {
        Self::ALL[((vertical.clamp(-1, 1) + 1) * 3 + horizontal.clamp(-1, 1) + 1) as usize]
    }
}
/// A mod's place: what it keeps to and its distance from there in GUI
/// pixels, so it stays put whatever the window size, and its own scale.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Place {
    pub position: Anchor,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}
impl Default for Place {
    fn default() -> Self {
        Self::at(Anchor::TopLeft)
    }
}
impl Place {
    pub const fn at(position: Anchor) -> Self {
        Self {
            position,
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FpsMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub place: Place,
    pub accuracy: FpsAccuracy,
    pub style: FpsStyle,
}
impl Default for FpsMod {
    fn default() -> Self {
        Self {
            enabled: true,
            place: Place::at(Anchor::TopLeft),
            accuracy: FpsAccuracy::Whole,
            style: FpsStyle::Plain,
        }
    }
}
/// A mod whose only settings are whether it shows and where.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlacedMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub place: Place,
}
/// The three mods, in the order they draw and the settings list them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mod {
    Fps,
    Keystrokes,
    Coordinates,
}
impl Mod {
    pub const ALL: [Mod; 3] = [Mod::Fps, Mod::Keystrokes, Mod::Coordinates];
    fn key(self) -> &'static str {
        match self {
            Mod::Fps => "fps",
            Mod::Keystrokes => "keystrokes",
            Mod::Coordinates => "coordinates",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mods {
    pub fps: FpsMod,
    pub keystrokes: PlacedMod,
    pub coordinates: PlacedMod,
}
impl Default for Mods {
    fn default() -> Self {
        Self {
            fps: FpsMod::default(),
            keystrokes: PlacedMod {
                enabled: false,
                place: Place::at(Anchor::BottomRight),
            },
            coordinates: PlacedMod {
                enabled: false,
                place: Place::at(Anchor::TopRight),
            },
        }
    }
}
impl Mods {
    pub fn enabled(&self, m: Mod) -> bool {
        match m {
            Mod::Fps => self.fps.enabled,
            Mod::Keystrokes => self.keystrokes.enabled,
            Mod::Coordinates => self.coordinates.enabled,
        }
    }
    pub fn enabled_mut(&mut self, m: Mod) -> &mut bool {
        match m {
            Mod::Fps => &mut self.fps.enabled,
            Mod::Keystrokes => &mut self.keystrokes.enabled,
            Mod::Coordinates => &mut self.coordinates.enabled,
        }
    }
    pub fn place(&self, m: Mod) -> Place {
        match m {
            Mod::Fps => self.fps.place,
            Mod::Keystrokes => self.keystrokes.place,
            Mod::Coordinates => self.coordinates.place,
        }
    }
    pub fn place_mut(&mut self, m: Mod) -> &mut Place {
        match m {
            Mod::Fps => &mut self.fps.place,
            Mod::Keystrokes => &mut self.keystrokes.place,
            Mod::Coordinates => &mut self.coordinates.place,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientSettings {
    pub mods: Mods,
}

/// An image the title background shows.
#[derive(Debug)]
pub struct BackdropImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// A banner's blocks: nearest filtering, anchored to the bottom.
    pub blocky: bool,
}

#[derive(Default)]
struct State {
    settings: ClientSettings,
    /// `<game dir>/patina`; settings are only saved once it is known.
    dir: Option<PathBuf>,
}
static STATE: LazyLock<RwLock<State>> = LazyLock::new(Default::default);
static BACKDROP: LazyLock<Arc<BackdropImage>> = LazyLock::new(|| Arc::new(banner_image()));

fn read() -> std::sync::RwLockReadGuard<'static, State> {
    STATE.read().unwrap_or_else(|e| e.into_inner())
}
fn write() -> std::sync::RwLockWriteGuard<'static, State> {
    STATE.write().unwrap_or_else(|e| e.into_inner())
}

/// What corners stacked by before mods had a place of their own.
const OLD_STACK_GAP: f32 = 3.0;

/// Parse `client.json`: what is missing or broken keeps its default, so a
/// file from another version never costs the rest. A file from before the
/// mods could be dragged keeps their old corner stacks.
pub fn parse(text: &str) -> ClientSettings {
    let value: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
    // Earlier Rust builds kept the mods at the top level.
    let mut mods = match value.get("mods") {
        Some(mods) => mods.clone(),
        None => value,
    };
    sanitize(&mut mods);
    let mut settings = ClientSettings {
        mods: serde_json::from_value(mods.clone()).unwrap_or_default(),
    };
    let defaults = Mods::default();
    let mut used = [0.0; 9];
    for m in Mod::ALL {
        let saved = &mods[m.key()];
        let placed = saved.get("x").is_some() && saved.get("y").is_some();
        if saved.get("position").is_none() {
            settings.mods.place_mut(m).position = defaults.place(m).position;
        }
        let height = match m {
            Mod::Fps if settings.mods.fps.style == FpsStyle::Panel => 14.0,
            Mod::Fps => 8.0,
            Mod::Keystrokes => 58.0,
            Mod::Coordinates => 18.0,
        };
        let enabled = settings.mods.enabled(m);
        let place = settings.mods.place_mut(m);
        let corner = Anchor::ALL
            .iter()
            .position(|c| *c == place.position)
            .unwrap_or(0);
        if !placed || !place.x.is_finite() || !place.y.is_finite() {
            place.x = 0.0;
            place.y = if enabled { used[corner] } else { 0.0 };
        }
        if enabled {
            used[corner] += height + OLD_STACK_GAP;
        }
        if !(MIN_SCALE..=MAX_SCALE).contains(&place.scale) {
            place.scale = 1.0;
        }
    }
    settings
}
/// Drops the values a mod's fields can't take, so one broken value never
/// costs the rest of the file.
fn sanitize(mods: &mut serde_json::Value) {
    use serde_json::Value;
    for m in Mod::ALL {
        let Some(Value::Object(fields)) = mods.get_mut(m.key()) else {
            if let Some(map) = mods.as_object_mut() {
                map.remove(m.key());
            }
            continue;
        };
        fields.retain(|key, value| match key.as_str() {
            "enabled" => value.is_boolean(),
            "x" | "y" | "scale" => value.is_number(),
            "position" => serde_json::from_value::<Anchor>(value.clone()).is_ok(),
            "accuracy" => serde_json::from_value::<FpsAccuracy>(value.clone()).is_ok(),
            "style" => serde_json::from_value::<FpsStyle>(value.clone()).is_ok(),
            _ => false,
        });
    }
}
/// Load `<game_dir>/patina/client.json`.
pub fn init(game_dir: &Path) {
    let dir = game_dir.join("patina");
    let settings = std::fs::read_to_string(dir.join("client.json"))
        .map(|text| parse(&text))
        .unwrap_or_default();
    let mut state = write();
    state.settings = settings;
    state.dir = Some(dir);
}
pub fn get() -> ClientSettings {
    read().settings.clone()
}
/// Change the settings and save them.
pub fn update(change: impl FnOnce(&mut ClientSettings)) {
    let (dir, text) = {
        let mut state = write();
        change(&mut state.settings);
        (
            state.dir.clone(),
            serde_json::to_string_pretty(&state.settings),
        )
    };
    if let (Some(dir), Ok(text)) = (dir, text) {
        let written = std::fs::create_dir_all(&dir)
            .and_then(|()| std::fs::write(dir.join("client.json"), text));
        if let Err(error) = written {
            log::warn!("Patina: could not save client.json: {error}");
        }
    }
}
/// Change the settings without saving, while a mod is being dragged.
pub fn update_unsaved(change: impl FnOnce(&mut ClientSettings)) {
    change(&mut write().settings);
}
/// The title background: its generation and picture.
pub fn backdrop() -> (u64, Option<Arc<BackdropImage>>) {
    (1, Some(BACKDROP.clone()))
}

/// The launcher's `ridge`: one skyline's top per block column, in block rows.
pub fn ridge(seed: u32, base: i32, swing: i32) -> [i32; SCENE_COLUMNS] {
    let mut heights = [0; SCENE_COLUMNS];
    let (mut x, mut y, mut state) = (0usize, base, f64::from(seed));
    while x < SCENE_COLUMNS {
        // As JavaScript computes it, in doubles, so the skylines match.
        state = (state * 1103515245.0 + 12345.0) % 2147483648.0;
        let bits = state as u32;
        let run = 2 + (bits >> 8) as usize % 5;
        let step = ((bits >> 16) % 3) as i32 - 1;
        let end = (x + run).min(SCENE_COLUMNS);
        heights[x..end].fill(y);
        x = end;
        y = (y + step).clamp(base - swing, base + swing);
    }
    heights
}
/// The Mono banner one texel per block.
pub fn banner_image() -> BackdropImage {
    let (colors, seed) = MONO;
    let color = |c: u32| [(c >> 16) as u8, (c >> 8) as u8, c as u8, 255];
    let mut pixels = color(colors[0]).repeat(SCENE_COLUMNS * SCENE_ROWS);
    let ridges = [
        (ridge(seed, 9, 2), colors[1]),
        (ridge(seed + 7, 11, 2), colors[2]),
        (ridge(seed + 13, 13, 1), colors[3]),
    ];
    for (heights, c) in ridges {
        for (x, top) in heights.into_iter().enumerate() {
            for y in top.max(0) as usize..SCENE_ROWS {
                pixels[(y * SCENE_COLUMNS + x) * 4..][..4].copy_from_slice(&color(c));
            }
        }
    }
    BackdropImage {
        width: SCENE_COLUMNS as u32,
        height: SCENE_ROWS as u32,
        pixels,
        blocky: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_foreign_fields_keep_their_defaults() {
        let s = parse(r#"{"background":"sunset","mods":{"fps":{"style":"panel"}},"future":1}"#);
        assert_eq!(s.mods.fps.style, FpsStyle::Panel);
        assert!(s.mods.fps.enabled);
        assert!(!s.mods.keystrokes.enabled);
        assert_eq!(s.mods.coordinates.place.position, Anchor::TopRight);
        assert_eq!(parse("not json"), ClientSettings::default());
        // One broken value keeps the rest, and an earlier Rust file still reads.
        let s = parse(r#"{"mods":{"keystrokes":{"enabled":true,"position":"middle"}}}"#);
        assert!(s.mods.keystrokes.enabled);
        assert_eq!(s.mods.keystrokes.place.position, Anchor::BottomRight);
        let s = parse(r#"{"fps":{"accuracy":"exact","style":"plain","position":"top_right"}}"#);
        assert_eq!(s.mods.fps.accuracy, FpsAccuracy::Exact);
        assert_eq!(s.mods.fps.place.position, Anchor::TopRight);
    }
    #[test]
    fn the_fabric_mods_file_reads_the_same() {
        let s = parse(
            r#"{"mods":{"fps":{"enabled":true,"position":"bottom_left","x":12.0,"y":3.5,"scale":1.5,"accuracy":"precise","style":"text"},
            "keystrokes":{"enabled":true,"position":"top_right","x":0.0,"y":40.0,"scale":9.0}}}"#,
        );
        let fps = &s.mods.fps;
        assert_eq!(
            fps.place,
            Place {
                position: Anchor::BottomLeft,
                x: 12.0,
                y: 3.5,
                scale: 1.5
            }
        );
        assert_eq!(
            (fps.accuracy, fps.style),
            (FpsAccuracy::Exact, FpsStyle::Plain)
        );
        // Out of range sizes go back to 1.
        assert_eq!(s.mods.keystrokes.place.scale, 1.0);
        let text = serde_json::to_string(&s).unwrap();
        assert!(text.contains(r#""accuracy":"precise""#) && text.contains(r#""style":"text""#));
        assert_eq!(parse(&text), s);
    }
    #[test]
    fn an_old_file_keeps_its_corner_stacks() {
        let s = parse(
            r#"{"mods":{"fps":{"enabled":true,"position":"top_left"},
            "coordinates":{"enabled":true,"position":"top_left"}}}"#,
        );
        assert_eq!((s.mods.fps.place.x, s.mods.fps.place.y), (0.0, 0.0));
        assert_eq!(
            (s.mods.coordinates.place.x, s.mods.coordinates.place.y),
            (0.0, 11.0)
        );
    }
    #[test]
    fn whole_numbers_and_the_panel_style_read() {
        let s = parse(
            r#"{"mods": {"fps": {"enabled": true, "position": "top_left", "x": 0, "y": 0, "scale": 1, "accuracy": "whole", "style": "panel"}}}"#,
        );
        assert_eq!(s.mods.fps.style, FpsStyle::Panel);
        assert_eq!(s.mods.fps.place.scale, 1.0);
    }
    #[test]
    fn anchors_name_their_sides() {
        let sides = |a: Anchor| (a.horizontal(), a.vertical());
        assert_eq!(sides(Anchor::TopLeft), (-1, -1));
        assert_eq!(sides(Anchor::Center), (0, 0));
        assert_eq!(Anchor::from_sides(1, 0), Anchor::CenterRight);
        assert_eq!(Anchor::from_sides(0, 1), Anchor::BottomCenter);
        let name = serde_json::to_string(&Anchor::CenterLeft).unwrap();
        assert_eq!(name, r#""center_left""#);
    }
    #[test]
    fn the_banner_is_the_launchers_mono() {
        let far = ridge(83, 9, 2);
        assert!(far.iter().all(|h| (7..=11).contains(h)));
        let image = banner_image();
        assert_eq!(image.pixels.len(), SCENE_COLUMNS * SCENE_ROWS * 4);
        assert_eq!(image.pixels[..4], [0x1f, 0x21, 0x22, 255]);
    }
}
