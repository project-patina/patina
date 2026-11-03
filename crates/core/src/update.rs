//! Self-update of the portable folders we ship: the launcher and the engine.
//!
//! `latest.json` is checked against its minisign signature before it is parsed,
//! and it pins the package's SHA-256 and size. A package is a zip with `launcher/`
//! and `engine/` at the top. Files are swapped by rename with a backup beside
//! each one, and the launcher itself goes last, so a failure at any step leaves
//! the previous version whole.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use base64::Engine;
use reqwest::Url;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::lang::tr;
use crate::{bail, engine, Ctx, Error, Result};

/// Public half of the release signing key. Generated with `tauri signer generate`.
pub const PUBLIC_KEY: &str = "RWRqG2msT/cWm7QKSxzGO5e42+tFKzLtqDqPPNYs8ZIzV2mN8OrT96hu";
const UPDATES: &str = "https://raw.githubusercontent.com/project-patina/patina-updates/main/";
const RELEASES: &str = "https://github.com/project-patina/patina-updates/releases/download/";
const REDIRECT_HOSTS: &str = ".githubusercontent.com";

const MAX_MANIFEST: usize = 64 * 1024;
pub(crate) const MAX_SIGNATURE: usize = 4 * 1024;
const MAX_PACKAGE: u64 = 400 * 1024 * 1024;
const MAX_UNPACKED: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

/// Replaced whole: `patina-prepare` rejects a folder with classes its manifest does not list.
const WHOLE_DIRS: [&str; 2] = ["mods/patina/src/test/java", "exporters"];
const NEW_SUFFIX: &str = ".patina-new";
const OLD_SUFFIX: &str = ".patina-old";

/// The running version; a test build can pretend to be older.
pub fn current_version() -> &'static str {
    match option_env!("PATINA_TEST_VERSION") {
        Some(version) => version,
        None => crate::LAUNCHER_VERSION,
    }
}

/// Where updates come from. Only tests and test builds differ from [`Source::official`].
#[derive(Debug, Clone)]
pub struct Source {
    base: String,
    packages: String,
    public_key: String,
    /// HTTPS everywhere, with redirects to GitHub's download hosts only.
    strict: bool,
}

impl Source {
    pub fn official() -> Self {
        match option_env!("PATINA_TEST_UPDATE_BASE") {
            Some(base) => Self::testing(base, PUBLIC_KEY),
            None => Self { base: UPDATES.into(), packages: RELEASES.into(), public_key: PUBLIC_KEY.into(), strict: true },
        }
    }

    /// Another of the project's repositories, signed with the same key.
    pub(crate) fn repository(base: &str, packages: &str) -> Self {
        Self { base: base.into(), packages: packages.into(), public_key: PUBLIC_KEY.into(), strict: true }
    }

    /// Everything, packages included, comes from `base`, on any scheme.
    pub(crate) fn testing(base: &str, public_key: &str) -> Self {
        let base = format!("{}/", base.trim_end_matches('/'));
        Self { packages: base.clone(), base, public_key: public_key.into(), strict: false }
    }

    pub(crate) fn base(&self) -> &str {
        &self.base
    }

    pub(crate) fn public_key(&self) -> &str {
        &self.public_key
    }

