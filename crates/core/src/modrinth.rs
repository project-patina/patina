//! Modrinth API: search, project pages, versions, installing with
//! dependencies, identifying jars by hash and checking for updates.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::content::{self, ContentKind, ContentMeta};
use crate::download::{download_all, DlItem};
use crate::instance::Instance;
use crate::loaders::Loader;
use crate::progress::Reporter;
use crate::{bail, Ctx, Error, Result};

const API: &str = "https://api.modrinth.com/v2";

/// (slug, display name). Client-side only, safe on any server.
pub const PERFORMANCE_MODS: &[(&str, &str)] = &[
    ("fabric-api", "Fabric API"),
    ("sodium", "Sodium"),
    ("lithium", "Lithium"),
    ("ferrite-core", "FerriteCore"),
    ("immediatelyfast", "ImmediatelyFast"),
    ("entityculling", "Entity Culling"),
    ("modmenu", "Mod Menu"),
];

// ---------------------------------------------------------------------------
// API types (field names as Modrinth sends them, camelCase towards the UI)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SearchHit {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub follows: u64,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub display_categories: Vec<String>,
    pub project_type: String,
    #[serde(default)]
    pub date_modified: String,
    #[serde(default)]
    pub featured_gallery: Option<String>,
    #[serde(default)]
    pub color: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SearchResult {
    pub hits: Vec<SearchHit>,
    pub offset: u32,
    pub limit: u32,
    pub total_hits: u32,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    #[serde(default)]
    pub query: String,
    /// `mod`, `resourcepack` or `shader`.
    pub kind: ContentKind,
    /// Game version filter (usually the target instance's).
    #[serde(default)]
    pub game_version: Option<String>,
    #[serde(default)]
    pub loader: Option<Loader>,
    #[serde(default)]
    pub categories: Vec<String>,
    /// `relevance`, `downloads`, `follows`, `newest`, `updated`.
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GalleryImage {
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub downloads: u64,
    #[serde(default)]
    pub followers: u64,
    #[serde(default)]
    pub categories: Vec<String>,
    pub project_type: String,
    #[serde(default)]
    pub gallery: Vec<GalleryImage>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub issues_url: Option<String>,
    #[serde(default)]
    pub wiki_url: Option<String>,
    #[serde(default)]
    pub discord_url: Option<String>,
    #[serde(default)]
    pub license: Option<License>,
    #[serde(default)]
    pub client_side: Option<String>,
    #[serde(default)]
    pub server_side: Option<String>,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub primary: bool,
    pub hashes: Hashes,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashes {
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Dependency {
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub version_id: Option<String>,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Version {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    /// `release`, `beta`, `alpha`.
    pub version_type: String,
    #[serde(default)]
    pub game_versions: Vec<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    pub date_published: String,
    #[serde(default)]
    pub downloads: u64,
    pub files: Vec<VersionFile>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub changelog: Option<String>,
}

impl Version {
    pub fn primary_file(&self) -> Option<&VersionFile> {
        self.files.iter().find(|f| f.primary).or(self.files.first())
    }
}

// ---------------------------------------------------------------------------
// Helpers

fn json_list<T: Serialize>(v: &[T]) -> String {
    serde_json::to_string(v).unwrap()
}

/// Loader names Modrinth should accept for an instance. Quilt runs Fabric mods.
pub fn loader_names(kind: ContentKind, loader: Loader) -> Vec<&'static str> {
    match kind {
        ContentKind::Mod => match loader {
            Loader::Vanilla | Loader::Rust => vec![],
            Loader::Fabric => vec!["fabric"],
            Loader::Quilt => vec!["quilt", "fabric"],
        },
        // Packs and shaders are not tied to the mod loader.
        _ => vec![],
    }
}

async fn get<T: serde::de::DeserializeOwned>(ctx: &Ctx, url: &str, query: &[(&str, String)]) -> Result<Option<T>> {
    let r = ctx.http.get(url).query(query).send().await?;
    if r.status().as_u16() == 404 {
        return Ok(None);
    }
    if r.status().as_u16() == 429 {
        bail!("Modrinth ogranicza liczbę zapytań — spróbuj za chwilę");
    }
    Ok(Some(r.error_for_status()?.json().await?))
}

// ---------------------------------------------------------------------------
// Search & browse

pub async fn search(ctx: &Ctx, q: &SearchQuery) -> Result<SearchResult> {
    let mut facets: Vec<Vec<String>> = vec![vec![format!("project_type:{}", q.kind.project_type())]];
    if let Some(v) = q.game_version.as_deref().filter(|v| !v.is_empty()) {
        facets.push(vec![format!("versions:{v}")]);
    }
    if let Some(l) = q.loader {
        let names = loader_names(q.kind, l);
        if !names.is_empty() {
            facets.push(names.iter().map(|n| format!("categories:{n}")).collect());
        }
    }
    for c in &q.categories {
        facets.push(vec![format!("categories:{c}")]);
    }
    let index = match q.sort.as_deref() {
        Some(s @ ("downloads" | "follows" | "newest" | "updated")) => s,
        _ if q.query.trim().is_empty() => "downloads",
        _ => "relevance",
    };
    let query = vec![
        ("query", q.query.trim().to_string()),
        ("facets", serde_json::to_string(&facets)?),
        ("index", index.to_string()),
        ("offset", q.offset.to_string()),
        ("limit", "20".to_string()),
    ];
    get(ctx, &format!("{API}/search"), &query).await?.ok_or_else(|| Error::msg("Brak wyników"))
}

pub async fn project(ctx: &Ctx, id: &str) -> Result<Project> {
    get(ctx, &format!("{API}/project/{id}"), &[]).await?.ok_or_else(|| Error::msg("Nie znaleziono projektu"))
}

/// All versions of a project, newest first. Filters are optional so the UI
/// can also offer older (downgrade) or "incompatible" versions.
pub async fn versions(ctx: &Ctx, id: &str, loaders: &[&str], game_version: Option<&str>) -> Result<Vec<Version>> {
    let mut q = vec![];
    if !loaders.is_empty() {
        q.push(("loaders", json_list(loaders)));
    }
    if let Some(v) = game_version {
        q.push(("game_versions", json_list(&[v])));
    }
    Ok(get(ctx, &format!("{API}/project/{id}/version"), &q).await?.unwrap_or_default())
}

pub async fn version(ctx: &Ctx, id: &str) -> Result<Version> {
    get(ctx, &format!("{API}/version/{id}"), &[]).await?.ok_or_else(|| Error::msg("Nie znaleziono wersji"))
}

/// Newest version of a project compatible with the instance.
pub async fn best_version(ctx: &Ctx, project_id: &str, kind: ContentKind, inst: &Instance) -> Result<Option<Version>> {
    let loaders = loader_names(kind, inst.loader);
    let list = versions(ctx, project_id, &loaders, Some(&inst.mc_version)).await?;
    // Prefer releases, then anything.
    Ok(list.iter().find(|v| v.version_type == "release").cloned().or_else(|| list.into_iter().next()))
}

// ---------------------------------------------------------------------------
// Installing

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub title: String,
    pub file: String,
    pub dependency: bool,
}

/// Install a specific version (or the best compatible one when `version_id`
/// is `None`) plus its required dependencies. Replaces any other version of
/// the same project already in the instance — that is how updates and
/// downgrades work.
pub async fn install(
    ctx: &Ctx,
    inst: &Instance,
    kind: ContentKind,
    project_id: &str,
    version_id: Option<&str>,
    reporter: &Reporter,
    cancel: &CancellationToken,
) -> Result<Vec<Installed>> {
    if kind == ContentKind::Mod && inst.loader.is_plain() {
        bail!("Ta instancja nie obsługuje modów — zmień loader na Fabric");
    }
    // One metadata writer per instance at a time (installs, identify).
    let _guard = content::lock(&inst.id).await;
    let mut meta = content::load_meta(ctx, &inst.id);
    let mut queue: Vec<(String, Option<String>, bool)> = vec![(project_id.to_string(), version_id.map(str::to_string), false)];
    let mut seen = HashSet::new();
    let mut plan: Vec<(Version, Project, bool)> = Vec::new();

    while let Some((pid, vid, is_dep)) = queue.pop() {
        if !seen.insert(pid.clone()) {
            continue;
        }
        // Dependencies already present are left alone.
        if is_dep && meta.values().any(|m| m.project_id == pid) {
            continue;
        }
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let v = match vid {
            Some(v) => version(ctx, &v).await?,
            None => match best_version(ctx, &pid, kind, inst).await? {
                Some(v) => v,
                None if is_dep => {
                    log::warn!("dependency {pid} has no build for {}", inst.mc_version);
                    continue;
                }
                None => bail!("Ten projekt nie ma wersji dla {} {}", inst.loader.label(), inst.mc_version),
            },
        };
        let p = project(ctx, &pid).await?;
        reporter.report("content", plan.len() as u64, (plan.len() + queue.len() + 1) as u64, format!("Przygotowywanie: {}", p.title));
        for d in v.dependencies.iter().filter(|d| d.dependency_type == "required") {
            // Fabric API on Quilt → Quilt's own port (see performance mods).
            let dep = match (inst.loader, d.project_id.as_deref()) {
                (Loader::Quilt, Some("P7dR8mSH")) => Some(("qvIfYCYJ".to_string(), None)),
                (_, Some(id)) => Some((id.to_string(), d.version_id.clone())),
                (_, None) => d.version_id.clone().map(|vid| (String::new(), Some(vid))),
            };
            if let Some((dpid, dvid)) = dep {
                let dpid = if dpid.is_empty() { version(ctx, dvid.as_deref().unwrap()).await?.project_id } else { dpid };
                queue.push((dpid, dvid, true));
            }
        }
        plan.push((v, p, is_dep));
    }

    let dir = ctx.paths.game_dir(&inst.id).join(kind.dir());
    std::fs::create_dir_all(&dir)?;
    let mut items = Vec::new();
    let mut out = Vec::new();
    for (v, p, dep) in &plan {
        let Some(f) = v.primary_file() else { continue };
        if !content::safe_name(&f.filename) {
            bail!("Podejrzana nazwa pliku: {}", f.filename);
        }
        items.push(DlItem::new(&f.url, dir.join(&f.filename)).sha1(Some(f.hashes.sha1.clone())).size(Some(f.size)));
        out.push(Installed { title: p.title.clone(), file: f.filename.clone(), dependency: *dep });
    }
    download_all(&ctx.http, items, 6, reporter, "content", "Pobieranie", cancel).await?;

    // The instance may have been deleted while we were downloading.
    if !ctx.paths.instance_dir(&inst.id).join("instance.json").exists() {
        let _ = std::fs::remove_dir_all(ctx.paths.instance_dir(&inst.id));
        bail!("Instancja została usunięta w trakcie instalacji");
    }

    // Swap out older files of the same projects, then record metadata. A
    // project the player had disabled stays disabled after an update.
    for (v, p, _) in &plan {
        let Some(f) = v.primary_file() else { continue };
        let previous: Vec<String> = meta.iter().filter(|(_, m)| m.project_id == p.id && m.kind == kind).map(|(k, _)| k.clone()).collect();
        let was_disabled = previous.iter().any(|k| dir.join(format!("{k}.disabled")).exists());
        for k in previous.iter().filter(|k| **k != f.filename) {
            content::remove_file(ctx, &inst.id, kind, k)?;
            meta.remove(k);
        }
        let stale = dir.join(format!("{}.disabled", f.filename));
        if stale.exists() {
            std::fs::remove_file(&stale)?;
        }
        if was_disabled {
            std::fs::rename(dir.join(&f.filename), &stale)?;
        }
        meta.insert(
            f.filename.clone(),
            ContentMeta {
                kind,
                project_id: p.id.clone(),
                version_id: v.id.clone(),
                title: p.title.clone(),
                icon_url: p.icon_url.clone(),
                version_number: v.version_number.clone(),
            },
        );
    }
    content::save_meta(ctx, &inst.id, &meta)?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// Identify unknown files + updates

#[derive(Deserialize)]
struct ProjectBrief {
    id: String,
    title: String,
    #[serde(default)]
    icon_url: Option<String>,
}

/// Look up files without metadata by SHA1 and remember what they are.
pub async fn identify(ctx: &Ctx, inst_id: &str, kind: ContentKind) -> Result<()> {
    let _guard = content::lock(inst_id).await;
    let mut meta = content::load_meta(ctx, inst_id);
    let unknown: Vec<(String, String)> = content::files(ctx, inst_id, kind)
        .into_iter()
        .filter(|(name, _)| !meta.contains_key(name))
        .filter_map(|(name, path)| crate::download::sha1_file(&path).ok().map(|h| (h, name)))
        .collect();
    if unknown.is_empty() {
        return Ok(());
    }
    let hashes: Vec<&str> = unknown.iter().map(|(h, _)| h.as_str()).collect();
    let r = ctx.http.post(format!("{API}/version_files")).json(&json!({ "hashes": hashes, "algorithm": "sha1" })).send().await?;
    let found: HashMap<String, Version> = r.error_for_status()?.json().await?;
    if found.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = found.values().map(|v| v.project_id.as_str()).collect::<HashSet<_>>().into_iter().collect();
    let projects: Vec<ProjectBrief> = get(ctx, &format!("{API}/projects"), &[("ids", json_list(&ids))]).await?.unwrap_or_default();
    let by_id: HashMap<&str, &ProjectBrief> = projects.iter().map(|p| (p.id.as_str(), p)).collect();
    for (hash, name) in &unknown {
        let Some(v) = found.get(hash) else { continue };
        let Some(p) = by_id.get(v.project_id.as_str()) else { continue };
        meta.insert(
            name.clone(),
            ContentMeta { kind, project_id: p.id.clone(), version_id: v.id.clone(), title: p.title.clone(), icon_url: p.icon_url.clone(), version_number: v.version_number.clone() },
        );
    }
    content::save_meta(ctx, inst_id, &meta)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    pub file: String,
    pub project_id: String,
    pub version_id: String,
    pub version_number: String,
}

/// Newer compatible versions for every known file of `kind`.
pub async fn check_updates(ctx: &Ctx, inst: &Instance, kind: ContentKind) -> Result<Vec<Update>> {
    let meta = content::load_meta(ctx, &inst.id);
    let mut by_hash = HashMap::new();
    for (name, path) in content::files(ctx, &inst.id, kind) {
        if let (Some(m), Ok(h)) = (meta.get(&name), crate::download::sha1_file(&path)) {
            by_hash.insert(h, (name, m.clone()));
        }
    }
    if by_hash.is_empty() {
        return Ok(vec![]);
    }
    let mut body = json!({ "hashes": by_hash.keys().collect::<Vec<_>>(), "algorithm": "sha1", "game_versions": [inst.mc_version] });
    let loaders = loader_names(kind, inst.loader);
    if !loaders.is_empty() {
        body["loaders"] = json!(loaders);
    }
    let r = ctx.http.post(format!("{API}/version_files/update")).json(&body).send().await?;
    let latest: HashMap<String, Version> = r.error_for_status()?.json().await?;
    Ok(latest
        .into_iter()
        .filter_map(|(hash, v)| {
            let (file, m) = by_hash.get(&hash)?;
            (v.id != m.version_id).then(|| Update { file: file.clone(), project_id: m.project_id.clone(), version_id: v.id, version_number: v.version_number })
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Performance mod set

/// Install the performance mod set. Mods without a build for this version are
/// skipped. Returns the names that were installed.
pub async fn install_performance_mods(ctx: &Ctx, inst: &Instance, reporter: &Reporter, cancel: &CancellationToken) -> Result<Vec<String>> {
    if inst.loader.is_plain() {
        return Ok(vec![]);
    }
    reporter.report("mods", 0, PERFORMANCE_MODS.len() as u64, "Instalowanie modów optymalizacyjnych…");
    let meta = content::load_meta(ctx, &inst.id);
    let existing: Vec<String> = content::files(ctx, &inst.id, ContentKind::Mod).into_iter().map(|(n, _)| n.to_lowercase()).collect();
    let mut names = Vec::new();
    for &(slug, name) in PERFORMANCE_MODS {
        // Plain Fabric API can demand a newer fabricloader than Quilt provides;
        // Quilt ships its own port of it.
        let (slug, name) = if inst.loader == Loader::Quilt && slug == "fabric-api" { ("qsl", "Quilted Fabric API") } else { (slug, name) };
        // Don't fight a jar the player already has (any version).
        if meta.values().any(|m| m.title.eq_ignore_ascii_case(name)) || existing.iter().any(|e| e.starts_with(slug)) {
            continue;
        }
        match install(ctx, inst, ContentKind::Mod, slug, None, reporter, cancel).await {
            Ok(done) if !done.is_empty() => names.push(name.to_string()),
            Ok(_) => {}
            Err(Error::Cancelled) => return Err(Error::Cancelled),
            Err(e) => log::info!("{name}: skipped ({e})"),
        }
    }
    Ok(names)
}
