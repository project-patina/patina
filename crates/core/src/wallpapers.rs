//! User images (wallpapers, banners, saved skins) copied into the data dir and
//! served to the UI by id.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{bail, Ctx, Result};

const MAX_BYTES: u64 = 30 * 1024 * 1024;

/// `wallpapers` for backgrounds/banners, `skins` for the skin library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Store {
    Wallpapers,
    Skins,
}

impl Store {
    pub fn dir(self, ctx: &Ctx) -> PathBuf {
        ctx.paths.root.join(match self {
            Store::Wallpapers => "wallpapers",
            Store::Skins => "skins",
        })
    }
    fn exts(self) -> &'static [&'static str] {
        match self {
            Store::Wallpapers => &["png", "jpg", "jpeg", "webp", "gif"],
            Store::Skins => &["png"],
        }
    }
}

pub fn mime(name: &str) -> &'static str {
    match name.rsplit('.').next().map(str::to_lowercase).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

/// Ids are `<uuid>.<ext>`; anything else is rejected before touching disk.
pub fn valid_id(id: &str) -> bool {
    let Some((stem, ext)) = id.rsplit_once('.') else { return false };
    stem.len() == 32 && stem.chars().all(|c| c.is_ascii_hexdigit()) && ext.chars().all(|c| c.is_ascii_alphanumeric()) && ext.len() <= 5
}

pub fn path(ctx: &Ctx, store: Store, id: &str) -> Result<PathBuf> {
    if !valid_id(id) {
        bail!("Niepoprawny identyfikator obrazu", "Invalid image id");
    }
    Ok(store.dir(ctx).join(id))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: String,
    pub size: u64,
    pub added: i64,
    /// Original file name, for display.
    pub name: String,
}

/// Width/height from a PNG header.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    Some((w, h))
}

pub fn import(ctx: &Ctx, store: Store, src: &Path) -> Result<Image> {
    let ext = src.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    if !store.exts().contains(&ext.as_str()) {
        bail!("Nieobsługiwany format pliku (.{ext})", "Unsupported file format (.{ext})");
    }
    let meta = std::fs::metadata(src)?;
    if meta.len() > MAX_BYTES {
        bail!("Plik jest za duży (maks. 30 MB)", "The file is too large (30 MB at most)");
    }
    let bytes = std::fs::read(src)?;
    if store == Store::Skins {
        match png_size(&bytes) {
            Some((64, 64)) | Some((64, 32)) => {}
            _ => bail!("Skin musi być plikiem PNG 64×64 (lub starym 64×32)", "A skin must be a 64×64 PNG file (or an old 64×32 one)"),
        }
    }
    let id = format!("{}.{ext}", uuid::Uuid::new_v4().simple());
    let dir = store.dir(ctx);
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(&id), &bytes)?;
    let name = src.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    std::fs::write(dir.join(format!("{id}.name")), &name)?;
    Ok(Image { id, size: meta.len(), added: chrono::Utc::now().timestamp(), name })
}

pub fn list(ctx: &Ctx, store: Store) -> Vec<Image> {
    let dir = store.dir(ctx);
    let mut out: Vec<Image> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let id = e.file_name().to_string_lossy().to_string();
            if !valid_id(&id) {
                return None;
            }
            let m = e.metadata().ok()?;
            let added = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64).unwrap_or(0);
            let name = std::fs::read_to_string(dir.join(format!("{id}.name"))).unwrap_or_default();
            Some(Image { id, size: m.len(), added, name })
        })
        .collect();
    out.sort_by_key(|i| std::cmp::Reverse(i.added));
    out
}

pub fn delete(ctx: &Ctx, store: Store, id: &str) -> Result<()> {
    let p = path(ctx, store, id)?;
    let _ = std::fs::remove_file(p.with_file_name(format!("{id}.name")));
    if p.exists() {
        std::fs::remove_file(p)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn ids() {
        assert!(super::valid_id("0123456789abcdef0123456789abcdef.png"));
        assert!(!super::valid_id("../etc/passwd"));
        assert!(!super::valid_id("0123456789abcdef0123456789abcdef.png/../x"));
    }

    #[test]
    fn png_header() {
        let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
        b.extend(64u32.to_be_bytes());
        b.extend(32u32.to_be_bytes());
        assert_eq!(super::png_size(&b), Some((64, 32)));
        assert_eq!(super::png_size(b"nope"), None);
    }
}
