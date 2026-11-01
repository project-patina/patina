use amethyst_core::accounts::AccountView;
use amethyst_core::instance::{Instance, NewInstance};
use amethyst_core::java::JavaInstall;
use amethyst_core::loaders::{self, Loader, LoaderVersion};
use amethyst_core::settings::{self, Settings};
use amethyst_core::{accounts, bail, engine, mojang, Error, Result};
use serde::Serialize;
use tauri::State;
use tokio_util::sync::CancellationToken;

use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    data_dir: String,
    total_memory_mb: u64,
    os: &'static str,
    arch: &'static str,
    /// Debug builds only: page to open on start (`AMETHYST_START_PAGE`), for testing.
    start_page: Option<String>,
}

#[tauri::command]
pub fn app_info(state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: amethyst_core::LAUNCHER_VERSION,
        data_dir: state.ctx.paths.root.to_string_lossy().into(),
        total_memory_mb: settings::total_memory_mb(),
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        start_page: if cfg!(debug_assertions) { std::env::var("AMETHYST_START_PAGE").ok() } else { None },
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Settings {
    state.settings()
}

#[tauri::command]
pub fn list_news() -> Result<Vec<amethyst_core::news::ReleaseNote>> {
    amethyst_core::news::list()
}

#[tauri::command]
pub fn save_settings(state: State<'_, AppState>, settings: Settings) -> Result<Settings> {
    let s = settings.sanitized();
    let mut current = state.settings.write().unwrap();
    s.save(&state.ctx)?;
    *current = s.clone();
    Ok(s)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameVersion {
    id: String,
    kind: String,
    release_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionList {
    latest_release: String,
    latest_snapshot: String,
    versions: Vec<GameVersion>,
}

#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionList> {
    let m = mojang::manifest(&state.ctx).await?;
    Ok(VersionList {
        latest_release: m.latest.release,
        latest_snapshot: m.latest.snapshot,
        versions: m.versions.into_iter().map(|v| GameVersion { id: v.id, kind: v.kind, release_time: v.release_time }).collect(),
    })
}

#[tauri::command]
pub async fn loader_versions(state: State<'_, AppState>, loader: Loader, mc: String) -> Result<Vec<LoaderVersion>> {
    loaders::versions(&state.ctx, loader, &mc).await
}

#[tauri::command]
pub fn list_instances(state: State<'_, AppState>) -> Vec<Instance> {
    Instance::list(&state.ctx)
}

#[tauri::command]
pub fn get_instance(state: State<'_, AppState>, id: String) -> Result<Instance> {
    Instance::load(&state.ctx, &id)
}

#[tauri::command]
pub fn create_instance(state: State<'_, AppState>, req: NewInstance) -> Result<Instance> {
    let inst = Instance::create(&state.ctx, req)?;
    let mut s = state.settings.write().unwrap();
    s.last_instance = Some(inst.id.clone());
    let _ = s.save(&state.ctx);
    Ok(inst)
}

/// Saves user-editable fields; bookkeeping (id, play time, …) stays as on disk.
#[tauri::command]
pub fn update_instance(state: State<'_, AppState>, instance: Instance) -> Result<Instance> {
    let installs = state.installs.lock().unwrap();
    let running = state.running.lock().unwrap();
    let mut cur = Instance::load(&state.ctx, &instance.id)?;
    let name = instance.name.trim();
    if name.is_empty() {
        bail!("Nazwa nie może być pusta");
    }
    let mc_version = if instance.loader == Loader::Rust { engine::MC_VERSION.into() } else { instance.mc_version };
    amethyst_core::instance::validate_version_id(&mc_version)?;
    let loader_version = if instance.loader.is_plain() { None } else { instance.loader_version.filter(|v| !v.is_empty()) };
    let changed_game = cur.mc_version != mc_version || cur.loader != instance.loader || cur.loader_version != loader_version;
    if changed_game && (running.contains_key(&cur.id) || installs.contains_key(&cur.id)) {
        bail!("Zatrzymaj grę lub przygotowanie przed zmianą wersji i loadera");
    }
    cur.name = name.chars().take(48).collect();
    cur.icon = instance.icon;
    cur.banner = instance.banner;
    cur.overrides = instance.overrides;
    cur.loader = instance.loader;
    cur.mc_version = mc_version;
    cur.loader_version = loader_version;
    if changed_game {
        cur.installed = false;
    }
    cur.save(&state.ctx)?;
    Ok(cur)
}

#[tauri::command]
pub fn delete_instance(state: State<'_, AppState>, id: String) -> Result<()> {
    let installs = state.installs.lock().unwrap();
    let running = state.running.lock().unwrap();
    if running.contains_key(&id) || installs.contains_key(&id) {
        bail!("Najpierw zamknij grę w tej instancji");
    }
    Instance::delete(&state.ctx, &id)
}

#[tauri::command]
pub async fn install_performance_mods(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<Vec<String>> {
    let inst = Instance::load(&state.ctx, &id)?;
    let reporter = crate::game::reporter(&app, &id);
    amethyst_core::modrinth::install_performance_mods(&state.ctx, &inst, &reporter, &CancellationToken::new()).await
}

/// Open the data dir, or an instance's game dir (optionally a sub-folder).
#[tauri::command]
pub fn open_folder(state: State<'_, AppState>, id: Option<String>, sub: Option<String>) -> Result<()> {
    let mut path = match &id {
        Some(id) => {
            Instance::load(&state.ctx, id)?;
            state.ctx.paths.game_dir(id)
        }
        None => state.ctx.paths.root.clone(),
    };
    if let Some(sub) = sub.filter(|s| !s.is_empty()) {
        if !sub.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            bail!("Niepoprawny folder");
        }
        path = path.join(sub);
    }
    std::fs::create_dir_all(&path)?;
    tauri_plugin_opener::open_path(&path, None::<&str>).map_err(|e| Error::msg(e.to_string()))
}

#[tauri::command]
pub async fn detect_java(state: State<'_, AppState>) -> Result<Vec<JavaInstall>> {
    Ok(amethyst_core::java::detect(&state.ctx).await)
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<AccountView>> {
    Ok(state.accounts.lock().await.views())
}

#[tauri::command]
pub async fn set_active_account(state: State<'_, AppState>, uuid: String) -> Result<Vec<AccountView>> {
    let mut accs = state.accounts.lock().await;
    if !accs.list.iter().any(|a| a.uuid == uuid) {
        bail!("Nie ma takiego konta");
    }
    accs.active = Some(uuid);
    accs.save(&state.ctx)?;
    Ok(accs.views())
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, uuid: String) -> Result<Vec<AccountView>> {
    let mut accs = state.accounts.lock().await;
    accs.remove(&uuid);
    accs.save(&state.ctx)?;
    Ok(accs.views())
}

#[tauri::command]
pub async fn skin_data_url(state: State<'_, AppState>, url: String) -> Result<String> {
    accounts::skin_data_url(&state.ctx, &url).await
}
