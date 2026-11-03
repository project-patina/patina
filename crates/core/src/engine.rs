//! The Rust engine: a Minecraft client written in Rust, built from `engine/`
//! (MIT, see `engine/LICENSE`) and started with the session on standard input.
//!
//! The engine holds no game files. Its resources are prepared locally from the
//! game the launcher already downloaded, so a Rust instance installs like a
//! vanilla one and then runs `patina-prepare` once.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::download::{download_all, DlItem};
use crate::install::Prepared;
use crate::instance::Instance;
use crate::launch::{self, LaunchSpec, Player};
use crate::paths::write_atomic;
use crate::progress::Reporter;
use crate::settings::Settings;
use crate::{bail, Ctx, Error, Result};

/// The only game version the engine runs.
pub const MC_VERSION: &str = "26.3";
const GRAPHICS_FILE: &str = "patina-graphics.txt";
const READY_MARKER: &str = "amethyst-ready";

/// Engine options the launcher applies at every start. `None` and `game` leave
/// the choice made in the game's own menus alone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngineSettings {
    /// Distant terrain in chunks, 0 = off.
    pub lod_distance: Option<u32>,
    /// Render resolution in %, 50–200.
    pub resolution_scale: Option<u32>,
    /// `game`, `off`, `fxaa` or `taa`.
    pub antialiasing: String,
    /// `game`, `auto`, `vulkan` or `opengl`.
    pub graphics_api: String,
    pub hdr: Option<bool>,
    pub battery_mode: Option<bool>,
    pub fps_overlay: Option<bool>,
    /// Present through XWayland. Measured with NVIDIA rendering and Intel driving the
    /// screens: 500 FPS against 310 fullscreen and 175 in a window on plain Wayland.
    pub xwayland: bool,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            lod_distance: None,
            resolution_scale: None,
            antialiasing: "game".into(),
            graphics_api: "game".into(),
            hdr: None,
            battery_mode: None,
            fps_overlay: None,
            xwayland: true,
        }
    }
}

impl EngineSettings {
    pub fn sanitized(mut self) -> Self {
        self.lod_distance = self.lod_distance.map(|d| d.min(1024));
        self.resolution_scale = self.resolution_scale.map(|s| s.clamp(50, 200));
        if !["game", "off", "fxaa", "taa"].contains(&self.antialiasing.as_str()) {
            self.antialiasing = "game".into();
        }
        if !["game", "auto", "vulkan", "opengl"].contains(&self.graphics_api.as_str()) {
            self.graphics_api = "game".into();
        }
        self
    }

    /// The `patina-graphics.txt` lines these settings decide.
    fn graphics_lines(&self) -> Vec<(&'static str, String)> {
        let mut lines = Vec::new();
        if let Some(d) = self.lod_distance {
            lines.push(("lodDistance", d.to_string()));
        }
        if let Some(s) = self.resolution_scale {
            lines.push(("resolutionScale", s.to_string()));
        }
        if self.antialiasing != "game" {
            lines.push(("fxaa", (self.antialiasing == "fxaa").to_string()));
            lines.push(("taa", (self.antialiasing == "taa").to_string()));
        }
        if self.graphics_api != "game" {
            lines.push(("graphicsApi", self.graphics_api.clone()));
        }
        if let Some(h) = self.hdr {
            lines.push(("hdr", h.to_string()));
        }
        if let Some(b) = self.battery_mode {
            lines.push(("batteryMode", b.to_string()));
        }
        if let Some(f) = self.fps_overlay {
            lines.push(("overlay", if f { "fps" } else { "off" }.to_string()));
        }
        lines
    }
}

/// Where the built engine lives: `AMETHYST_ENGINE_DIR`, else an `engine` folder
/// beside the launcher (the Windows package), else `engine` in the data folder.
pub fn root(ctx: &Ctx) -> PathBuf {
    std::env::var_os("AMETHYST_ENGINE_DIR").map(PathBuf::from).or_else(beside_launcher).unwrap_or_else(|| ctx.paths.root.join("engine"))
}

