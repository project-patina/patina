//! Official Patina mods for the Rust client: a signed index in
//! `project-patina/patina-mods` and `.pmod` packages in that repository's releases.
//!
//! The index is checked against its signature before it is parsed, with the key that
//! signs launcher updates, and it pins each package's SHA-256 and size. Only an index
//! that passed picks what gets downloaded: the UI just names a mod.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::instance::Instance;
use crate::lang::tr;
use crate::loaders::Loader;
use crate::paths::write_atomic;
use crate::update::{self, Notes, Package, Source};
use crate::{bail, Ctx, Error, Result};

const INDEX: &str = "https://raw.githubusercontent.com/project-patina/patina-mods/main/";
const RELEASES: &str = "https://github.com/project-patina/patina-mods/releases/download/";
const SCHEMA: u32 = 1;
const MAX_INDEX: usize = 256 * 1024;
const MAX_PACKAGE: u64 = 64 * 1024 * 1024;
const MAX_ID: usize = 64;

/// What the engine reads, beside `options.txt` (`mod_list.rs` in `engine/`).
const MODS_DIR: &str = "patina-mods";
const MODS_LIST: &str = "patina-mods.txt";
const EXTENSION: &str = "pmod";

#[derive(Debug, Deserialize)]
struct Index {
    schema: u32,
    mods: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    id: String,
    name: String,
    version: String,
    mc_version: String,
    summary: Notes,
    icon: Option<String>,
    #[serde(flatten)]
    package: Package,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModState {
    Available,
    Installed,
    Outdated,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: Notes,
    pub icon: Option<String>,
    pub size: u64,
    pub state: ModState,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    /// False when the index cannot be reached: offline, or nothing published yet.
    pub available: bool,
    pub mods: Vec<OfficialMod>,
    /// Ids of installed mods the index does not list, so they can still be removed.
    pub local: Vec<String>,
}

pub fn source() -> Source {
    Source::repository(INDEX, RELEASES)
}

/// The engine's rule for mod ids, which also makes an id safe as a file name.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

fn check(entry: &Entry, source: &Source) -> bool {
    valid_id(&entry.id)
        && !entry.name.trim().is_empty()
        && source.package_allowed(&entry.package.url)
        && update::is_sha256(&entry.package.sha256)
        && (1..=MAX_PACKAGE).contains(&entry.package.size)
        && entry.icon.as_deref().is_none_or(|icon| icon.starts_with("https://"))
}

fn parse(bytes: &[u8], source: &Source) -> Result<Vec<Entry>> {
    let index: Index = serde_json::from_slice(bytes)?;
    if index.schema != SCHEMA {
        bail!("Lista modów wymaga nowszego launchera", "The mod list needs a newer launcher");
    }
    if !index.mods.iter().all(|entry| check(entry, source)) {
        return Err(Error::msg(tr("Nieprawidłowa lista modów", "Invalid mod list")));
    }
    Ok(index.mods)
}

/// `Ok(None)` when the server has no index or cannot be reached.
async fn fetch(source: &Source) -> Result<Option<Vec<Entry>>> {
    async fn get(http: &reqwest::Client, url: String, limit: usize) -> Result<Option<Vec<u8>>> {
        match update::bounded_get(http, &url, limit).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(Error::Http(_)) => Ok(None),
            Err(error) => Err(error),
        }
    }
    let http = source.client()?;
    let Some(index) = get(&http, format!("{}index.json", source.base()), MAX_INDEX).await? else { return Ok(None) };
    let Some(signature) = get(&http, format!("{}index.json.sig", source.base()), update::MAX_SIGNATURE).await? else { return Ok(None) };
    update::verify_signature(&index, &signature, source.public_key()).map_err(|_| Error::msg(tr("Podpis listy modów jest nieprawidłowy", "The mod list's signature is invalid")))?;
    parse(&index, source).map(Some)
}

fn mods_dir(ctx: &Ctx, inst: &Instance) -> PathBuf {
    ctx.paths.game_dir(&inst.id).join(MODS_DIR)
}

fn package_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.{EXTENSION}"))
}

fn state_of(path: &Path, sha256: &str) -> ModState {
    match fs::read(path) {
        Ok(bytes) if hex::encode(Sha256::digest(&bytes)).eq_ignore_ascii_case(sha256) => ModState::Installed,
        Ok(_) => ModState::Outdated,
        Err(_) => ModState::Available,
    }
}

