//! The integrated Patina client mod for Fabric instances of 26.3: the loading screen,
//! the main menu and its signature. The Rust client has all of this built in.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::content::ContentKind;
use crate::instance::Instance;
use crate::loaders::Loader;
use crate::paths::write_atomic;
use crate::{engine, Ctx, Result};

/// The file in the instance's `mods` folder.
pub const FILE: &str = "patina-client.jar";

#[cfg(patina_mod_jar)]
const JAR: Option<&[u8]> = Some(include_bytes!("../../../resources/patina-mod/patina-client-26.3.jar"));
#[cfg(not(patina_mod_jar))]
const JAR: Option<&[u8]> = None;

/// Run before every launch. The jar requires 26.3, so an instance moved to another
/// version must lose it or Fabric refuses to start; no other mod is touched.
pub fn sync(ctx: &Ctx, inst: &Instance) -> Result<()> {
    place(&ctx.paths.game_dir(&inst.id), inst.loader, &inst.mc_version, JAR)
}

fn place(game_dir: &Path, loader: Loader, mc_version: &str, jar: Option<&[u8]>) -> Result<()> {
    let path = game_dir.join(ContentKind::Mod.dir()).join(FILE);
    if loader != Loader::Fabric {
        return Ok(());
    }
    if mc_version != engine::MC_VERSION {
        return remove(&path);
    }
    let Some(jar) = jar else { return Ok(()) };
    remove(&path.with_extension("jar.disabled"))?;
    if fs::read(&path).map_or(true, |current| current != jar) {
        write_atomic(&path, jar)?;
    }
    Ok(())
}

fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_dir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("patina-mod-{}", uuid::Uuid::new_v4().simple()))
    }

    #[test]
    fn fabric_26_3_gets_the_jar_and_a_changed_one_is_replaced() {
        let dir = game_dir();
        let jar = dir.join("mods").join(FILE);
        place(&dir, Loader::Fabric, engine::MC_VERSION, Some(b"new")).unwrap();
        assert_eq!(fs::read(&jar).unwrap(), b"new");
        fs::write(&jar, b"old").unwrap();
        fs::write(dir.join("mods/sodium.jar"), b"mod").unwrap();
        fs::write(dir.join("mods/patina-client.jar.disabled"), b"off").unwrap();
        place(&dir, Loader::Fabric, engine::MC_VERSION, Some(b"new")).unwrap();
        assert_eq!(fs::read(&jar).unwrap(), b"new");
        assert_eq!(fs::read(dir.join("mods/sodium.jar")).unwrap(), b"mod");
        assert!(!dir.join("mods/patina-client.jar.disabled").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn other_versions_and_loaders_get_nothing() {
        let dir = game_dir();
        place(&dir, Loader::Fabric, "1.21.1", Some(b"new")).unwrap();
        place(&dir, Loader::Rust, engine::MC_VERSION, Some(b"new")).unwrap();
        place(&dir, Loader::Quilt, engine::MC_VERSION, Some(b"new")).unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn an_instance_moved_off_26_3_loses_only_this_jar() {
        let dir = game_dir();
        fs::create_dir_all(dir.join("mods")).unwrap();
        fs::write(dir.join("mods").join(FILE), b"old").unwrap();
        fs::write(dir.join("mods/sodium.jar"), b"mod").unwrap();
        place(&dir, Loader::Fabric, "26.4", Some(b"new")).unwrap();
        assert!(!dir.join("mods").join(FILE).exists());
        assert!(dir.join("mods/sodium.jar").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_build_without_the_jar_leaves_the_folder_alone() {
        let dir = game_dir();
        place(&dir, Loader::Fabric, engine::MC_VERSION, None).unwrap();
        assert!(!dir.exists());
    }
}