fn beside_launcher() -> Option<PathBuf> {
    let folder = std::env::current_exe().ok()?.parent()?.join("engine");
    client(&folder).is_file().then_some(folder)
}

fn client(root: &Path) -> PathBuf {
    root.join(format!("patina-client{}", std::env::consts::EXE_SUFFIX))
}

fn resources(root: &Path) -> PathBuf {
    root.join(".local/patina-client").join(MC_VERSION)
}

/// The engine runs only for an account that owns the game. Only a clear "no"
/// blocks: without a network the player keeps what they already have.
pub async fn ensure_owner(ctx: &Ctx, token: &str) -> Result<()> {
    let Ok(response) = ctx.http.get("https://api.minecraftservices.com/entitlements/mcstore").bearer_auth(token).send().await else {
        return Ok(());
    };
    if !response.status().is_success() {
        return Ok(());
    }
    let Ok(body) = response.json::<serde_json::Value>().await else { return Ok(()) };
    if owns_game(&body) {
        return Ok(());
    }
    bail!("To konto nie posiada Minecraft: Java Edition.", "This account does not own Minecraft: Java Edition.")
}

fn owns_game(entitlements: &serde_json::Value) -> bool {
    entitlements["items"].as_array().is_some_and(|items| items.iter().any(|i| matches!(i["name"].as_str(), Some("game_minecraft" | "product_minecraft"))))
}

/// Build the engine's resources from the game `prep` downloaded. Runs once.
pub async fn prepare(ctx: &Ctx, prep: &Prepared, settings: &Settings, reporter: &Reporter, cancel: &CancellationToken) -> Result<()> {
    let root = root(ctx);
    if !client(&root).is_file() {
        bail!("Silnik Rust nie jest zainstalowany ({}). Uruchom scripts/install-engine.sh.", "The Rust engine is not installed ({}). Run scripts/install-engine.sh.", root.display());
    }
    // The marker names the lock it was prepared for: a newer engine prepares again.
    let stamp = lock_stamp(&root);
    if is_ready(&root, &stamp) {
        return Ok(());
    }
    let Some(server) = prep.version.downloads.as_ref().and_then(|d| d.server.clone()) else {
        bail!("Wersja {} nie ma pliku serwera", "Version {} has no server file", prep.vanilla_id);
    };
    let server_jar = ctx.paths.version_dir(&prep.vanilla_id).join("server.jar");
    let item = DlItem::new(&server.url, &server_jar).sha1(server.sha1.clone()).size(server.size);
    download_all(&ctx.http, vec![item], settings.max_concurrent_downloads as usize, reporter, "engine", crate::lang::tr("Pobieranie plików serwera", "Downloading server files"), cancel).await?;

    let java_home = prep.java.parent().and_then(Path::parent).ok_or_else(|| Error::msg(crate::lang::tr("Nieznany folder Javy", "Unknown Java folder")))?;
    let minecraft = ctx.paths.root.to_string_lossy().into_owned();
    let root_arg = root.to_string_lossy().into_owned();
    let server_arg = server_jar.to_string_lossy().into_owned();
    reporter.report("engine", 0, 2, crate::lang::tr("Przygotowywanie zasobów silnika…", "Preparing engine resources…"));
    run_prepare(&root, java_home, &["--prepare", "--server-jar", &server_arg, "--minecraft-dir", &minecraft, "--root", &root_arg], 0, reporter, cancel).await?;
    reporter.report("engine", 1, 2, crate::lang::tr("Kopiowanie dźwięków…", "Copying sounds…"));
    run_prepare(&root, java_home, &["sounds", "--minecraft-dir", &minecraft, "--root", &root_arg], 1, reporter, cancel).await?;
    write_atomic(&resources(&root).join(READY_MARKER), stamp.as_bytes())?;
    Ok(())
}

/// SHA-256 of `research-lock.json`, which pins everything the resources are made from.
fn lock_stamp(root: &Path) -> String {
    use sha2::{Digest, Sha256};
    std::fs::read(root.join("research-lock.json")).map_or_else(|_| String::new(), |lock| hex::encode(Sha256::digest(lock)))
}

