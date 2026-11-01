use serde::{Deserialize, Serialize};

use crate::{Ctx, Result};

/// Global launcher settings. Every field has a default so older files keep loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    // Appearance
    /// `dark`, `light` or `system`.
    pub theme: String,
    /// Colour palette, independent of light/dark mode.
    pub palette: String,
    /// Accent colour as `#rrggbb`.
    pub accent: String,
    pub animations: bool,
    pub blur: bool,
    /// UI zoom, 0.8–1.25.
    pub ui_scale: f32,
    /// `inter`, `outfit`, `system` or `mono`.
    pub font: String,
    /// `sharp`, `normal` or `round`.
    pub radius: String,
    pub compact: bool,

    // Home hero + wallpapers. Image refs: "" (instance banner), `preset:<key>`, `custom:<id>`.
    pub hero_wallpaper: String,
    /// Show the living village behind the player instead of the wallpaper.
    pub hero_scene: bool,
    /// Darkening over the hero image, 0–80 %.
    pub hero_dim: u32,
    pub app_wallpaper: String,
    pub app_wallpaper_dim: u32,
    /// `center`, `left`, `right`.
    pub model_position: String,
    pub follow_cursor: bool,
    pub show_nametag: bool,

    // Versions
    pub show_snapshots: bool,
    pub show_old_versions: bool,
    pub default_loader: crate::loaders::Loader,
    pub install_performance_mods: bool,

    // Java & memory
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    /// Empty = pick automatically (Mojang runtime matching the version).
    pub java_path: String,
    pub jvm_args: String,
    /// Add tuned G1GC flags before the user's own JVM args.
    pub optimized_jvm_args: bool,

    // Game window
    pub game_width: u32,
    pub game_height: u32,
    pub fullscreen: bool,

    // Launcher behaviour
    /// `keep`, `minimize`, `hide` or `close`.
    pub on_launch: String,
    pub reopen_on_exit: bool,
    pub open_console_on_launch: bool,
    pub max_concurrent_downloads: u32,

    // Linux / advanced
    pub gamemode: bool,
    pub mangohud: bool,
    pub discrete_gpu: bool,
    /// One `KEY=value` per line.
    pub env_vars: String,
    /// Prepended to the java command, e.g. `prime-run`.
    pub wrapper_command: String,
    pub pre_launch_command: String,
    pub post_exit_command: String,

    /// Applied when a Rust instance starts.
    pub rust_engine: crate::engine::EngineSettings,
    /// JSON of the home screen stance; empty for the default.
    pub model_pose: String,

    /// 1 once the new brand's accent and backdrop have been applied.
    #[serde(default)]
    pub brand_version: u32,
    pub onboarded: bool,
    pub last_instance: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            palette: "patina".into(),
            accent: "#3fb8a0".into(),
            animations: true,
            blur: true,
            ui_scale: 1.0,
            font: "inter".into(),
            radius: "normal".into(),
            compact: false,
            hero_wallpaper: String::new(),
            hero_scene: true,
            hero_dim: 25,
            app_wallpaper: String::new(),
            app_wallpaper_dim: 75,
            model_position: "center".into(),
            follow_cursor: true,
            show_nametag: true,
            show_snapshots: false,
            show_old_versions: false,
            default_loader: crate::loaders::Loader::Fabric,
            install_performance_mods: true,
            memory_min_mb: 1024,
            memory_max_mb: default_max_memory(),
            java_path: String::new(),
            jvm_args: String::new(),
            optimized_jvm_args: true,
            game_width: 1280,
            game_height: 720,
            fullscreen: false,
            on_launch: "hide".into(),
            reopen_on_exit: true,
            open_console_on_launch: false,
            max_concurrent_downloads: 16,
            gamemode: false,
            mangohud: false,
            discrete_gpu: false,
            env_vars: String::new(),
            wrapper_command: String::new(),
            pre_launch_command: String::new(),
            post_exit_command: String::new(),
            rust_engine: Default::default(),
            model_pose: String::new(),
            brand_version: 1,
            onboarded: false,
            last_instance: None,
        }
    }
}

