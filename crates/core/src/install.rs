//! Resolve an instance to a complete, downloaded game: version JSON (with the
//! loader folded in), client jar, libraries, natives, assets and Java.

use std::io::Read;
use std::path::PathBuf;

use tokio_util::sync::CancellationToken;

use crate::download::{download_all, DlItem};
use crate::instance::Instance;
use crate::loaders;
use crate::mojang::{self, AssetIndex, Features, VersionJson};
use crate::progress::Reporter;
use crate::settings::Settings;
use crate::{bail, http, java, Ctx, Error, Result};

const LIBRARIES_URL: &str = "https://libraries.minecraft.net/";

/// Everything `launch` needs, all present on disk.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub version: VersionJson,
    pub vanilla_id: String,
    pub java: PathBuf,
    pub classpath: Vec<PathBuf>,
    pub natives_dir: PathBuf,
    pub assets_root: PathBuf,
    pub assets_index: String,
    /// For pre-1.6 versions the game reads assets from a flat folder.
    pub legacy_assets_dir: Option<PathBuf>,
    pub logging_arg: Option<String>,
}

async fn vanilla_json(ctx: &Ctx, mc: &str) -> Result<VersionJson> {
    let path = ctx.paths.version_json(mc);
    let manifest = mojang::manifest(ctx).await;
    let entry = manifest.as_ref().ok().and_then(|m| m.versions.iter().find(|v| v.id == mc).cloned());
    if let Ok(bytes) = std::fs::read(&path) {
        // Re-download only if Mojang changed the file (sha1 mismatch).
        let fresh = match &entry.as_ref().and_then(|e| e.sha1.clone()) {
            Some(sha) => crate::download::sha1_file(&path).map(|h| &h == sha).unwrap_or(false),
            None => true,
        };
        if fresh {
            return Ok(serde_json::from_slice(&bytes)?);
        }
    }
    let Some(entry) = entry else {
        manifest?;
        bail!("Nie znaleziono wersji Minecraft {mc}", "Minecraft version {mc} not found");
    };
    let bytes = http::get_bytes(&ctx.http, &entry.url).await?;
    let v: VersionJson = serde_json::from_slice(&bytes)?;
    crate::paths::write_atomic(&path, &bytes)?;
    Ok(v)
}

/// Vanilla JSON with the instance's loader profile merged on top.
pub async fn resolve_version(ctx: &Ctx, inst: &mut Instance) -> Result<VersionJson> {
    let vanilla = vanilla_json(ctx, &inst.mc_version).await?;
    if inst.loader.is_plain() {
        return Ok(vanilla);
    }
    let lv = match &inst.loader_version {
        Some(v) => v.clone(),
        None => {
            let v = loaders::recommended(ctx, inst.loader, &inst.mc_version).await?;
            inst.loader_version = Some(v.clone());
            inst.save(ctx)?;
            v
        }
    };
    let profile = loaders::profile(ctx, inst.loader, &inst.mc_version, &lv).await?;
    Ok(mojang::merge(vanilla, profile))
}

fn natives_classifier(lib: &mojang::Library) -> Option<String> {
    let arch_bits = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
    lib.natives.as_ref()?.get(mojang::os_name()).map(|c| c.replace("${arch}", arch_bits))
}

