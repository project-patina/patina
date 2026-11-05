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
    HotbarLeft,
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
/// A rendering module has no HUD position or size.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToggleMod {
    pub enabled: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeysMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub place: Place,
    pub show_mouse: bool,
    pub show_space: bool,
    pub show_shift: bool,
    pub show_labels: bool,
}
impl Default for KeysMod {
    fn default() -> Self { Self { enabled: false, place: Place::at(Anchor::BottomRight), show_mouse: true, show_space: true, show_shift: true, show_labels: true } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CoordinatesMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub place: Place,
    pub show_direction: bool,
    pub show_angle: bool,
    pub show_biome: bool,
    pub background: bool,
}
impl Default for CoordinatesMod {
    fn default() -> Self { Self { enabled: false, place: Place::at(Anchor::TopRight), show_direction: true, show_angle: false, show_biome: false, background: false } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArmorMod {
    pub enabled: bool,
    #[serde(flatten)]
    pub place: Place,
    pub vertical: bool,
    pub theme: String,
    pub hide_empty: bool,
    pub display: String,
    pub dynamic_color: bool,
    pub warning: bool,
    pub warning_threshold: u8,
    hotbar_anchor_migrated: bool,
}
impl Default for ArmorMod {
    fn default() -> Self { Self { enabled: false, place: Place::at(Anchor::HotbarLeft), vertical: false, theme: "minecraft".into(), hide_empty: false, display: "bar".into(), dynamic_color: true, warning: true, warning_threshold: 20, hotbar_anchor_migrated: true } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InfoMod {
    pub enabled: bool,
    #[serde(flatten)] pub place: Place,
    pub background: bool,
    pub unit: String,
}
impl Default for InfoMod {
    fn default() -> Self { Self { enabled: false, place: Place::default(), background: false, unit: "m/s".into() } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsMod {
    pub enabled: bool,
    #[serde(flatten)] pub place: Place,
    pub columns: u8,
    pub show_duration: bool,
    pub show_amplifier: bool,
}
impl Default for EffectsMod {
    fn default() -> Self { Self { enabled: false, place: Place::at(Anchor::CenterRight), columns: 2, show_duration: true, show_amplifier: true } }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Utility { AppleSkin, ShulkerTooltip, Zoom, Perspective, HorseStats, Weather, DeathInfo, NoFog }
impl Utility {
    pub const ALL: [Self; 8] = [Self::AppleSkin, Self::ShulkerTooltip, Self::Zoom, Self::Perspective, Self::HorseStats, Self::Weather, Self::DeathInfo, Self::NoFog];
    pub fn key(self) -> &'static str { match self { Self::AppleSkin => "appleskin", Self::ShulkerTooltip => "shulker_tooltip", Self::Zoom => "zoom", Self::Perspective => "perspective", Self::HorseStats => "horse_stats", Self::Weather => "weather", Self::DeathInfo => "death_info", Self::NoFog => "no_fog" } }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UtilityMod {
    pub enabled: bool, pub key: i32, pub activation: String, pub factor: f32,
    pub smooth: bool, pub sensitivity: f32, pub weather: String, pub strength: f32, pub require_shift: bool,
}
impl Default for UtilityMod {
    fn default() -> Self { Self { enabled: false, key: 67, activation: "hold".into(), factor: 4.0, smooth: true, sensitivity: 0.25, weather: "clear".into(), strength: 1.0, require_shift: false } }
}
/// GLFW key codes shared with the Fabric configuration.
pub fn glfw_key(code: winit::keyboard::KeyCode) -> i32 {
    use winit::keyboard::KeyCode as K;
    let name = format!("{code:?}");
    if let Some(letter) = name.strip_prefix("Key").filter(|s| s.len() == 1) { return letter.as_bytes()[0] as i32; }
    if let Some(digit) = name.strip_prefix("Digit").filter(|s| s.len() == 1) { return digit.as_bytes()[0] as i32; }
    if let Some(number) = name.strip_prefix('F').and_then(|s| s.parse::<i32>().ok()) { if (1..=25).contains(&number) { return 289 + number; } }
    match code { K::CapsLock => 280, K::ScrollLock => 281, K::NumLock => 282, K::PrintScreen => 283, K::Pause => 284, K::Numpad0 => 320, K::Numpad1 => 321, K::Numpad2 => 322, K::Numpad3 => 323, K::Numpad4 => 324, K::Numpad5 => 325, K::Numpad6 => 326, K::Numpad7 => 327, K::Numpad8 => 328, K::Numpad9 => 329, K::NumpadDecimal => 330, K::NumpadDivide => 331, K::NumpadMultiply => 332, K::NumpadSubtract => 333, K::NumpadAdd => 334, K::NumpadEnter => 335, K::NumpadEqual => 336, K::ContextMenu => 348, K::Space => 32, K::Escape => 256, K::Enter => 257, K::Tab => 258, K::Backspace => 259, K::Insert => 260, K::Delete => 261, K::ArrowRight => 262, K::ArrowLeft => 263, K::ArrowDown => 264, K::ArrowUp => 265, K::PageUp => 266, K::PageDown => 267, K::Home => 268, K::End => 269, K::ShiftLeft => 340, K::ControlLeft => 341, K::AltLeft => 342, K::SuperLeft => 343, K::ShiftRight => 344, K::ControlRight => 345, K::AltRight => 346, K::SuperRight => 347, K::Minus => 45, K::Equal => 61, K::BracketLeft => 91, K::BracketRight => 93, K::Backslash => 92, K::Semicolon => 59, K::Quote => 39, K::Comma => 44, K::Period => 46, K::Slash => 47, K::Backquote => 96, _ => -1 }
}

/// The built-in HUD modules, in their drawing and catalogue order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mod {
    Fps,
    Keystrokes,
    Coordinates,
    Armor,
    Speedometer, Ping, Effects,
}
impl Mod {
    pub const ALL: [Mod; 7] = [Mod::Fps, Mod::Keystrokes, Mod::Coordinates, Mod::Armor, Mod::Speedometer, Mod::Ping, Mod::Effects];
    fn key(self) -> &'static str {
        match self {
            Mod::Fps => "fps",
            Mod::Keystrokes => "keystrokes",
            Mod::Coordinates => "coordinates",
            Mod::Armor => "armor",
            Mod::Speedometer => "speedometer", Mod::Ping => "ping", Mod::Effects => "status_effects",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mods {
    pub fps: FpsMod,
    pub keystrokes: KeysMod,
    pub coordinates: CoordinatesMod,
    pub armor: ArmorMod,
    pub fullbright: ToggleMod,
    pub speedometer: InfoMod, pub ping: InfoMod, pub status_effects: EffectsMod,
    pub appleskin: UtilityMod, pub shulker_tooltip: UtilityMod, pub zoom: UtilityMod,
    pub perspective: UtilityMod, pub horse_stats: UtilityMod, pub weather: UtilityMod, pub death_info: UtilityMod, pub no_fog: UtilityMod,
}
impl Default for Mods {
    fn default() -> Self {
        Self {
            fps: FpsMod::default(),
            keystrokes: KeysMod::default(),
            coordinates: CoordinatesMod::default(),
            armor: ArmorMod::default(),
            fullbright: ToggleMod::default(),
            speedometer: InfoMod { place: Place::at(Anchor::BottomLeft), ..Default::default() }, ping: InfoMod::default(), status_effects: EffectsMod::default(),
            appleskin: UtilityMod::default(), shulker_tooltip: UtilityMod::default(), zoom: UtilityMod::default(),
            perspective: UtilityMod { key: 86, ..Default::default() }, horse_stats: UtilityMod::default(), weather: UtilityMod::default(), death_info: UtilityMod::default(), no_fog: UtilityMod::default(),
        }
    }
}
impl Mods {
    pub fn utility(&self, m: Utility) -> &UtilityMod { match m { Utility::AppleSkin => &self.appleskin, Utility::ShulkerTooltip => &self.shulker_tooltip, Utility::Zoom => &self.zoom, Utility::Perspective => &self.perspective, Utility::HorseStats => &self.horse_stats, Utility::Weather => &self.weather, Utility::DeathInfo => &self.death_info, Utility::NoFog => &self.no_fog } }
    pub fn utility_mut(&mut self, m: Utility) -> &mut UtilityMod { match m { Utility::AppleSkin => &mut self.appleskin, Utility::ShulkerTooltip => &mut self.shulker_tooltip, Utility::Zoom => &mut self.zoom, Utility::Perspective => &mut self.perspective, Utility::HorseStats => &mut self.horse_stats, Utility::Weather => &mut self.weather, Utility::DeathInfo => &mut self.death_info, Utility::NoFog => &mut self.no_fog } }

    pub fn enabled(&self, m: Mod) -> bool {
        match m {
            Mod::Fps => self.fps.enabled,
            Mod::Keystrokes => self.keystrokes.enabled,
            Mod::Coordinates => self.coordinates.enabled,
            Mod::Armor => self.armor.enabled,
            Mod::Speedometer => self.speedometer.enabled, Mod::Ping => self.ping.enabled, Mod::Effects => self.status_effects.enabled,
        }
    }
    pub fn enabled_mut(&mut self, m: Mod) -> &mut bool {
        match m {
            Mod::Fps => &mut self.fps.enabled,
            Mod::Keystrokes => &mut self.keystrokes.enabled,
            Mod::Coordinates => &mut self.coordinates.enabled,
            Mod::Armor => &mut self.armor.enabled,
            Mod::Speedometer => &mut self.speedometer.enabled, Mod::Ping => &mut self.ping.enabled, Mod::Effects => &mut self.status_effects.enabled,
        }
    }
    pub fn reset_options(&mut self, m: Mod) {
        let enabled = self.enabled(m);
        let place = self.place(m);
        match m {
            Mod::Fps => self.fps = FpsMod::default(),
            Mod::Keystrokes => self.keystrokes = KeysMod::default(),
            Mod::Coordinates => self.coordinates = CoordinatesMod::default(),
            Mod::Armor => self.armor = ArmorMod::default(),
            Mod::Speedometer => self.speedometer = InfoMod::default(), Mod::Ping => self.ping = InfoMod::default(), Mod::Effects => self.status_effects = EffectsMod::default(),
        }
        *self.enabled_mut(m) = enabled;
        *self.place_mut(m) = place;
    }
    pub fn place(&self, m: Mod) -> Place {
        match m {
            Mod::Fps => self.fps.place,
            Mod::Keystrokes => self.keystrokes.place,
            Mod::Coordinates => self.coordinates.place,
            Mod::Armor => self.armor.place,
            Mod::Speedometer => self.speedometer.place, Mod::Ping => self.ping.place, Mod::Effects => self.status_effects.place,
        }
    }
    pub fn place_mut(&mut self, m: Mod) -> &mut Place {
        match m {
            Mod::Fps => &mut self.fps.place,
            Mod::Keystrokes => &mut self.keystrokes.place,
            Mod::Coordinates => &mut self.coordinates.place,
            Mod::Armor => &mut self.armor.place,
            Mod::Speedometer => &mut self.speedometer.place, Mod::Ping => &mut self.ping.place, Mod::Effects => &mut self.status_effects.place,
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
            Mod::Armor => 32.0,
            Mod::Speedometer | Mod::Ping => 14.0, Mod::Effects => 60.0,
        };
        let enabled = settings.mods.enabled(m);
        let place = settings.mods.place_mut(m);
        if m == Mod::Armor && saved.get("hotbar_anchor_migrated").and_then(serde_json::Value::as_bool) != Some(true)
            && *place == Place::at(Anchor::BottomLeft)
        {
            place.position = Anchor::HotbarLeft;
        }
        if place.position == Anchor::HotbarLeft {
            place.x = 0.0;
            place.y = 0.0;
            if !(MIN_SCALE..=MAX_SCALE).contains(&place.scale) { place.scale = 1.0; }
            continue;
        }
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
    for utility in Utility::ALL {
        let m = settings.mods.utility_mut(utility);
        if !(1.5..=20.0).contains(&m.factor) { m.factor = 4.0; }
        if !(0.05..=1.0).contains(&m.sensitivity) { m.sensitivity = 0.25; }
        if !(0.0..=1.0).contains(&m.strength) { m.strength = 1.0; }
        if m.activation != "toggle" { m.activation = "hold".into(); }
        if !["clear", "rain", "thunder"].contains(&m.weather.as_str()) { m.weather = "clear".into(); }
        if !(-1..=348).contains(&m.key) { m.key = if utility == Utility::Perspective { 86 } else { 67 }; }
    }
    settings.mods.status_effects.columns = settings.mods.status_effects.columns.clamp(1, 4);
    if settings.mods.speedometer.unit != "km/h" { settings.mods.speedometer.unit = "m/s".into(); }
    settings.mods.armor.hotbar_anchor_migrated = true;
    settings
}
/// Drops the values a mod's fields can't take, so one broken value never
/// costs the rest of the file.
fn sanitize(mods: &mut serde_json::Value) {
    use serde_json::Value;
    if let Some(value) = mods.get("fullbright")
        && serde_json::from_value::<ToggleMod>(value.clone()).is_err()
        && let Some(map) = mods.as_object_mut()
    {
        map.remove("fullbright");
    }
    for utility in Utility::ALL {
        if let Some(value) = mods.get(utility.key()) && serde_json::from_value::<UtilityMod>(value.clone()).is_err()
            && let Some(map) = mods.as_object_mut() { map.remove(utility.key()); }
    }
    for m in Mod::ALL {
        let Some(Value::Object(fields)) = mods.get_mut(m.key()) else {
            if let Some(map) = mods.as_object_mut() {
                map.remove(m.key());
            }
            continue;
        };
        fields.retain(|key, value| match key.as_str() {
            "enabled" | "show_mouse" | "show_space" | "show_shift" | "show_labels" | "show_direction" | "show_angle" | "show_biome" | "background" | "vertical" | "hide_empty" | "dynamic_color" | "warning" | "show_duration" | "show_amplifier" => value.is_boolean(),
            "unit" => value.as_str().is_some_and(|s| ["m/s", "km/h"].contains(&s)),
            "columns" => value.as_u64().is_some_and(|v| (1..=4).contains(&v)),
            "theme" => value.as_str().is_some_and(|s| ["minecraft", "patina"].contains(&s)),
            "display" => value.as_str().is_some_and(|s| ["bar", "remaining"].contains(&s)),
            "warning_threshold" => value.as_u64().is_some_and(|v| (5..=50).contains(&v)),
            "hotbar_anchor_migrated" => m == Mod::Armor && value.is_boolean(),
            "x" | "y" | "scale" => value.is_number(),
            "position" => serde_json::from_value::<Anchor>(value.clone())
                .is_ok_and(|anchor| anchor != Anchor::HotbarLeft || m == Mod::Armor),
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
    fn utility_modules_round_trip_and_invalid_values_do_not_reset_existing_hud() {
        let settings=parse(r#"{"mods":{"zoom":{"enabled":true,"key":90,"activation":"toggle","factor":8.5,"smooth":false,"sensitivity":0.2},"perspective":{"enabled":true,"key":66,"activation":"hold","factor":6},"weather":{"enabled":true,"weather":"thunder","strength":0.75},"speedometer":{"enabled":true,"unit":"km/h"},"status_effects":{"enabled":true,"columns":3},"fps":{"position":"center","x":12,"y":-8}}}"#);
        assert_eq!(settings.mods.zoom.key,90);assert_eq!(settings.mods.perspective.key,66);
        assert_eq!(settings.mods.zoom.activation,"toggle");assert_eq!(settings.mods.weather.weather,"thunder");
        assert_eq!(settings.mods.speedometer.unit,"km/h");assert_eq!(settings.mods.status_effects.columns,3);
        assert_eq!(parse(&serde_json::to_string(&settings).unwrap()),settings);
        let broken=parse(r#"{"mods":{"zoom":{"enabled":"broken"},"perspective":null,"fps":{"position":"center","x":12,"y":-8}}}"#);
        assert_eq!(broken.mods.fps,settings.mods.fps);assert!(!broken.mods.zoom.enabled);assert_eq!(broken.mods.perspective.key,86);
    }

    #[test]
    fn fullbright_migrates_and_round_trips_independently_of_hud() {
        let old = parse(r#"{"mods":{"fps":{"position":"center","x":12,"y":-8}}}"#);
        assert!(!old.mods.fullbright.enabled);
        let enabled = parse(r#"{"mods":{"fullbright":{"enabled":true},"fps":{"position":"center","x":12,"y":-8}}}"#);
        assert!(enabled.mods.fullbright.enabled);
        assert_eq!(old.mods.fps, enabled.mods.fps);
        assert_eq!(parse(&serde_json::to_string(&enabled).unwrap()), enabled);
        let broken = parse(r#"{"mods":{"fullbright":{"enabled":"broken"},"fps":{"position":"center","x":12,"y":-8}}}"#);
        assert_eq!(broken, old);
    }

    #[test]
    fn module_options_round_trip_without_moving_existing_hud() {
        let original = parse(r#"{"mods":{"fps":{"enabled":false,"position":"center","x":12,"y":-8},"keystrokes":{"enabled":true,"show_space":false,"show_shift":false},"coordinates":{"show_biome":true,"show_angle":true},"armor":{"enabled":true,"vertical":true,"theme":"patina","display":"remaining","hide_empty":true,"dynamic_color":false,"warning_threshold":30}}}"#);
        assert!(!original.mods.fps.enabled);
        assert_eq!((original.mods.fps.place.x, original.mods.fps.place.y), (12.0, -8.0));
        assert!(!original.mods.keystrokes.show_space && !original.mods.keystrokes.show_shift);
        assert!(original.mods.coordinates.show_biome && original.mods.coordinates.show_angle);
        assert_eq!(original.mods.armor.warning_threshold, 30);
        assert_eq!(parse(&serde_json::to_string(&original).unwrap()), original);
        let repaired = parse(r#"{"mods":{"armor":{"display":"broken","theme":null,"warning_threshold":999},"keystrokes":{"show_space":"broken"}}}"#);
        assert_eq!(repaired.mods.armor, ArmorMod::default());
        assert!(repaired.mods.keystrokes.show_space);
    }

    #[test]
    fn only_the_old_armor_default_moves_to_the_hotbar() {
        let mut old = parse(r#"{"mods":{"armor":{"position":"bottom_left","x":0,"y":0,"scale":1}}}"#);
        assert_eq!(old.mods.armor.place.position, Anchor::HotbarLeft);
        old.mods.armor.place = Place::at(Anchor::BottomLeft);
        assert_eq!(parse(&serde_json::to_string(&old).unwrap()).mods.armor.place, old.mods.armor.place);
        let moved = parse(r#"{"mods":{"armor":{"position":"bottom_left","x":14,"y":0,"scale":1}}}"#);
        assert_eq!(moved.mods.armor.place.position, Anchor::BottomLeft);
        assert_eq!(moved.mods.armor.place.x, 14.0);
    }

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