pub fn total_memory_mb() -> u64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    sys.total_memory() / 1024 / 1024
}

/// A quarter of RAM, clamped to 2–8 GB, rounded to 512 MB.
fn default_max_memory() -> u32 {
    let quarter = (total_memory_mb() / 4).clamp(2048, 8192) as u32;
    quarter / 512 * 512
}

impl Settings {
    pub fn load(ctx: &Ctx) -> Self {
        let mut settings: Self = std::fs::read(ctx.paths.settings_file()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        if settings.brand_version < 1 {
            settings.accent = Self::default().accent;
            settings.hero_scene = true;
            settings.brand_version = 1;
        }
        settings.sanitized()
    }

    pub fn save(&self, ctx: &Ctx) -> Result<()> {
        crate::paths::write_atomic(&ctx.paths.settings_file(), &serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    /// Clamp values the UI could send out of range.
    pub fn sanitized(mut self) -> Self {
        self.ui_scale = if self.ui_scale.is_finite() { self.ui_scale.clamp(0.8, 1.25) } else { 1.0 };
        self.memory_max_mb = self.memory_max_mb.clamp(512, 65536);
        self.memory_min_mb = self.memory_min_mb.clamp(256, self.memory_max_mb);
        self.max_concurrent_downloads = self.max_concurrent_downloads.clamp(1, 64);
        self.game_width = self.game_width.clamp(320, 15360);
        self.game_height = self.game_height.clamp(240, 8640);
        if !["keep", "minimize", "hide", "close"].contains(&self.on_launch.as_str()) {
            self.on_launch = "hide".into();
        }
        self.rust_engine = self.rust_engine.sanitized();
        self.hero_dim = self.hero_dim.min(90);
        self.app_wallpaper_dim = self.app_wallpaper_dim.min(95);
        if !["dark", "light", "system"].contains(&self.theme.as_str()) {
            self.theme = "dark".into();
        }
        if !["patina", "graphite", "ocean", "forest", "ember", "iris"].contains(&self.palette.as_str()) {
            self.palette = "patina".into();
        }
        if self.accent.len() != 7 || !self.accent.starts_with('#') || !self.accent[1..].bytes().all(|b| b.is_ascii_hexdigit()) {
            self.accent = "#3fb8a0".into();
        }
        if !["inter", "outfit", "system", "mono"].contains(&self.font.as_str()) {
            self.font = "inter".into();
        }
        if !["sharp", "normal", "round"].contains(&self.radius.as_str()) {
            self.radius = "normal".into();
        }
        if !["left", "center", "right"].contains(&self.model_position.as_str()) {
            self.model_position = "center".into();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn older_settings_keep_defaults_and_customization() {
        let s: Settings = serde_json::from_str(r##"{"theme":"light","accent":"#abcdef","onboarded":true}"##).unwrap();
        assert_eq!(s.palette, "patina");
        assert_eq!(s.accent, "#abcdef");
        assert!(s.onboarded);
    }

    #[test]
    fn repairs_invalid_values_and_memory_relationship() {
        let s = Settings {
            ui_scale: f32::NAN, memory_min_mb: 8192, memory_max_mb: 1024,
            accent: "red; color: transparent".into(), palette: "unknown".into(),
            font: "bad".into(), radius: "bad".into(), model_position: "bad".into(),
            hero_dim: 500, ..Settings::default()
        }.sanitized();
        assert_eq!(s.ui_scale, 1.0);
        assert_eq!(s.memory_min_mb, 1024);
        assert_eq!(s.accent, "#3fb8a0");
        assert_eq!(s.palette, "patina");
        assert_eq!(s.font, "inter");
        assert_eq!(s.radius, "normal");
        assert_eq!(s.model_position, "center");
        assert_eq!(s.hero_dim, 90);
    }
}