pub async fn prepare(ctx: &Ctx, inst: &mut Instance, settings: &Settings, reporter: &Reporter, cancel: &CancellationToken) -> Result<Prepared> {
    let conc = settings.max_concurrent_downloads as usize;
    reporter.report("version", 0, 1, crate::tr!("Przygotowywanie {} {}…", "Preparing {} {}…", inst.loader.label(), inst.mc_version));
    let version = resolve_version(ctx, inst).await?;
    let vanilla_id = inst.mc_version.clone();
    let features = Features::default();

    // --- client jar + libraries -------------------------------------------
    let mut items = Vec::new();
    let client_jar = ctx.paths.client_jar(&vanilla_id);
    match version.downloads.as_ref().and_then(|d| d.client.as_ref()) {
        Some(c) => items.push(DlItem::new(&c.url, &client_jar).sha1(c.sha1.clone()).size(c.size)),
        None => bail!("Wersja {vanilla_id} nie ma pliku klienta", "Version {vanilla_id} has no client file"),
    }

    let libs_root = ctx.paths.libraries();
    let mut classpath = Vec::new();
    let mut natives_jars = Vec::new();
    for lib in &version.libraries {
        if !mojang::rules_allow(lib.rules.as_deref(), &features) {
            continue;
        }
        let artifact = lib.downloads.as_ref().and_then(|d| d.artifact.as_ref());
        if let Some(a) = artifact {
            let rel = a.path.clone().or_else(|| mojang::maven_path(&lib.name)).ok_or_else(|| Error::msg(crate::tr!("Zła biblioteka {}", "Bad library {}", lib.name)))?;
            let path = libs_root.join(rel);
            if !a.url.is_empty() {
                items.push(DlItem::new(&a.url, &path).sha1(a.sha1.clone()).size(a.size));
            }
            classpath.push(path);
        } else if lib.downloads.is_none() || lib.url.is_some() {
            // Plain maven library (Fabric/Quilt).
            let Some(rel) = mojang::maven_path(&lib.name) else { continue };
            let base = lib.url.as_deref().unwrap_or(LIBRARIES_URL);
            let base = if base.ends_with('/') { base.to_string() } else { format!("{base}/") };
            let path = libs_root.join(&rel);
            items.push(DlItem::new(format!("{base}{rel}"), &path).sha1(lib.sha1.clone()).size(lib.size));
            classpath.push(path);
        }
        if let Some(cl) = natives_classifier(lib) {
            if let Some(a) = lib.downloads.as_ref().and_then(|d| d.classifiers.as_ref()).and_then(|c| c.get(&cl)) {
                let rel = a.path.clone().or_else(|| mojang::maven_path(&format!("{}:{cl}", lib.name))).unwrap_or_default();
                let path = libs_root.join(rel);
                items.push(DlItem::new(&a.url, &path).sha1(a.sha1.clone()).size(a.size));
                natives_jars.push((path, lib.extract.as_ref().map(|e| e.exclude.clone()).unwrap_or_default()));
            }
        }
    }
    classpath.push(client_jar);

    // --- logging config ------------------------------------------------------
    let mut logging_arg = None;
    if let Some(lc) = version.logging.as_ref().and_then(|l| l.client.as_ref()) {
        let path = ctx.paths.assets().join("log_configs").join(&lc.file.id);
        items.push(DlItem::new(&lc.file.url, &path).sha1(lc.file.sha1.clone()).size(lc.file.size));
        logging_arg = Some(lc.argument.replace("${path}", &path.to_string_lossy()));
    }

    download_all(&ctx.http, items, conc, reporter, "libraries", crate::lang::tr("Pobieranie bibliotek", "Downloading libraries"), cancel).await?;

    // --- natives (pre-1.19 style: jars that must be unpacked) ---------------
    let natives_dir = ctx.paths.natives(&version.id);
    if !natives_jars.is_empty() {
        let dir = natives_dir.clone();
        tokio::task::spawn_blocking(move || extract_natives(&natives_jars, &dir)).await.map_err(|e| Error::msg(e.to_string()))??;
    } else {
        std::fs::create_dir_all(&natives_dir)?;
    }

    // --- assets ---------------------------------------------------------------
    let Some(ai) = version.asset_index.clone() else { bail!("Wersja {vanilla_id} nie ma indeksu zasobów", "Version {vanilla_id} has no asset index") };
    let assets_root = ctx.paths.assets();
    let index_path = assets_root.join("indexes").join(format!("{}.json", ai.id));
    download_all(&ctx.http, vec![DlItem::new(&ai.url, &index_path).sha1(ai.sha1.clone()).size(ai.size)], 1, reporter, "assets", crate::lang::tr("Indeks zasobów", "Asset index"), cancel).await?;
    let index: AssetIndex = serde_json::from_slice(&std::fs::read(&index_path)?)?;
    let objects: Vec<DlItem> = index
        .objects
        .values()
        .map(|o| {
            let sub = &o.hash[..2];
            let mut it = DlItem::new(format!("{}/{sub}/{}", mojang::RESOURCES_URL, o.hash), assets_root.join("objects").join(sub).join(&o.hash))
                .sha1(Some(o.hash.clone()))
                .size(Some(o.size));
            it.verify_existing = false;
            it
        })
        .collect();
    download_all(&ctx.http, objects, conc, reporter, "assets", crate::lang::tr("Pobieranie zasobów gry", "Downloading game assets"), cancel).await?;

    let legacy_assets_dir = if index.map_to_resources || index.is_virtual {
        let target = if index.map_to_resources { ctx.paths.game_dir(&inst.id).join("resources") } else { assets_root.join("virtual").join(&ai.id) };
        let t = target.clone();
        let root = assets_root.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            for (name, o) in &index.objects {
                let dst = t.join(name);
                if dst.exists() {
                    continue;
                }
                if let Some(p) = dst.parent() {
                    std::fs::create_dir_all(p)?;
                }
                std::fs::copy(root.join("objects").join(&o.hash[..2]).join(&o.hash), dst)?;
            }
            Ok(())
        })
        .await
        .map_err(|e| Error::msg(e.to_string()))??;
        Some(target)
    } else {
        None
    };

    // --- java -------------------------------------------------------------------
    let custom = inst.overrides.java_path.clone().filter(|p| !p.trim().is_empty()).unwrap_or_else(|| settings.java_path.clone());
    let java = if !custom.trim().is_empty() {
        let p = PathBuf::from(custom.trim());
        if !p.exists() {
            bail!("Nie znaleziono Javy: {}", "Java not found: {}", p.display());
        }
        p
    } else {
        let component = version.java_version.as_ref().map(|j| j.component.clone()).unwrap_or_else(|| "jre-legacy".into());
        java::ensure_runtime(ctx, &component, reporter, conc, cancel).await?
    };

    Ok(Prepared {
        assets_index: ai.id,
        version,
        vanilla_id,
        java,
        classpath,
        natives_dir,
        assets_root,
        legacy_assets_dir,
        logging_arg,
    })
}

fn extract_natives(jars: &[(PathBuf, Vec<String>)], dir: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    for (jar, exclude) in jars {
        let mut zip = zip::ZipArchive::new(std::fs::File::open(jar)?)?;
        for i in 0..zip.len() {
            let mut f = zip.by_index(i)?;
            let Some(name) = f.enclosed_name() else { continue };
            let name_str = name.to_string_lossy().replace('\\', "/");
            if f.is_dir() || name_str.starts_with("META-INF/") || exclude.iter().any(|e| name_str.starts_with(e.as_str())) {
                continue;
            }
            let out = dir.join(&name);
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut buf = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buf)?;
            std::fs::write(out, buf)?;
        }
    }
    Ok(())
}
