//! Wallpapers, the skin library and the account's skin/cape.

use std::path::PathBuf;

use amethyst_core::accounts::{self, AccountView};
use amethyst_core::game_textures::{self, GearTextures};
use amethyst_core::profile::{self, Profile};
use amethyst_core::wallpapers::{self, Image, Store};
use amethyst_core::{bail, Result};
use tauri::http::{Response, StatusCode};
use tauri::State;

use crate::AppState;

fn store(kind: &str) -> Result<Store> {
    match kind {
        "wallpaper" => Ok(Store::Wallpapers),
        "skin" => Ok(Store::Skins),
        _ => bail!("Nieznany rodzaj obrazu"),
    }
}

#[tauri::command]
pub fn gear_textures(state: State<'_, AppState>) -> Option<GearTextures> {
    game_textures::load(&state.ctx)
}

/// How many frames of the home screen's film `scripts/render-backdrop.py` left; 0 for none.
#[tauri::command]
pub fn backdrop_frames(state: State<'_, AppState>) -> usize {
    std::fs::read_dir(state.ctx.paths.backdrop()).map_or(0, |dir| dir.flatten().filter(|e| is_frame(&e.file_name().to_string_lossy())).count())
}

fn is_frame(name: &str) -> bool {
    name.strip_suffix(".jpg").is_some_and(|n| n.len() == 5 && n.bytes().all(|b| b.is_ascii_digit()))
}

#[tauri::command]
pub fn import_image(state: State<'_, AppState>, kind: String, path: PathBuf) -> Result<Image> {
    wallpapers::import(&state.ctx, store(&kind)?, &path)
}

#[tauri::command]
pub fn list_images(state: State<'_, AppState>, kind: String) -> Result<Vec<Image>> {
    Ok(wallpapers::list(&state.ctx, store(&kind)?))
}

#[tauri::command]
pub fn delete_image(state: State<'_, AppState>, kind: String, id: String) -> Result<()> {
    wallpapers::delete(&state.ctx, store(&kind)?, &id)
}

/// `media://localhost/<wallpaper|skin>/<id>` → the image bytes.
pub fn serve(state: &AppState, path: &str) -> Response<Vec<u8>> {
    // `convertFileSrc` percent-encodes the whole path, slashes included.
    let path = percent_decode(path);
    let mut parts = path.trim_start_matches('/').splitn(2, '/');
    let (kind, id) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let file = if kind == "backdrop" && is_frame(id) { Ok(state.ctx.paths.backdrop().join(id)) } else { store(kind).and_then(|s| wallpapers::path(&state.ctx, s, id)) };
    match file.ok().and_then(|p| std::fs::read(p).ok()) {
        Some(bytes) => Response::builder()
            .header("Content-Type", wallpapers::mime(id))
            .header("Cache-Control", "max-age=31536000, immutable")
            .header("Access-Control-Allow-Origin", "*")
            .body(bytes)
            .unwrap(),
        None => Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap(),
    }
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ---------------------------------------------------------------------------
// Profile (needs the account's fresh token)

async fn with_token(state: &AppState, uuid: &str) -> Result<String> {
    let mut accs = state.accounts.lock().await;
    let Some(acc) = accs.list.iter().find(|a| a.uuid == uuid).cloned() else { bail!("Nie ma takiego konta") };
    if let Some(fresh) = accounts::ensure_fresh(&state.ctx, &acc).await? {
        let token = fresh.mc_access_token.clone();
        let active = accs.active.clone();
        accs.upsert(fresh);
        accs.active = active;
        accs.save(&state.ctx)?;
        return Ok(token);
    }
    Ok(acc.mc_access_token)
}

/// Store the new skin/cape on the account and hand the UI fresh views.
async fn adopt(state: &AppState, uuid: &str, p: &Profile) -> Result<Vec<AccountView>> {
    let mut accs = state.accounts.lock().await;
    if let Some(a) = accs.list.iter_mut().find(|a| a.uuid == uuid) {
        a.apply_profile(p);
    }
    accs.save(&state.ctx)?;
    Ok(accs.views())
}

#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>, uuid: String) -> Result<Profile> {
    let token = with_token(&state, &uuid).await?;
    let p = profile::get(&state.ctx, &token).await?;
    adopt(&state, &uuid, &p).await?;
    Ok(p)
}

#[tauri::command]
pub async fn set_cape(state: State<'_, AppState>, uuid: String, cape: Option<String>) -> Result<Vec<AccountView>> {
    let token = with_token(&state, &uuid).await?;
    let p = profile::set_cape(&state.ctx, &token, cape.as_deref()).await?;
    adopt(&state, &uuid, &p).await
}

/// Upload a skin from the library (`skin` id) or from a picked file (`path`).
#[tauri::command]
pub async fn upload_skin(state: State<'_, AppState>, uuid: String, skin: Option<String>, path: Option<PathBuf>, slim: bool) -> Result<Vec<AccountView>> {
    let file = match (skin, path) {
        (Some(id), _) => wallpapers::path(&state.ctx, Store::Skins, &id)?,
        (None, Some(p)) => p,
        _ => bail!("Wybierz skina"),
    };
    let png = std::fs::read(file)?;
    let token = with_token(&state, &uuid).await?;
    let p = profile::upload_skin(&state.ctx, &token, png, slim).await?;
    adopt(&state, &uuid, &p).await
}

#[tauri::command]
pub async fn set_skin_variant(state: State<'_, AppState>, uuid: String, slim: bool) -> Result<Vec<AccountView>> {
    let token = with_token(&state, &uuid).await?;
    let url = {
        let accs = state.accounts.lock().await;
        accs.list.iter().find(|a| a.uuid == uuid).and_then(|a| a.skin_url.clone())
    };
    let Some(url) = url else { bail!("To konto nie ma własnego skina") };
    let p = profile::set_variant(&state.ctx, &token, &url, slim).await?;
    adopt(&state, &uuid, &p).await
}