/// `.pmod` files other than `listed`, by id.
fn local_mods(dir: &Path, listed: &[OfficialMod]) -> Vec<String> {
    let mut ids: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == EXTENSION))
        .filter_map(|path| Some(path.file_stem()?.to_str()?.to_string()))
        .filter(|id| valid_id(id) && !listed.iter().any(|m| m.id == *id))
        .collect();
    ids.sort();
    ids
}

/// What the index offers this instance. Being offline is not an error here.
pub async fn catalog(ctx: &Ctx, inst: &Instance, source: &Source) -> Result<Catalog> {
    let dir = mods_dir(ctx, inst);
    let entries = fetch(source).await?;
    let available = entries.is_some();
    let mods: Vec<OfficialMod> = entries
        .into_iter()
        .flatten()
        .filter(|entry| entry.mc_version == inst.mc_version)
        .map(|entry| OfficialMod {
            state: state_of(&package_path(&dir, &entry.id), &entry.package.sha256),
            id: entry.id,
            name: entry.name,
            version: entry.version,
            summary: entry.summary,
            icon: entry.icon,
            size: entry.package.size,
        })
        .collect();
    let local = local_mods(&dir, &mods);
    Ok(Catalog { available, mods, local })
}

/// Everything a line of `patina-mods.txt` says about `id` goes; `on` adds it back as on.
fn set_listed(game_dir: &Path, id: &str, on: bool) -> Result<()> {
    let path = game_dir.join(MODS_LIST);
    let key = format!("mod.{id}");
    let current = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound && !on => return Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let mut lines: Vec<&str> = current.lines().filter(|line| line.split_once(':').is_none_or(|(k, _)| k.trim() != key)).collect();
    let added = format!("{key}:true");
    if on {
        lines.push(&added);
    }
    write_atomic(&path, format!("{}\n", lines.join("\n")).trim_start().as_bytes())?;
    Ok(())
}

/// Installs or updates `id`. A new mod is turned on: the game keeps mods off until
/// told otherwise, and an update leaves the player's choice alone.
pub async fn install(ctx: &Ctx, inst: &Instance, source: &Source, id: &str) -> Result<()> {
    if !valid_id(id) {
        bail!("Niepoprawny identyfikator moda", "Invalid mod id");
    }
    if inst.loader != Loader::Rust {
        bail!("Mody Patina działają tylko w instancjach Rust", "Patina mods only work in Rust instances");
    }
    let Some(entries) = fetch(source).await? else {
        bail!("Nie udało się pobrać listy modów", "Couldn't download the mod list");
    };
    let Some(entry) = entries.into_iter().find(|entry| entry.id == id && entry.mc_version == inst.mc_version) else {
        bail!("Tego moda nie ma na liście", "This mod is not on the list");
    };

    let dir = mods_dir(ctx, inst);
    fs::create_dir_all(&dir)?;
    let target = package_path(&dir, id);
    let fresh = !target.exists();
    // The extension keeps a partial file out of the engine's sight.
    let partial = dir.join(format!(".{id}.{}.part", uuid::Uuid::new_v4().simple()));
    let placed = place(source, &entry.package, &partial, &target).await;
    if placed.is_err() {
        let _ = fs::remove_file(&partial);
    }
    placed?;
    if fresh {
        set_listed(&ctx.paths.game_dir(&inst.id), id, true)?;
    }
    Ok(())
}

async fn place(source: &Source, package: &Package, partial: &Path, target: &Path) -> Result<()> {
    let got = update::download(&source.client()?, package, partial, &|_| {}).await?;
    update::check_hash(package, &got)?;
    fs::rename(partial, target)?;
    Ok(())
}