fn is_ready(root: &Path, stamp: &str) -> bool {
    std::fs::read_to_string(resources(root).join(READY_MARKER)).is_ok_and(|ready| ready == stamp)
}

async fn run_prepare(root: &Path, java_home: &Path, args: &[&str], step: u64, reporter: &Reporter, cancel: &CancellationToken) -> Result<()> {
    let mut command = tokio::process::Command::new(root.join(format!("patina-prepare{}", std::env::consts::EXE_SUFFIX)));
    command.args(args).env("JAVA_HOME", java_home).current_dir(root).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true);
    launch::hide_console(&mut command);
    let mut child = command.spawn()?;
    let (tx, mut lines) = tokio::sync::mpsc::unbounded_channel();
    if let Some(out) = child.stdout.take() {
        pump(out, tx.clone());
    }
    if let Some(err) = child.stderr.take() {
        pump(err, tx);
    }
    let mut last = String::new();
    loop {
        tokio::select! {
            line = lines.recv() => match line {
                Some(line) => {
                    reporter.report("engine", step, 2, line.as_str());
                    last = line;
                }
                None => break,
            },
            _ = cancel.cancelled() => return Err(Error::Cancelled),
        }
    }
    if !child.wait().await?.success() {
        bail!("Przygotowanie silnika nie powiodło się: {last}", "Engine preparation failed: {last}");
    }
    Ok(())
}

fn pump(stream: impl AsyncRead + Unpin + Send + 'static, tx: UnboundedSender<String>) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx.send(line);
        }
    });
}

/// The command line for `patina-client`, with the session on standard input.
pub fn build(ctx: &Ctx, inst: &Instance, player: &Player, s: &Settings) -> Result<LaunchSpec> {
    let o = &inst.overrides;
    let root = root(ctx);
    let game_dir = ctx.paths.game_dir(&inst.id);
    std::fs::create_dir_all(&game_dir)?;
    apply_graphics(&game_dir, &s.rust_engine)?;

    let game_dir_arg = game_dir.to_string_lossy().into_owned();
    let mut args = vec!["--resources".to_string(), resources(&root).to_string_lossy().into_owned()];
    args.extend(["--game-dir".to_string(), game_dir_arg.clone(), "--home".to_string(), game_dir_arg, "--session-stdin".to_string()]);
    if o.fullscreen.unwrap_or(s.fullscreen) {
        args.push("--fullscreen".into());
    } else {
        args.extend(["--width".into(), o.game_width.unwrap_or(s.game_width).to_string(), "--height".into(), o.game_height.unwrap_or(s.game_height).to_string()]);
    }
    if let Some(server) = o.join_server.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        args.extend(["--quickPlayMultiplayer".into(), server.into()]);
    }

    let mut program = launch::wrappers(o, s);
    program.push(client(&root).to_string_lossy().into_owned());

    let mut env: Vec<(String, String)> = vec![("PIPEWIRE_PROPS".into(), format!("{{ application.name = \"{}\" media.role = \"Game\" }}", crate::LAUNCHER_NAME))];
    env.extend(launch::base_env(o, s));

    let env_remove = if uses_xwayland(&s.rust_engine, std::env::var_os("DISPLAY").as_deref()) { vec!["WAYLAND_DISPLAY".to_string()] } else { vec![] };
    let session = session_json(player);
    Ok(LaunchSpec { program, args, env, cwd: game_dir, secrets: vec![player.access_token.clone()], env_remove, stdin: Some(session) })
}

/// With no X server to fall back on the game stays on Wayland.
fn uses_xwayland(engine: &EngineSettings, display: Option<&std::ffi::OsStr>) -> bool {
    engine.xwayland && display.is_some_and(|d| !d.is_empty())
}

fn session_json(player: &Player) -> String {
    serde_json::json!({
        "name": player.name,
        "uuid": player.uuid,
        "access_token": player.access_token,
        "user_type": "msa",
        "xuid": player.xuid,
    })
    .to_string()
}

