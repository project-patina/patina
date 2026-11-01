//! Java runtimes. By default we use the same runtimes the official launcher
//! ships (piston-meta "java-runtime" components), so the player never has to
//! install Java themselves.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::download::{download_all, DlItem};
use crate::progress::Reporter;
use crate::{bail, http, Ctx, Result};

const RUNTIMES_URL: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

fn platform_key() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86") => "linux-i386",
        ("linux", _) => "linux",
        ("macos", "aarch64") => "mac-os-arm64",
        ("macos", _) => "mac-os",
        ("windows", "aarch64") => "windows-arm64",
        ("windows", "x86") => "windows-x86",
        _ => "windows-x64",
    }
}

#[derive(Deserialize)]
struct RuntimeEntry {
    manifest: ManifestRef,
    version: RuntimeVersion,
}

#[derive(Deserialize)]
struct ManifestRef {
    url: String,
}

#[derive(Deserialize)]
struct RuntimeVersion {
    name: String,
}

#[derive(Deserialize)]
struct RuntimeManifest {
    files: HashMap<String, RuntimeFile>,
}

#[derive(Deserialize)]
struct RuntimeFile {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    executable: bool,
    #[serde(default)]
    downloads: Option<RuntimeDownloads>,
    #[serde(default)]
    target: Option<String>,
}

#[derive(Deserialize)]
struct RuntimeDownloads {
    raw: RawDownload,
}

#[derive(Deserialize)]
struct RawDownload {
    url: String,
    sha1: String,
    size: u64,
}

fn java_bin(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("bin").join("javaw.exe")
    } else if cfg!(target_os = "macos") {
        home.join("jre.bundle/Contents/Home/bin/java")
    } else {
        home.join("bin").join("java")
    }
}

/// Make sure Mojang's runtime `component` (e.g. `java-runtime-delta`) is
/// installed and return the path to its `java` binary.
pub async fn ensure_runtime(ctx: &Ctx, component: &str, reporter: &Reporter, concurrency: usize, cancel: &CancellationToken) -> Result<PathBuf> {
    let home = ctx.paths.runtimes().join(component);
    let marker = home.join(".amethyst-complete");
    let bin = java_bin(&home);
    if marker.exists() && bin.exists() {
        return Ok(bin);
    }
    reporter.report("java", 0, 1, format!("Pobieranie Javy ({component})…"));
    let all: HashMap<String, HashMap<String, Vec<RuntimeEntry>>> = http::get_json(&ctx.http, RUNTIMES_URL).await?;
    let Some(entry) = all.get(platform_key()).and_then(|p| p.get(component)).and_then(|v| v.first()) else {
        bail!("Mojang nie udostępnia Javy {component} dla tej platformy — ustaw własną ścieżkę Javy w ustawieniach")
    };
    log::info!("installing java runtime {component} {}", entry.version.name);
    let manifest: RuntimeManifest = http::get_json(&ctx.http, &entry.manifest.url).await?;

    let mut items = Vec::new();
    let mut links = Vec::new();
    for (rel, f) in &manifest.files {
        let path = home.join(rel);
        match f.kind.as_str() {
            "directory" => std::fs::create_dir_all(&path)?,
            "file" => {
                let Some(d) = &f.downloads else { continue };
                let mut it = DlItem::new(&d.raw.url, path).sha1(Some(d.raw.sha1.clone())).size(Some(d.raw.size));
                it.executable = f.executable;
                items.push(it);
            }
            "link" => {
                if let Some(t) = &f.target {
                    links.push((path, t.clone()));
                }
            }
            _ => {}
        }
    }
    download_all(&ctx.http, items, concurrency, reporter, "java", "Pobieranie Javy", cancel).await?;
    #[cfg(unix)]
    for (path, target) in links {
        let _ = std::fs::remove_file(&path);
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::os::unix::fs::symlink(target, &path)?;
    }
    if !bin.exists() {
        bail!("Pobrana Java nie zawiera pliku {}", bin.display());
    }
    std::fs::write(&marker, &entry.version.name)?;
    Ok(bin)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInstall {
    pub path: String,
    pub version: String,
    pub major: u32,
    /// `bundled` for runtimes managed by the launcher, `system` otherwise.
    pub source: String,
}

/// Run `java -version` and pull out the version string.
pub async fn probe(path: &Path) -> Option<(String, u32)> {
    let out = tokio::process::Command::new(path).arg("-version").output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stderr).to_string() + &String::from_utf8_lossy(&out.stdout);
    let version = text.lines().find_map(|l| l.split('"').nth(1).map(str::to_string))?;
    Some((version.clone(), major_of(&version)))
}

pub fn major_of(version: &str) -> u32 {
    let mut parts = version.split(['.', '-', '_', '+']);
    let first: u32 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    if first == 1 {
        parts.next().and_then(|p| p.parse().ok()).unwrap_or(0)
    } else {
        first
    }
}

/// Java installations found on this machine plus runtimes we downloaded.
pub async fn detect(ctx: &Ctx) -> Vec<JavaInstall> {
    let mut candidates: Vec<(PathBuf, &str)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(ctx.paths.runtimes()) {
        for e in rd.flatten() {
            candidates.push((java_bin(&e.path()), "bundled"));
        }
    }
    for root in ["/usr/lib/jvm", "/usr/lib64/jvm", "/opt", "/Library/Java/JavaVirtualMachines"] {
        if let Ok(rd) = std::fs::read_dir(root) {
            for e in rd.flatten() {
                let p = e.path();
                candidates.push((p.join("bin/java"), "system"));
                candidates.push((p.join("Contents/Home/bin/java"), "system"));
            }
        }
    }
    if let Ok(home) = std::env::var("JAVA_HOME") {
        candidates.push((PathBuf::from(home).join("bin/java"), "system"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for (p, source) in candidates {
        let Ok(real) = std::fs::canonicalize(&p) else { continue };
        if !seen.insert(real) {
            continue;
        }
        if let Some((version, major)) = probe(&p).await {
            out.push(JavaInstall { path: p.to_string_lossy().into(), version, major, source: source.into() });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn majors() {
        assert_eq!(super::major_of("1.8.0_412"), 8);
        assert_eq!(super::major_of("21.0.3"), 21);
        assert_eq!(super::major_of("17"), 17);
    }
}