pub fn remove(ctx: &Ctx, inst: &Instance, id: &str) -> Result<()> {
    if !valid_id(id) {
        bail!("Niepoprawny identyfikator moda", "Invalid mod id");
    }
    match fs::remove_file(package_path(&mods_dir(ctx, inst), id)) {
        Err(e) if e.kind() != ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    set_listed(&ctx.paths.game_dir(&inst.id), id, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::NewInstance;
    use crate::paths::Paths;
    use crate::update::tests::{answer, keys, sha, Keys};
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn ctx() -> Ctx {
        Ctx::new(Paths::new(std::env::temp_dir().join(format!("patina-official-{}", uuid::Uuid::new_v4().simple())))).unwrap()
    }

    fn rust_instance(ctx: &Ctx) -> Instance {
        Instance::create(ctx, NewInstance { name: "Rust".into(), mc_version: crate::engine::MC_VERSION.into(), loader: Loader::Rust, loader_version: None, icon: None, banner: None }).unwrap()
    }

    fn index(mods: serde_json::Value) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({ "schema": 1, "mods": mods })).unwrap()
    }

    fn entry(id: &str, url: &str, package: &[u8]) -> serde_json::Value {
        serde_json::json!({
            "id": id, "name": "Mapa", "version": "1.0.0", "mcVersion": crate::engine::MC_VERSION,
            "summary": { "pl": "Opis", "en": "Summary" },
            "url": url, "sha256": sha(package), "size": package.len(),
        })
    }

    fn strict() -> Source {
        Source::repository(INDEX, RELEASES)
    }

    /// The index names the server's own address, so the port comes first.
    fn listener() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        (listener, base)
    }

    fn signed(keys: &Keys, bytes: Vec<u8>) -> Vec<(String, Vec<u8>)> {
        let signature = keys.sign(&bytes);
        vec![("index.json".into(), bytes), ("index.json.sig".into(), signature)]
    }

    #[test]
    fn ids_cannot_leave_the_mods_folder() {
        for id in ["", "../evil", "a/b", "a\\b", "..", ".hidden", "UPPER", "a.b", "a-b", &"a".repeat(65)] {
            assert!(!valid_id(id), "{id}");
        }
        assert!(valid_id("patina_map2"));
    }

    #[test]
    fn a_traversal_in_an_id_rejects_the_index() {
        let url = format!("{RELEASES}v1/map.pmod");
        assert_eq!(parse(&index(serde_json::json!([entry("map", &url, b"x")])), &strict()).unwrap().len(), 1);
        assert!(parse(&index(serde_json::json!([entry("../../evil", &url, b"x")])), &strict()).is_err());
    }

    #[test]
    fn packages_must_come_from_this_repository() {
        for url in [
            "https://github.com/project-patina/patina-updates/releases/download/v1/map.pmod",
            "https://github.com/project-patina/patina-mods/releases/download/../../../evil/map.pmod",
            "http://github.com/project-patina/patina-mods/releases/download/v1/map.pmod",
            "https://example.com/map.pmod",
        ] {
            assert!(parse(&index(serde_json::json!([entry("map", url, b"x")])), &strict()).is_err(), "{url}");
        }
    }

    #[test]
    fn sizes_hashes_icons_and_schema_are_checked() {
        let url = format!("{RELEASES}v1/map.pmod");
        let with = |key: &str, value: serde_json::Value| {
            let mut mod_entry = entry("map", &url, b"x");
            mod_entry[key] = value;
            index(serde_json::json!([mod_entry]))
        };
        assert!(parse(&with("size", serde_json::json!(MAX_PACKAGE + 1)), &strict()).is_err());
        assert!(parse(&with("size", serde_json::json!(0)), &strict()).is_err());
        assert!(parse(&with("sha256", serde_json::json!("abc")), &strict()).is_err());
        assert!(parse(&with("icon", serde_json::json!("http://example.com/a.png")), &strict()).is_err());
        assert!(parse(&with("icon", serde_json::json!("https://example.com/a.png")), &strict()).is_ok());
        assert!(parse(br#"{"schema":2,"mods":[]}"#, &strict()).is_err());
    }

    #[tokio::test]
    async fn an_index_failing_its_signature_is_never_parsed() {
        let keys = keys();
        let (listener, base) = listener();
        let server = answer(listener, vec![("index.json".into(), b"{ not json".to_vec()), ("index.json.sig".into(), b"bad".to_vec())]);
        let ctx = ctx();
        let error = catalog(&ctx, &rust_instance(&ctx), &Source::testing(&base, &keys.public)).await.unwrap_err();
        assert!(!matches!(error, Error::Json(_)), "{error}");
        server.join().unwrap();
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }

    #[tokio::test]
    async fn another_keys_signature_is_refused() {
        let (signer, reader) = (keys(), keys());
        let (listener, base) = listener();
        let server = answer(listener, signed(&signer, index(serde_json::json!([]))));
        let ctx = ctx();
        assert!(catalog(&ctx, &rust_instance(&ctx), &Source::testing(&base, &reader.public)).await.is_err());
        server.join().unwrap();
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }

    #[tokio::test]
    async fn no_repository_or_no_network_is_a_quiet_state() {
        let (listener, base) = listener();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = stream.read(&mut [0; 4096]).unwrap();
            stream.write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").unwrap();
        });
        let ctx = ctx();
        let inst = rust_instance(&ctx);
        let keys = keys();
        let missing = catalog(&ctx, &inst, &Source::testing(&base, &keys.public)).await.unwrap();
        assert!(!missing.available && missing.mods.is_empty());
        server.join().unwrap();
        // Nothing listens on this port any more.
        let offline = catalog(&ctx, &inst, &Source::testing(&base, &keys.public)).await.unwrap();
        assert!(!offline.available);
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }

    #[tokio::test]
    async fn installs_updates_and_removes_a_signed_mod() {
        let keys = keys();
        let ctx = ctx();
        let inst = rust_instance(&ctx);
        let game = ctx.paths.game_dir(&inst.id);
        fs::create_dir_all(&game).unwrap();
        fs::write(game.join(MODS_LIST), "mod.other:true\nkey.other.zoom:key.keyboard.z\n").unwrap();

        let (listener, base) = listener();
        let source = Source::testing(&base, &keys.public);
        let package = b"pmod bytes".to_vec();
        let mut other_version = entry("other_version", &format!("{base}x.pmod"), b"x");
        other_version["mcVersion"] = "1.21.1".into();
        let bytes = index(serde_json::json!([entry("map", &format!("{base}map.pmod"), &package), other_version]));
        let mut responses = signed(&keys, bytes.clone());
        responses.extend(signed(&keys, bytes.clone()));
        responses.push(("map.pmod".into(), package.clone()));
        responses.extend(signed(&keys, bytes.clone()));
        let server = answer(listener, responses);

        let before = catalog(&ctx, &inst, &source).await.unwrap();
        assert!(before.available);
        assert_eq!(before.mods.iter().map(|m| (m.id.as_str(), m.state)).collect::<Vec<_>>(), [("map", ModState::Available)]);

        install(&ctx, &inst, &source, "map").await.unwrap();
        assert_eq!(fs::read(game.join("patina-mods/map.pmod")).unwrap(), package);
        assert_eq!(fs::read_dir(game.join(MODS_DIR)).unwrap().count(), 1);
        let list = fs::read_to_string(game.join(MODS_LIST)).unwrap();
        assert_eq!(list, "mod.other:true\nkey.other.zoom:key.keyboard.z\nmod.map:true\n");

        fs::write(game.join("patina-mods/map.pmod"), b"older").unwrap();
        fs::write(game.join("patina-mods/mine.pmod"), b"dropped in").unwrap();
        let after = catalog(&ctx, &inst, &source).await.unwrap();
        assert_eq!(after.mods[0].state, ModState::Outdated);
        assert_eq!(after.local, ["mine"]);
        server.join().unwrap();

        remove(&ctx, &inst, "map").unwrap();
        assert!(!game.join("patina-mods/map.pmod").exists());
        assert_eq!(fs::read_to_string(game.join(MODS_LIST)).unwrap(), "mod.other:true\nkey.other.zoom:key.keyboard.z\n");
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }

    #[tokio::test]
    async fn a_wrong_hash_installs_nothing_and_leaves_no_partial_file() {
        let keys = keys();
        let ctx = ctx();
        let inst = rust_instance(&ctx);
        let (listener, base) = listener();
        let bytes = index(serde_json::json!([entry("map", &format!("{base}map.pmod"), b"expected!!")]));
        let mut responses = signed(&keys, bytes);
        responses.push(("map.pmod".into(), b"tampered!!".to_vec()));
        let server = answer(listener, responses);
        assert!(install(&ctx, &inst, &Source::testing(&base, &keys.public), "map").await.is_err());
        server.join().unwrap();
        assert_eq!(fs::read_dir(mods_dir(&ctx, &inst)).unwrap().count(), 0);
        assert!(!ctx.paths.game_dir(&inst.id).join(MODS_LIST).exists());
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }

    #[tokio::test]
    async fn unsafe_ids_and_other_loaders_touch_nothing() {
        let ctx = ctx();
        let inst = rust_instance(&ctx);
        let outside = ctx.paths.instance_dir(&inst.id).join("keep.pmod");
        fs::write(&outside, b"keep").unwrap();
        // The source is never contacted: these all stop first.
        let source = Source::testing("http://127.0.0.1:9/", "");
        assert!(install(&ctx, &inst, &source, "../../keep").await.is_err());
        assert!(remove(&ctx, &inst, "../../keep").is_err());
        assert!(remove(&ctx, &inst, "..\\..\\keep").is_err());
        assert!(outside.exists());
        let fabric = Instance { loader: Loader::Fabric, ..inst };
        assert!(install(&ctx, &fabric, &source, "map").await.is_err());
        fs::remove_dir_all(&ctx.paths.root).unwrap();
    }
}
