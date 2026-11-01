mod commands;
mod content_cmds;
mod game;
mod login;
mod media;

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, RwLock};

use amethyst_core::accounts::Accounts;
use amethyst_core::paths::Paths;
use amethyst_core::settings::Settings;
use amethyst_core::Ctx;
use serde::Serialize;
use tauri::Manager;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub id: String,
    pub line: String,
    pub error: bool,
}

pub struct Running {
    pub kill: CancellationToken,
    pub started: i64,
}

pub struct AppState {
    pub ctx: Ctx,
    pub settings: RwLock<Settings>,
    pub accounts: tokio::sync::Mutex<Accounts>,
    /// Instances currently downloading/preparing → cancel token.
    pub installs: Mutex<HashMap<String, CancellationToken>>,
    pub running: Mutex<HashMap<String, Running>>,
    /// Last lines of each instance's game output, for the console view.
    pub logs: Mutex<HashMap<String, VecDeque<LogLine>>>,
    pub login_cancel: Mutex<Option<CancellationToken>>,
}

pub const LOG_KEEP: usize = 3000;

impl AppState {
    pub fn push_log(&self, l: LogLine) {
        let mut logs = self.logs.lock().unwrap();
        let q = logs.entry(l.id.clone()).or_default();
        if q.len() >= LOG_KEEP {
            q.pop_front();
        }
        q.push_back(l);
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }
}

/// True when every DRM card is NVIDIA (vendor 0x10de).
#[cfg(target_os = "linux")]
fn nvidia_only() -> bool {
    let vendors: Vec<String> = std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("card") && !e.file_name().to_string_lossy().contains('-'))
        .filter_map(|e| std::fs::read_to_string(e.path().join("device/vendor")).ok())
        .map(|v| v.trim().to_lowercase())
        .collect();
    !vendors.is_empty() && vendors.iter().all(|v| v == "0x10de")
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,tao=warn,wry=warn")).init();

    // WebKitGTK's DMA-BUF renderer can show a blank window when NVIDIA is the
    // only GPU. Disabling it costs a lot (measured 144 → 2 FPS on Intel+NVIDIA),
    // so only do it when no other GPU can draw the window.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() && nvidia_only() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    let root = std::env::var("AMETHYST_ROOT").map(Into::into).unwrap_or_else(|_| Paths::default_root());
    let ctx = Ctx::new(Paths::new(root)).expect("cannot create launcher data directory");
    let state = AppState {
        settings: RwLock::new(Settings::load(&ctx)),
        accounts: tokio::sync::Mutex::new(Accounts::load(&ctx)),
        ctx,
        installs: Mutex::default(),
        running: Mutex::default(),
        logs: Mutex::default(),
        login_cancel: Mutex::default(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("media", |ctx, req| media::serve(&ctx.app_handle().state::<AppState>(), req.uri().path()))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_news,
            media::gear_textures,
            media::backdrop_frames,
            commands::get_settings,
            commands::save_settings,
            commands::list_versions,
            commands::loader_versions,
            commands::list_instances,
            commands::get_instance,
            commands::create_instance,
            commands::update_instance,
            commands::delete_instance,
            content_cmds::modrinth_search,
            content_cmds::modrinth_project,
            content_cmds::modrinth_versions,
            content_cmds::install_content,
            content_cmds::list_content,
            content_cmds::set_content_enabled,
            content_cmds::delete_content,
            content_cmds::import_content,
            content_cmds::check_content_updates,
            media::import_image,
            media::list_images,
            media::delete_image,
            media::get_profile,
            media::set_cape,
            media::upload_skin,
            media::set_skin_variant,
            commands::install_performance_mods,
            commands::open_folder,
            commands::detect_java,
            commands::list_accounts,
            commands::set_active_account,
            commands::remove_account,
            commands::skin_data_url,
            login::login_microsoft,
            login::login_device_start,
            login::login_device_finish,
            login::cancel_login,
            game::launch,
            game::stop,
            game::cancel_install,
            game::running,
            game::game_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Patina");
}
