mod commands;
mod content_cmds;
mod game;
mod login;
mod media;
mod presence;
mod updates;

// Check Tauri's actual mode: its CLI may enable the dependency feature directly.
#[cfg(not(debug_assertions))]
const _: () = assert!(!tauri::is_dev(), "A standalone Patina release must embed its frontend. Run `npm run build:desktop` or enable `--features custom-protocol` after building the frontend.");

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
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
    /// An update is being installed, or has been and waits for a restart.
    pub updating: AtomicBool,
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
    let settings = Settings::load(&ctx);
    amethyst_core::lang::set(&settings.language);
    let state = AppState {
        settings: RwLock::new(settings),
        accounts: tokio::sync::Mutex::new(Accounts::load(&ctx)),
        ctx,
        installs: Mutex::default(),
        running: Mutex::default(),
        logs: Mutex::default(),
        login_cancel: Mutex::default(),
        updating: AtomicBool::new(false),
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
        .manage(presence::PresenceState::new())
        // Here and not earlier: a second instance has exited by now and cannot touch a running update.
        .setup(|app| {
            amethyst_core::update::clean_stale(&app.state::<AppState>().ctx);
            presence::refresh(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_announcements,
            media::gear_textures,
            commands::get_settings,
            commands::save_settings,
            commands::set_language,
            updates::update_check,
            updates::update_install,
            updates::update_restart,
            commands::list_versions,
            commands::loader_versions,
            commands::list_instances,
            commands::get_instance,
            commands::create_instance,
            commands::update_instance,
            commands::delete_instance,
            content_cmds::modrinth_search,
            content_cmds::servers_list,
            content_cmds::server_ping,
            content_cmds::modrinth_project,
            content_cmds::modrinth_versions,
            content_cmds::install_content,
            content_cmds::list_content,
            content_cmds::set_content_enabled,
            content_cmds::delete_content,
            content_cmds::import_content,
            content_cmds::check_content_updates,
            content_cmds::official_mods,
            content_cmds::install_official_mod,
            content_cmds::remove_official_mod,
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
            presence::presence_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Patina");
}

#[cfg(all(test, feature = "custom-protocol"))]
mod production_tests {
    #[test]
    fn standalone_context_embeds_html_scripts_styles_and_card_images() {
        assert!(!tauri::is_dev(), "The standalone build must not use devUrl");
        let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        let assets = context.assets();
        let index = assets.get(&"/index.html".into()).expect("embedded index.html");
        let html = String::from_utf8_lossy(&index);
        let mut scripts = 0;
        let mut styles = 0;
        for reference in html.split('"').filter(|value| value.starts_with("/assets/")) {
            assert!(assets.get(&reference.into()).is_some(), "missing embedded asset: {reference}");
            if reference.ends_with(".js") { scripts += 1; }
            if reference.ends_with(".css") { styles += 1; }
        }
        assert!(scripts > 0 && styles > 0);
        let image = "/announcements/first-version.png";
        assert!(assets.get(&image.into()).is_some(), "missing card image: {image}");
    }
}