    pub(crate) fn client(&self) -> Result<reqwest::Client> {
        let source = self.clone();
        let redirects = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() >= 5 {
                attempt.error("too many redirects")
            } else if source.redirect_allowed(attempt.url()) {
                attempt.follow()
            } else {
                attempt.error("redirect outside the update hosts")
            }
        });
        let mut builder = reqwest::Client::builder()
            .user_agent(format!("{}-Launcher/{}", crate::LAUNCHER_NAME, crate::LAUNCHER_VERSION))
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(60))
            .redirect(redirects)
            .gzip(false);
        builder = if self.strict { builder.https_only(true) } else { builder.no_proxy() };
        Ok(builder.build()?)
    }

    fn redirect_allowed(&self, url: &Url) -> bool {
        if self.strict {
            url.scheme() == "https" && url.host_str().is_some_and(|h| h.ends_with(REDIRECT_HOSTS))
        } else {
            url.as_str().starts_with(&self.base)
        }
    }

    pub(crate) fn package_allowed(&self, url: &str) -> bool {
        Url::parse(url).is_ok_and(|u| u.as_str().starts_with(&self.packages) && (!self.strict || u.scheme() == "https"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notes {
    pub pl: String,
    pub en: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Package {
    pub(crate) url: String,
    pub(crate) sha256: String,
    pub(crate) size: u64,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    critical: bool,
    date: String,
    notes: Notes,
    platforms: HashMap<String, Package>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    pub critical: bool,
    pub date: String,
    pub notes: Notes,
    pub size_bytes: u64,
    pub installable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    /// `download`, `verify` or `install`.
    pub stage: &'static str,
    pub done: u64,
    pub total: u64,
}

/// A newer version this platform can install, from a manifest that passed its signature check.
#[derive(Debug)]
pub struct Release {
    manifest: Manifest,
    package: Package,
    current: String,
}

impl Release {
    pub fn info(&self, installable: bool) -> UpdateInfo {
        UpdateInfo {
            version: self.manifest.version.clone(),
            current: self.current.clone(),
            critical: self.manifest.critical,
            date: self.manifest.date.clone(),
            notes: self.manifest.notes.clone(),
            size_bytes: self.package.size,
            installable,
        }
    }
}

fn platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// The manifest's exact bytes against `latest.json.sig`: base64 of a minisign `.minisig` text.
pub(crate) fn verify_signature(manifest: &[u8], signature: &[u8], public_key: &str) -> Result<()> {
    let failed = || Error::msg(tr("Podpis aktualizacji jest nieprawidłowy", "The update signature is invalid"));
    let text = std::str::from_utf8(signature).map_err(|_| failed())?;
    let decoded = base64::engine::general_purpose::STANDARD.decode(text.trim()).map_err(|_| failed())?;
    let decoded = String::from_utf8(decoded).map_err(|_| failed())?;
    let signature = minisign_verify::Signature::decode(&decoded).map_err(|_| failed())?;
    let key = minisign_verify::PublicKey::from_base64(public_key).map_err(|_| failed())?;
    key.verify(manifest, &signature, false).map_err(|_| failed())
}

pub(crate) fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What `manifest` offers `platform`, if it is newer than `current`.
fn select(mut manifest: Manifest, source: &Source, current: &str, platform: &str) -> Result<Option<Release>> {
    let bad = || Error::msg(tr("Nieprawidłowy manifest aktualizacji", "Invalid update manifest"));
    let offered = Version::parse(&manifest.version).map_err(|_| bad())?;
    let running = Version::parse(current).map_err(|_| bad())?;
    if offered <= running {
        return Ok(None);
    }
    let Some(package) = manifest.platforms.remove(platform) else { return Ok(None) };
    if !source.package_allowed(&package.url) || !is_sha256(&package.sha256) || package.size == 0 || package.size > MAX_PACKAGE {
        return Err(bad());
    }
    Ok(Some(Release { manifest, package, current: current.into() }))
}

pub(crate) async fn bounded_get(http: &reqwest::Client, url: &str, limit: usize) -> Result<Vec<u8>> {
    let too_large = || Error::msg(tr("Odpowiedź serwera aktualizacji jest zbyt duża", "The update server's response is too large"));
    let mut response = http.get(url).timeout(Duration::from_secs(15)).send().await?.error_for_status()?;
    if response.content_length().is_some_and(|n| n > limit as u64) {
        return Err(too_large());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > limit {
            return Err(too_large());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

/// `Ok(None)` when up to date; a network failure is an error.
pub async fn find(source: &Source, current: &str) -> Result<Option<Release>> {
    let http = source.client()?;
    let manifest = bounded_get(&http, &format!("{}latest.json", source.base), MAX_MANIFEST).await?;
    let signature = bounded_get(&http, &format!("{}latest.json.sig", source.base), MAX_SIGNATURE).await?;
    verify_signature(&manifest, &signature, &source.public_key)?;
    select(serde_json::from_slice(&manifest)?, source, current, &platform())
}

/// The files an update replaces.
#[derive(Debug, Clone)]
pub struct Targets {
    pub exe: PathBuf,
    pub engine: PathBuf,
}

impl Targets {
    pub fn current(ctx: &Ctx) -> Result<Self> {
        Ok(Self { exe: std::env::current_exe()?, engine: engine::root(ctx) })
    }

    fn launcher_dir(&self) -> Result<&Path> {
        self.exe.parent().ok_or_else(|| Error::msg(tr("Nieznany folder launchera", "Unknown launcher folder")))
    }
}

fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".patina-probe-{}", uuid::Uuid::new_v4().simple()));
    let created = fs::create_dir_all(dir).and_then(|_| fs::write(&probe, b"")).is_ok();
    let _ = fs::remove_file(&probe);
    created
}

/// Debug builds never replace themselves, and neither can a launcher in a read-only folder.
pub fn installable(targets: &Targets) -> bool {
    !cfg!(debug_assertions) && targets.launcher_dir().is_ok_and(writable) && writable(&targets.engine)
}

pub(crate) async fn download(http: &reqwest::Client, package: &Package, to: &Path, progress: &(dyn Fn(UpdateProgress) + Send + Sync)) -> Result<String> {
    let wrong_size = || Error::msg(tr("Rozmiar pobranego pliku się nie zgadza", "The downloaded file has the wrong size"));
    let mut response = http.get(&package.url).send().await?.error_for_status()?;
    if response.content_length().is_some_and(|n| n != package.size) {
        return Err(wrong_size());
    }
    let mut file = tokio::fs::File::create(to).await?;
    let mut hash = Sha256::new();
    let (mut done, mut reported) = (0, 0);
    while let Some(chunk) = response.chunk().await? {
        done += chunk.len() as u64;
        if done > package.size {
            return Err(wrong_size());
        }
        hash.update(&chunk);
        file.write_all(&chunk).await?;
        if done - reported >= package.size / 100 {
            reported = done;
            progress(UpdateProgress { stage: "download", done, total: package.size });
        }
    }
    file.flush().await?;
    if done != package.size {
        return Err(wrong_size());
    }
    Ok(hex::encode(hash.finalize()))
}

pub(crate) fn check_hash(package: &Package, got: &str) -> Result<()> {
    if !package.sha256.eq_ignore_ascii_case(got) {
        bail!("Suma SHA-256 pobranego pliku się nie zgadza", "The downloaded file's SHA-256 does not match");
    }
    Ok(())
}

/// Unpacks `launcher/` and `engine/` under `into`. Nothing can land elsewhere, or be a link.
fn extract(archive: &Path, into: &Path) -> Result<()> {
    let unsafe_entry = |name: &str| Error::msg(crate::tr!("Niebezpieczny wpis w paczce: {name}", "Unsafe entry in the package: {name}"));
    let mut zip = zip::ZipArchive::new(fs::File::open(archive)?)?;
    if zip.len() > MAX_ENTRIES {
        bail!("Paczka aktualizacji ma zbyt wiele plików", "The update package has too many files");
    }
    let mut budget = MAX_UNPACKED;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name()?.into_owned();
        let relative = entry.enclosed_name().ok_or_else(|| unsafe_entry(&name))?;
        let top = relative.components().next().and_then(|c| c.as_os_str().to_str());
        let normal = relative.components().all(|c| matches!(c, Component::Normal(_)));
        if entry.is_symlink() || !normal || !matches!(top, Some("launcher" | "engine")) {
            return Err(unsafe_entry(&name));
        }
        let path = into.join(&relative);
        if entry.is_dir() {
            fs::create_dir_all(&path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
        let copied = io::copy(&mut (&mut entry).take(budget + 1), &mut out)?;
        if copied > budget {
            bail!("Paczka aktualizacji jest zbyt duża po rozpakowaniu", "The update package is too large when unpacked");
        }
        budget -= copied;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode().map(|m| m & 0o777).filter(|m| *m != 0) {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(mode | 0o600))?;
        }
    }
    Ok(())
}

fn files_under(dir: &Path, skip: &[PathBuf], out: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if skip.contains(&path) {
            continue;
        }
        if path.is_dir() {
            files_under(&path, skip, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    if from.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        return Ok(());
    }
    fs::copy(from, to)?;
    fs::File::open(to)?.sync_all()
}

/// One replaced file or folder. The copy sits beside its destination first, so a
/// swap is two renames on one volume, whatever volume the data folder is on.
struct Swap {
    staged: PathBuf,
    dest: PathBuf,
    moved_aside: bool,
    placed: bool,
}

impl Swap {
    fn new(staged: PathBuf, dest: PathBuf) -> Self {
        Self { staged, dest, moved_aside: false, placed: false }
    }

    fn fresh(&self) -> PathBuf {
        with_suffix(&self.dest, NEW_SUFFIX)
    }

    fn old(&self) -> PathBuf {
        with_suffix(&self.dest, OLD_SUFFIX)
    }

    fn prepare(&self) -> io::Result<()> {
        if let Some(parent) = self.dest.parent() {
            fs::create_dir_all(parent)?;
        }
        remove_any(&self.fresh())?;
        remove_any(&self.old())?;
        copy_tree(&self.staged, &self.fresh())
    }

    fn commit(&mut self) -> io::Result<()> {
        if fs::symlink_metadata(&self.dest).is_ok() {
            fs::rename(&self.dest, self.old())?;
            self.moved_aside = true;
        }
        fs::rename(self.fresh(), &self.dest)?;
        self.placed = true;
        Ok(())
    }

    fn roll_back(&self) -> io::Result<()> {
        if self.placed {
            remove_any(&self.dest)?;
        }
        if self.moved_aside {
            fs::rename(self.old(), &self.dest)?;
        }
        remove_any(&self.fresh())
    }
}

/// Every swap for the unpacked package at `staged`, and the new launcher.
fn plan(staged: &Path, targets: &Targets) -> Result<(Vec<Swap>, PathBuf)> {
    let launcher = staged.join("launcher");
    let new_exe = launcher.join(format!("patina{}", std::env::consts::EXE_SUFFIX));
    if !new_exe.is_file() {
        bail!("Paczka aktualizacji nie zawiera launchera", "The update package has no launcher");
    }
    let mut swaps = Vec::new();
    let mut files = Vec::new();
    files_under(&launcher, std::slice::from_ref(&new_exe), &mut files)?;
    let launcher_dir = targets.launcher_dir()?;
    for file in files {
        let relative = file.strip_prefix(&launcher).expect("listed under launcher").to_path_buf();
        swaps.push(Swap::new(file, launcher_dir.join(relative)));
    }
    let engine = staged.join("engine");
    if engine.is_dir() {
        let whole: Vec<PathBuf> = WHOLE_DIRS.iter().map(|d| engine.join(d)).collect();
        let mut files = Vec::new();
        files_under(&engine, &whole, &mut files)?;
        files.extend(whole.into_iter().filter(|d| d.is_dir()));
        for file in files {
            let relative = file.strip_prefix(&engine).expect("listed under engine").to_path_buf();
            swaps.push(Swap::new(file, targets.engine.join(relative)));
        }
    }
    Ok((swaps, new_exe))
}

/// Puts the unpacked package in place. Files not in the package stay: the engine
/// folder also holds prepared game resources and logs.
fn apply(staged: &Path, targets: &Targets, replace_exe: impl FnOnce(&Path) -> io::Result<()>, progress: &(dyn Fn(UpdateProgress) + Send + Sync)) -> Result<()> {
    let (mut swaps, new_exe) = plan(staged, targets)?;
    let total = swaps.len() as u64 + 1;
    progress(UpdateProgress { stage: "install", done: 0, total });
    let mut outcome = Ok(());
    for (i, swap) in swaps.iter().enumerate() {
        outcome = swap.prepare();
        if outcome.is_err() {
            break;
        }
        progress(UpdateProgress { stage: "install", done: i as u64, total });
    }
    if outcome.is_ok() {
        for swap in &mut swaps {
            outcome = swap.commit();
            if outcome.is_err() {
                break;
            }
        }
    }
    // Last: nothing after this can fail, and it cannot be undone from the running program.
    let outcome = outcome.and_then(|()| replace_exe(&new_exe));
    if let Err(error) = outcome {
        for swap in swaps.iter().rev() {
            if let Err(e) = swap.roll_back() {
                log::error!("update rollback of {}: {e}", swap.dest.display());
            }
        }
        return Err(error.into());
    }
    for swap in &swaps {
        // A loaded DLL on Windows stays until the next start cleans it up.
        let _ = remove_any(&swap.old());
    }
    progress(UpdateProgress { stage: "install", done: total, total });
    Ok(())
}

fn staging(ctx: &Ctx) -> PathBuf {
    ctx.paths.root.join("update-staging")
}

/// Downloads, verifies and installs `release`. The launcher then only needs a restart.
pub async fn install(ctx: &Ctx, source: &Source, release: &Release, targets: Targets, progress: impl Fn(UpdateProgress) + Send + Sync + 'static) -> Result<()> {
    let work = staging(ctx).join(uuid::Uuid::new_v4().simple().to_string());
    fs::create_dir_all(&work)?;
    let result = download_and_apply(source, &release.package, &work, targets, progress).await;
    let _ = fs::remove_dir_all(&work);
    result
}

async fn download_and_apply(source: &Source, package: &Package, work: &Path, targets: Targets, progress: impl Fn(UpdateProgress) + Send + Sync + 'static) -> Result<()> {
    let archive = work.join("package.zip");
    let got = download(&source.client()?, package, &archive, &progress).await?;
    progress(UpdateProgress { stage: "verify", done: 0, total: 1 });
    check_hash(package, &got)?;
    let work = work.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let staged = work.join("staged");
        extract(&archive, &staged)?;
        progress(UpdateProgress { stage: "verify", done: 1, total: 1 });
        apply(&staged, &targets, |new| self_replace::self_replace(new), &progress)
    })
    .await
    .map_err(|e| Error::msg(e.to_string()))?
}

/// Whatever a crash left behind. A backup whose file is missing goes back in place.
fn sweep(dir: &Path, deep: bool) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(original) = name.strip_suffix(OLD_SUFFIX) {
            let original = dir.join(original);
            if original.exists() {
                let _ = remove_any(&path);
            } else {
                let _ = fs::rename(&path, original);
            }
        } else if name.ends_with(NEW_SUFFIX) {
            let _ = remove_any(&path);
        } else if deep && name != ".local" && path.is_dir() {
            sweep(&path, true);
        }
    }
}

/// Run once at start, before any update can begin.
pub fn clean_stale(ctx: &Ctx) {
    let _ = fs::remove_dir_all(staging(ctx));
    if let Ok(targets) = Targets::current(ctx) {
        if let Ok(dir) = targets.launcher_dir() {
            sweep(dir, false);
        }
        sweep(&targets.engine, true);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use std::sync::{Arc, Mutex};
    use zip::write::SimpleFileOptions;

    /// A real `tauri signer sign` signature of [`SIGNED_BY_TAURI`], made with a throwaway key.
    const TAURI_KEY: &str = "RWTvlNGtH20ANAPzEcYavCWasC9/FJ9FMTT+BQt7qbhg0P5S2+ls7xD4";
    const TAURI_SIGNATURE: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVUdmxOR3RIMjBBTkkrSXlHd3lVeUsyWmxOZ2RxMEtWbEVBa3VybmFudTdoaHVHa0tWeDZ3bHVYYlpmRDA1cjg0c3lDekw2TUhyR2JXVlFkRlhLcldXeHBLRkVFb2p3MXc4PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkzNTQ5NjQ2CWZpbGU6Zml4dHVyZS5qc29uCmxoWUlkc1V4R1FhUmFhS2V5dUNwYnJqVVR3Wm13SXROMHQ0LzVKNVkvcTVERmM3RUlGUkZmcFJweFF5TUllTFM3SVpuN3VEb0djeS9XdDhubDNWYUJRPT0K";
    const SIGNED_BY_TAURI: &str = "{\"version\":\"0.3.0\"}\n";

    pub(crate) struct Keys {
        pub(crate) public: String,
        secret: minisign::SecretKey,
    }

    pub(crate) fn keys() -> Keys {
        let pair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        Keys { public: pair.pk.to_base64(), secret: pair.sk }
    }

    impl Keys {
        pub(crate) fn sign(&self, data: &[u8]) -> Vec<u8> {
            let signature = minisign::sign(None, &self.secret, Cursor::new(data), Some("trusted"), Some("untrusted")).unwrap();
            base64::engine::general_purpose::STANDARD.encode(signature.to_string()).into_bytes()
        }
    }

    fn manifest(version: &str, url: &str, sha256: &str, size: u64) -> Vec<u8> {
        let mut platforms = serde_json::Map::new();
        platforms.insert(platform(), serde_json::json!({ "url": url, "sha256": sha256, "size": size }));
        serde_json::to_vec(&serde_json::json!({ "version": version, "critical": false, "date": "2026-11-01", "notes": { "pl": "a", "en": "b" }, "platforms": platforms })).unwrap()
    }

    fn parsed(bytes: &[u8]) -> Manifest {
        serde_json::from_slice(bytes).unwrap()
    }

    pub(crate) fn sha(data: &[u8]) -> String {
        hex::encode(Sha256::digest(data))
    }

    fn temp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("patina-update-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    fn zip_of(files: &[(&str, &str)]) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, body) in files {
            zip.start_file(*name, SimpleFileOptions::default().unix_permissions(0o755)).unwrap();
            zip.write_all(body.as_bytes()).unwrap();
        }
        zip.finish().unwrap().into_inner()
    }

    /// Answers each `(path, body)` once, in order.
    pub(crate) fn answer(listener: std::net::TcpListener, responses: Vec<(String, Vec<u8>)>) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            for (path, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                let size = stream.read(&mut request).unwrap();
                assert!(String::from_utf8_lossy(&request[..size]).starts_with(&format!("GET /{path} ")));
                write!(stream, "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n", body.len()).unwrap();
                stream.write_all(&body).unwrap();
            }
        })
    }

    fn serve(responses: Vec<(String, Vec<u8>)>) -> (String, std::thread::JoinHandle<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        (format!("http://{}/", listener.local_addr().unwrap()), answer(listener, responses))
    }

    fn no_progress(_: UpdateProgress) {}

    #[test]
    fn the_embedded_key_is_valid() {
        assert!(minisign_verify::PublicKey::from_base64(PUBLIC_KEY).is_ok());
        assert_eq!(Source::official().base, UPDATES);
        assert!(Source::official().strict);
    }

    #[test]
    fn accepts_the_format_tauri_signer_writes() {
        verify_signature(SIGNED_BY_TAURI.as_bytes(), TAURI_SIGNATURE.as_bytes(), TAURI_KEY).unwrap();
        assert!(verify_signature(b"{\"version\":\"9.9.9\"}\n", TAURI_SIGNATURE.as_bytes(), TAURI_KEY).is_err());
    }

    #[test]
    fn signature_must_match_key_and_exact_bytes() {
        let keys = keys();
        let data = manifest("9.0.0", "https://x", &sha(b"x"), 1);
        let signature = keys.sign(&data);
        verify_signature(&data, &signature, &keys.public).unwrap();
        let mut tampered = data.clone();
        tampered[12] ^= 1;
        assert!(verify_signature(&tampered, &signature, &keys.public).is_err());
        assert!(verify_signature(&data, &signature, &self::keys().public).is_err());
        assert!(verify_signature(&data, &self::keys().sign(&data), &keys.public).is_err());
        assert!(verify_signature(&data, b"not base64 at all!", &keys.public).is_err());
        assert!(verify_signature(&data, b"", &keys.public).is_err());
    }

    #[test]
    fn only_strictly_newer_versions_are_offered() {
        let source = Source::official();
        let url = format!("{RELEASES}v0.3.0/patina.zip");
        let hash = sha(b"x");
        let offer = |version: &str, current: &str| select(parsed(&manifest(version, &url, &hash, 1)), &source, current, &platform()).unwrap();
        assert!(offer("0.3.0", "0.2.0").is_some());
        assert!(offer("0.10.0", "0.9.0").is_some());
        assert!(offer("0.3.0", "0.3.0").is_none());
        assert!(offer("0.2.9", "0.3.0").is_none());
        assert!(select(parsed(&manifest("not-a-version", &url, &hash, 1)), &source, "0.2.0", &platform()).is_err());
    }

    #[test]
    fn a_missing_platform_means_no_update() {
        let bytes = manifest("9.0.0", &format!("{RELEASES}v9/p.zip"), &sha(b"x"), 1);
        assert!(select(parsed(&bytes), &Source::official(), "0.2.0", "plan9-mips").unwrap().is_none());
    }

    #[test]
    fn packages_must_come_from_the_releases_of_the_update_repository() {
        let source = Source::official();
        let hash = sha(b"x");
        let bad = |url: &str| select(parsed(&manifest("9.0.0", url, &hash, 1)), &source, "0.2.0", &platform()).is_err();
        assert!(!bad(&format!("{RELEASES}v9/p.zip")));
        for url in ["https://example.com/p.zip", "http://github.com/project-patina/patina-updates/releases/download/v9/p.zip", "https://github.com/project-patina/other/releases/download/v9/p.zip",
            "https://github.com/project-patina/patina-updates/releases/download/../../../other/p.zip", "https://github.com.evil.io/project-patina/patina-updates/releases/download/v9/p.zip"] {
            assert!(bad(url), "{url}");
        }
        assert!(select(parsed(&manifest("9.0.0", &format!("{RELEASES}p.zip"), "abc", 1)), &source, "0.2.0", &platform()).is_err());
        assert!(select(parsed(&manifest("9.0.0", &format!("{RELEASES}p.zip"), &hash, MAX_PACKAGE + 1)), &source, "0.2.0", &platform()).is_err());
    }

    #[test]
    fn redirects_stay_on_github_download_hosts() {
        let source = Source::official();
        let allowed = |url: &str| source.redirect_allowed(&Url::parse(url).unwrap());
        assert!(allowed("https://release-assets.githubusercontent.com/x"));
        assert!(allowed("https://objects.githubusercontent.com/x"));
        assert!(!allowed("http://objects.githubusercontent.com/x"));
        assert!(!allowed("https://evilgithubusercontent.com/x"));
        assert!(!allowed("https://example.com/x"));
        assert!(!Source::testing("http://127.0.0.1:1/", "").redirect_allowed(&Url::parse("http://127.0.0.1:2/x").unwrap()));
    }

    #[tokio::test]
    async fn finds_downloads_and_unpacks_a_signed_release() {
        let keys = keys();
        let package = zip_of(&[("launcher/patina", "new"), ("engine/patina-client", "client")]);
        // The manifest names the server's own address, so the port comes first.
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}/", listener.local_addr().unwrap());
        let bytes = manifest("9.0.0", &format!("{base}p.zip"), &sha(&package), package.len() as u64);
        let server = answer(listener, vec![("latest.json".into(), bytes.clone()), ("latest.json.sig".into(), keys.sign(&bytes)), ("p.zip".into(), package.clone())]);
        let source = Source::testing(&base, &keys.public);
        let release = find(&source, "0.2.0").await.unwrap().unwrap();
        let info = release.info(false);
        assert_eq!((info.version.as_str(), info.current.as_str(), info.size_bytes), ("9.0.0", "0.2.0", package.len() as u64));
        let dir = temp();
        let seen = Arc::new(Mutex::new(0));
        let counter = seen.clone();
        let got = download(&source.client().unwrap(), &release.package, &dir.join("p.zip"), &move |_| *counter.lock().unwrap() += 1).await.unwrap();
        check_hash(&release.package, &got).unwrap();
        assert!(*seen.lock().unwrap() > 0);
        extract(&dir.join("p.zip"), &dir.join("staged")).unwrap();
        assert_eq!(read(&dir.join("staged/engine/patina-client")), "client");
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn a_manifest_failing_its_signature_is_never_parsed() {
        let keys = keys();
        let (base, server) = serve(vec![("latest.json".into(), b"{ not json".to_vec()), ("latest.json.sig".into(), b"bad".to_vec())]);
        let error = find(&Source::testing(&base, &keys.public), "0.2.0").await.unwrap_err();
        assert!(!matches!(error, Error::Json(_)), "{error}");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn a_wrong_hash_or_size_is_rejected() {
        let dir = temp();
        let (base, server) = serve(vec![("p.zip".into(), b"package".to_vec()), ("p.zip".into(), b"package".to_vec()), ("p.zip".into(), vec![0; 2 * MAX_MANIFEST])]);
        let source = Source::testing(&base, "");
        let http = source.client().unwrap();
        let package = |sha256: String, size| Package { url: format!("{base}p.zip"), sha256, size };
        let right = package(sha(b"package"), 7);
        let got = download(&http, &right, &dir.join("a"), &no_progress).await.unwrap();
        assert!(check_hash(&right, &got).is_ok());
        assert!(check_hash(&package(sha(b"other"), 7), &got).is_err());
        assert!(download(&http, &package(sha(b"x"), 6), &dir.join("b"), &no_progress).await.is_err());
        assert!(bounded_get(&http, &format!("{base}p.zip"), MAX_MANIFEST).await.is_err());
        server.join().unwrap();
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn path_traversal_links_and_foreign_folders_are_rejected() {
        let dir = temp();
        let attempt = |name: &str, bytes: Vec<u8>| {
            fs::write(dir.join(name), bytes).unwrap();
            extract(&dir.join(name), &dir.join(format!("{name}-out")))
        };
        for (i, name) in ["../evil", "launcher/../../evil", "/etc/evil", "other/file", "evil"].iter().enumerate() {
            assert!(attempt(&format!("bad{i}"), zip_of(&[(name, "x")])).is_err(), "{name}");
        }
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.add_symlink("launcher/link", "/etc/passwd", SimpleFileOptions::default()).unwrap();
        assert!(attempt("link", zip.finish().unwrap().into_inner()).is_err());
        assert!(!dir.parent().unwrap().join("evil").exists());
        assert!(attempt("good", zip_of(&[("launcher/patina", "x")])).is_ok());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(dir.join("good-out/launcher/patina")).unwrap().permissions().mode() & 0o777, 0o755);
        }
        fs::remove_dir_all(dir).unwrap();
    }

    struct Layout {
        dir: PathBuf,
        staged: PathBuf,
        targets: Targets,
    }

    fn layout() -> Layout {
        let dir = temp();
        let staged = dir.join("staged");
        let targets = Targets { exe: dir.join("app/launcher-binary"), engine: dir.join("engine") };
        write(&targets.exe, "old exe");
        write(&dir.join("app/WebView2Loader.dll"), "old dll");
        write(&targets.engine.join("patina-client"), "old client");
        write(&targets.engine.join("research-lock.json"), "old lock");
        write(&targets.engine.join(".local/patina-client/26.3/amethyst-ready"), "stamp");
        write(&targets.engine.join("logs/latest.log"), "log");
        write(&targets.engine.join("mods/patina/src/test/java/dev/Old.java"), "old java");
        write(&targets.engine.join("mods/patina/README"), "kept");
        write(&targets.engine.join("exporters/server/Old.class"), "old class");
        write(&staged.join("launcher/patina"), "new exe");
        write(&staged.join("launcher/WebView2Loader.dll"), "new dll");
        write(&staged.join("engine/patina-client"), "new client");
        write(&staged.join("engine/research-lock.json"), "new lock");
        write(&staged.join("engine/mods/patina/src/test/java/dev/New.java"), "new java");
        write(&staged.join("engine/exporters/server/New.class"), "new class");
        Layout { dir, staged, targets }
    }

    #[test]
    fn apply_replaces_packaged_files_and_keeps_everything_else() {
        let l = layout();
        let exe = Mutex::new(None);
        apply(&l.staged, &l.targets, |new| { *exe.lock().unwrap() = Some(read(new)); Ok(()) }, &no_progress).unwrap();
        assert_eq!(exe.lock().unwrap().as_deref(), Some("new exe"));
        let engine = &l.targets.engine;
        assert_eq!(read(&l.dir.join("app/WebView2Loader.dll")), "new dll");
        assert_eq!(read(&engine.join("patina-client")), "new client");
        assert_eq!(read(&engine.join("research-lock.json")), "new lock");
        // Never in a package, so never deleted.
        assert_eq!(read(&engine.join(".local/patina-client/26.3/amethyst-ready")), "stamp");
        assert_eq!(read(&engine.join("logs/latest.log")), "log");
        assert_eq!(read(&engine.join("mods/patina/README")), "kept");
        // The whole folders are replaced, not merged.
        assert_eq!(read(&engine.join("mods/patina/src/test/java/dev/New.java")), "new java");
        assert!(!engine.join("mods/patina/src/test/java/dev/Old.java").exists());
        assert!(!engine.join("exporters/server/Old.class").exists());
        let mut left = Vec::new();
        files_under(&l.dir.join("app"), &[], &mut left).unwrap();
        files_under(engine, &[], &mut left).unwrap();
        assert!(!left.iter().any(|p| p.to_string_lossy().contains(".patina-")), "{left:?}");
        fs::remove_dir_all(l.dir).unwrap();
    }

    fn assert_untouched(l: &Layout) {
        let engine = &l.targets.engine;
        assert_eq!(read(&l.targets.exe), "old exe");
        assert_eq!(read(&l.dir.join("app/WebView2Loader.dll")), "old dll");
        assert_eq!(read(&engine.join("patina-client")), "old client");
        assert_eq!(read(&engine.join("research-lock.json")), "old lock");
        assert_eq!(read(&engine.join("mods/patina/src/test/java/dev/Old.java")), "old java");
        assert_eq!(read(&engine.join("exporters/server/Old.class")), "old class");
        assert!(!engine.join("mods/patina/src/test/java/dev/New.java").exists());
        let mut left = Vec::new();
        files_under(&l.dir.join("app"), &[], &mut left).unwrap();
        files_under(engine, &[], &mut left).unwrap();
        assert!(!left.iter().any(|p| p.to_string_lossy().contains(".patina-")), "{left:?}");
    }

    #[test]
    fn a_failed_launcher_swap_rolls_everything_back() {
        let l = layout();
        write(&l.staged.join("engine/brand-new-file"), "new");
        let failed = apply(&l.staged, &l.targets, |_| Err(io::Error::other("locked")), &no_progress);
        assert!(failed.is_err());
        assert_untouched(&l);
        assert!(!l.targets.engine.join("brand-new-file").exists());
        fs::remove_dir_all(l.dir).unwrap();
    }

    #[test]
    fn a_failed_copy_leaves_the_old_version_in_place() {
        let l = layout();
        // A file where the package needs a folder.
        write(&l.targets.engine.join("blocked"), "a file");
        write(&l.staged.join("engine/blocked/inner"), "new");
        let failed = apply(&l.staged, &l.targets, |_| panic!("must not get this far"), &no_progress);
        assert!(failed.is_err());
        assert_untouched(&l);
        assert_eq!(read(&l.targets.engine.join("blocked")), "a file");
        fs::remove_dir_all(l.dir).unwrap();
    }

    #[test]
    fn a_package_without_a_launcher_changes_nothing() {
        let l = layout();
        fs::remove_file(l.staged.join("launcher/patina")).unwrap();
        assert!(apply(&l.staged, &l.targets, |_| Ok(()), &no_progress).is_err());
        assert_untouched(&l);
        fs::remove_dir_all(l.dir).unwrap();
    }

    #[test]
    fn sweeping_restores_lost_files_and_removes_leftovers() {
        let dir = temp();
        write(&dir.join("kept"), "new");
        write(&dir.join("kept.patina-old"), "old");
        write(&dir.join("lost.patina-old"), "old");
        write(&dir.join("half.patina-new"), "partial");
        write(&dir.join("nested/x.patina-new"), "partial");
        write(&dir.join(".local/resources.patina-new"), "not ours to scan");
        sweep(&dir, true);
        assert_eq!(read(&dir.join("kept")), "new");
        assert_eq!(read(&dir.join("lost")), "old");
        assert!(!dir.join("kept.patina-old").exists());
        assert!(!dir.join("half.patina-new").exists());
        assert!(!dir.join("nested/x.patina-new").exists());
        assert!(dir.join(".local/resources.patina-new").exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
