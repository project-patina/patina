//! The sword and shield of the player model on the home screen, read from the
//! game the player already has. Nothing here is stored or shipped with the launcher.

use std::io::Read;
use std::path::Path;

use base64::Engine;
use serde::Serialize;

use crate::Ctx;

const SWORD: &str = "assets/minecraft/textures/item/diamond_sword.png";
/// Moved into `entity/shield/` in later versions.
const SHIELD: [&str; 2] = ["assets/minecraft/textures/entity/shield/shield_base_nopattern.png", "assets/minecraft/textures/entity/shield_base_nopattern.png"];

/// PNG `data:` URLs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GearTextures {
    pub sword: String,
    pub shield: String,
}

/// The first installed game that holds both textures; `None` before any is installed.
pub fn load(ctx: &Ctx) -> Option<GearTextures> {
    std::fs::read_dir(ctx.paths.versions()).ok()?.flatten().find_map(|entry| {
        let id = entry.file_name().into_string().ok()?;
        from_jar(&ctx.paths.client_jar(&id))
    })
}

fn from_jar(jar: &Path) -> Option<GearTextures> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(jar).ok()?).ok()?;
    let sword = entry_url(&mut zip, SWORD)?;
    let shield = SHIELD.iter().find_map(|path| entry_url(&mut zip, path))?;
    Some(GearTextures { sword, shield })
}

fn entry_url(zip: &mut zip::ZipArchive<std::fs::File>, path: &str) -> Option<String> {
    let mut bytes = Vec::new();
    zip.by_name(path).ok()?.read_to_end(&mut bytes).ok()?;
    Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}
