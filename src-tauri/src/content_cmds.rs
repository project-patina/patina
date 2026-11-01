//! Mods / resource packs / shaders: browsing Modrinth and managing files.

use std::path::PathBuf;

use amethyst_core::content::{self, ContentItem, ContentKind};
use amethyst_core::instance::Instance;
use amethyst_core::modrinth::{self, Installed, Project, SearchQuery, SearchResult, Update, Version};
use amethyst_core::Result;
use tauri::{AppHandle, State};
use tokio_util::sync::CancellationToken;

use crate::AppState;

#[tauri::command]
pub async fn modrinth_search(state: State<'_, AppState>, query: SearchQuery) -> Result<SearchResult> {
    modrinth::search(&state.ctx, &query).await
}

#[tauri::command]
pub async fn modrinth_project(state: State<'_, AppState>, id: String) -> Result<Project> {
    modrinth::project(&state.ctx, &id).await
}

/// Versions of a project. With `instance`, they are filtered to its loader;
/// `all_versions` drops the game-version filter (for downgrades / testing).
#[tauri::command]
pub async fn modrinth_versions(state: State<'_, AppState>, id: String, kind: ContentKind, instance: Option<String>, all_versions: bool) -> Result<Vec<Version>> {
    let (loaders, mc) = match instance {
        Some(i) => {
            let inst = Instance::load(&state.ctx, &i)?;
            (modrinth::loader_names(kind, inst.loader), (!all_versions).then_some(inst.mc_version))
        }
        None => (vec![], None),
    };
    modrinth::versions(&state.ctx, &id, &loaders, mc.as_deref()).await
}

#[tauri::command]
pub async fn install_content(
    app: AppHandle,
    state: State<'_, AppState>,
    instance: String,
    kind: ContentKind,
    project: String,
    version: Option<String>,
) -> Result<Vec<Installed>> {
    let inst = Instance::load(&state.ctx, &instance)?;
    let rep = crate::game::reporter(&app, &format!("content:{instance}"));
    modrinth::install(&state.ctx, &inst, kind, &project, version.as_deref(), &rep, &CancellationToken::new()).await
}

#[tauri::command]
pub async fn list_content(state: State<'_, AppState>, id: String, kind: ContentKind, identify: bool) -> Result<Vec<ContentItem>> {
    Instance::load(&state.ctx, &id)?;
    if identify {
        // Best effort: offline still lists the files.
        if let Err(e) = modrinth::identify(&state.ctx, &id, kind).await {
            log::warn!("identify {id}: {e}");
        }
    }
    Ok(content::list(&state.ctx, &id, kind))
}

#[tauri::command]
pub fn set_content_enabled(state: State<'_, AppState>, id: String, kind: ContentKind, file: String, enabled: bool) -> Result<()> {
    Instance::load(&state.ctx, &id)?;
    content::set_enabled(&state.ctx, &id, kind, &file, enabled)
}

#[tauri::command]
pub async fn delete_content(state: State<'_, AppState>, id: String, kind: ContentKind, file: String) -> Result<()> {
    Instance::load(&state.ctx, &id)?;
    content::delete(&state.ctx, &id, kind, &file).await
}

#[tauri::command]
pub async fn import_content(state: State<'_, AppState>, id: String, kind: ContentKind, paths: Vec<PathBuf>) -> Result<Vec<String>> {
    Instance::load(&state.ctx, &id)?;
    content::import(&state.ctx, &id, kind, &paths).await
}

#[tauri::command]
pub async fn check_content_updates(state: State<'_, AppState>, id: String, kind: ContentKind) -> Result<Vec<Update>> {
    let inst = Instance::load(&state.ctx, &id)?;
    modrinth::check_updates(&state.ctx, &inst, kind).await
}
