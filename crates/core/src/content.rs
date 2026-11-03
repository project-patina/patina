//! An instance's installable content (mods, resource packs, shaders) and the
//! metadata we remember about each file (`content.json`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};

use crate::{bail, Ctx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    #[default]
    Mod,
    Resourcepack,
    Shader,
}

impl ContentKind {
    pub fn dir(self) -> &'static str {
        match self {
            ContentKind::Mod => "mods",
            ContentKind::Resourcepack => "resourcepacks",
            ContentKind::Shader => "shaderpacks",
        }
    }

    pub fn project_type(self) -> &'static str {
        match self {
            ContentKind::Mod => "mod",
            ContentKind::Resourcepack => "resourcepack",
            ContentKind::Shader => "shader",
        }
    }

    fn accepts(self, name: &str) -> bool {
        let n = name.to_lowercase();
        match self {
            ContentKind::Mod => n.ends_with(".jar"),
            _ => n.ends_with(".zip"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentMeta {
    pub kind: ContentKind,
    pub project_id: String,
    pub version_id: String,
    pub title: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub version_number: String,
}

/// Keyed by file name (without `.disabled`).
pub type MetaMap = HashMap<String, ContentMeta>;

/// Serialises read-modify-write of an instance's `content.json`.
pub async fn lock(inst: &str) -> tokio::sync::OwnedMutexGuard<()> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    let m = LOCKS.get_or_init(Default::default).lock().unwrap().entry(inst.to_string()).or_default().clone();
    m.lock_owned().await
}

fn meta_path(ctx: &Ctx, inst: &str) -> PathBuf {
    ctx.paths.instance_dir(inst).join("content.json")
}

pub fn load_meta(ctx: &Ctx, inst: &str) -> MetaMap {
    std::fs::read(meta_path(ctx, inst)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

pub fn save_meta(ctx: &Ctx, inst: &str, meta: &MetaMap) -> Result<()> {
    crate::paths::write_atomic(&meta_path(ctx, inst), &serde_json::to_vec_pretty(meta)?)?;
    Ok(())
}

/// A file name we are willing to touch: no paths, no hidden files.
pub fn safe_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && !name.starts_with('.') && name != ".."
}

/// `(name without .disabled, actual path)` of every file of `kind`.
pub fn files(ctx: &Ctx, inst: &str, kind: ContentKind) -> Vec<(String, PathBuf)> {
    let dir = ctx.paths.game_dir(inst).join(kind.dir());
    let mut out = Vec::new();
    for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
        let raw = e.file_name().to_string_lossy().to_string();
        let name = raw.strip_suffix(".disabled").unwrap_or(&raw).to_string();
        if kind.accepts(&name) && e.path().is_file() {
            out.push((name, e.path()));
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentItem {
    pub file: String,
    pub enabled: bool,
    pub size: u64,
    pub meta: Option<ContentMeta>,
}

pub fn list(ctx: &Ctx, inst: &str, kind: ContentKind) -> Vec<ContentItem> {
    let meta = load_meta(ctx, inst);
    let mut out: Vec<ContentItem> = files(ctx, inst, kind)
        .into_iter()
        .map(|(name, path)| ContentItem {
            enabled: !path.to_string_lossy().ends_with(".disabled"),
            size: path.metadata().map(|m| m.len()).unwrap_or(0),
            meta: meta.get(&name).cloned(),
            file: name,
        })
        .collect();
    out.sort_by_key(|c| c.meta.as_ref().map(|m| m.title.clone()).unwrap_or_else(|| c.file.clone()).to_lowercase());
    out
}

pub fn set_enabled(ctx: &Ctx, inst: &str, kind: ContentKind, file: &str, enabled: bool) -> Result<()> {
    if !safe_name(file) {
        bail!("Niepoprawna nazwa pliku", "Invalid file name");
    }
    let dir = ctx.paths.game_dir(inst).join(kind.dir());
    let on = dir.join(file);
    let off = dir.join(format!("{file}.disabled"));
    let (from, to) = if enabled { (off, on) } else { (on, off) };
    if from.exists() {
        std::fs::rename(from, to)?;
    }
    Ok(())
}

/// Delete a file (to the trash when possible) without touching metadata.
pub fn remove_file(ctx: &Ctx, inst: &str, kind: ContentKind, file: &str) -> Result<()> {
    if !safe_name(file) {
        bail!("Niepoprawna nazwa pliku", "Invalid file name");
    }
    let dir = ctx.paths.game_dir(inst).join(kind.dir());
    for p in [dir.join(file), dir.join(format!("{file}.disabled"))] {
        if p.exists() && trash::delete(&p).is_err() {
            std::fs::remove_file(&p)?;
        }
    }
    Ok(())
}

pub async fn delete(ctx: &Ctx, inst: &str, kind: ContentKind, file: &str) -> Result<()> {
    remove_file(ctx, inst, kind, file)?;
    forget(ctx, inst, &[file.to_string()]).await
}

/// Drop metadata for files that were removed or replaced by something else.
async fn forget(ctx: &Ctx, inst: &str, files: &[String]) -> Result<()> {
    let _guard = lock(inst).await;
    let mut meta = load_meta(ctx, inst);
    let before = meta.len();
    for f in files {
        meta.remove(f);
    }
    if meta.len() != before {
        save_meta(ctx, inst, &meta)?;
    }
    Ok(())
}

/// Copy local files (picked by the user) into the instance.
pub async fn import(ctx: &Ctx, inst: &str, kind: ContentKind, paths: &[PathBuf]) -> Result<Vec<String>> {
    let dir = ctx.paths.game_dir(inst).join(kind.dir());
    std::fs::create_dir_all(&dir)?;
    let mut out = Vec::new();
    for p in paths {
        let Some(name) = p.file_name().map(|n| n.to_string_lossy().to_string()) else { continue };
        if !kind.accepts(&name) || !safe_name(&name) {
            bail!("{name}: zły typ pliku (oczekiwano {})", "{name}: wrong file type (expected {})", if kind == ContentKind::Mod { ".jar" } else { ".zip" });
        }
        std::fs::copy(p, dir.join(&name))?;
        out.push(name);
    }
    // An overwritten file is no longer what its old metadata says.
    forget(ctx, inst, &out).await?;
    Ok(out)
}