fn apply_graphics(game_dir: &Path, engine: &EngineSettings) -> Result<()> {
    let changes = engine.graphics_lines();
    if changes.is_empty() {
        return Ok(());
    }
    let path = game_dir.join(GRAPHICS_FILE);
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    write_atomic(&path, merge_lines(&current, &changes).as_bytes())?;
    Ok(())
}

/// `current` with each `key:value` of `changes` replaced or added; every other line stays.
fn merge_lines(current: &str, changes: &[(&str, String)]) -> String {
    let mut out = String::with_capacity(current.len());
    for line in current.lines() {
        let key = line.split_once(':').map(|(k, _)| k);
        if !changes.iter().any(|(k, _)| Some(*k) == key) {
            out.push_str(line);
            out.push('\n');
        }
    }
    for (key, value) in changes {
        out.push_str(&format!("{key}:{value}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_changed_lock_prepares_the_resources_again() {
        let root = std::env::temp_dir().join(format!("patina-ready-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(resources(&root)).unwrap();
        std::fs::write(root.join("research-lock.json"), "one").unwrap();
        let stamp = lock_stamp(&root);
        assert!(!is_ready(&root, &stamp));
        // The marker written by earlier versions was empty.
        std::fs::write(resources(&root).join(READY_MARKER), "").unwrap();
        assert!(!is_ready(&root, &stamp));
        std::fs::write(resources(&root).join(READY_MARKER), &stamp).unwrap();
        assert!(is_ready(&root, &stamp));
        std::fs::write(root.join("research-lock.json"), "two").unwrap();
        assert!(!is_ready(&root, &lock_stamp(&root)));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn defaults_leave_the_games_choices_alone() {
        let e = EngineSettings::default();
        assert!(e.graphics_lines().is_empty());
    }

    #[test]
    fn merging_keeps_lines_the_launcher_does_not_decide() {
        let merged = merge_lines("preset:ultra\nlodDistance:0\nhdr:false\n", &[("lodDistance", "256".into())]);
        assert_eq!(merged, "preset:ultra\nhdr:false\nlodDistance:256\n");
    }

    #[test]
    fn antialiasing_sets_both_switches() {
        let e = EngineSettings { antialiasing: "taa".into(), ..Default::default() };
        assert_eq!(e.graphics_lines(), vec![("fxaa", "false".to_string()), ("taa", "true".to_string())]);
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let e = EngineSettings { lod_distance: Some(9000), resolution_scale: Some(10), antialiasing: "x".into(), ..Default::default() }.sanitized();
        assert_eq!(e.lod_distance, Some(1024));
        assert_eq!(e.resolution_scale, Some(50));
        assert_eq!(e.antialiasing, "game");
    }

    #[test]
    fn ownership_needs_a_game_entitlement() {
        assert!(owns_game(&serde_json::json!({ "items": [{ "name": "product_minecraft" }] })));
        assert!(!owns_game(&serde_json::json!({ "items": [{ "name": "game_dungeons" }] })));
        assert!(!owns_game(&serde_json::json!({ "items": [] })));
    }

    #[test]
    fn xwayland_needs_an_x_server() {
        let e = EngineSettings::default();
        assert!(uses_xwayland(&e, Some(std::ffi::OsStr::new(":0"))));
        assert!(!uses_xwayland(&e, Some(std::ffi::OsStr::new(""))));
        assert!(!uses_xwayland(&e, None));
        assert!(!uses_xwayland(&EngineSettings { xwayland: false, ..e }, Some(std::ffi::OsStr::new(":0"))));
    }

    #[test]
    fn the_session_matches_the_launch_api() {
        let p = Player { name: "Alex".into(), uuid: "abc".into(), access_token: "t".into(), xuid: "1".into(), user_type: "msa".into() };
        let v: serde_json::Value = serde_json::from_str(&session_json(&p)).unwrap();
        assert_eq!(v["access_token"], "t");
        assert_eq!(v["user_type"], "msa");
    }
}
