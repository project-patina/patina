//! Mod loaders. Fabric and Quilt publish ready-made launcher profiles that
//! inherit from the vanilla version, so "installing" them is just fetching that
//! profile and its maven libraries.

use serde::{Deserialize, Serialize};

use crate::mojang::VersionJson;
use crate::{bail, http, Ctx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    Vanilla,
    #[default]
    Fabric,
    Quilt,
    /// The game itself rewritten in Rust (`engine.rs`), not a mod loader.
    Rust,
}

impl Loader {
    fn meta(self) -> Option<&'static str> {
        match self {
            Loader::Vanilla | Loader::Rust => None,
            Loader::Fabric => Some("https://meta.fabricmc.net/v2"),
            Loader::Quilt => Some("https://meta.quiltmc.org/v3"),
        }
    }

    /// Whether the instance runs Mojang's game as it is, with no loader profile on top.
    pub fn is_plain(self) -> bool {
        matches!(self, Loader::Vanilla | Loader::Rust)
    }

    pub fn label(self) -> &'static str {
        match self {
            Loader::Vanilla => "Vanilla",
            Loader::Fabric => "Fabric",
            Loader::Quilt => "Quilt",
            Loader::Rust => "Rust",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoaderVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Deserialize)]
struct LoaderEntry {
    loader: LoaderInfo,
}

#[derive(Deserialize)]
struct LoaderInfo {
    version: String,
    #[serde(default)]
    stable: Option<bool>,
}

/// Loader versions that support `mc`, newest first. Empty when the loader does
/// not support that Minecraft version.
pub async fn versions(ctx: &Ctx, loader: Loader, mc: &str) -> Result<Vec<LoaderVersion>> {
    let Some(meta) = loader.meta() else { return Ok(vec![]) };
    let url = format!("{meta}/versions/loader/{mc}");
    let entries: Vec<LoaderEntry> = match http::get_json(&ctx.http, &url).await {
        Ok(e) => e,
        // Both metas answer 400/404 for unknown game versions.
        Err(crate::Error::Http(e)) if e.status().is_some_and(|s| s.is_client_error()) => vec![],
        Err(e) => return Err(e),
    };
    let mut out: Vec<LoaderVersion> = entries
        .into_iter()
        .map(|e| {
            // Quilt has no "stable" flag; its betas carry "beta" in the version.
            let stable = e.loader.stable.unwrap_or_else(|| !e.loader.version.contains('-'));
            LoaderVersion { version: e.loader.version, stable }
        })
        .collect();
    // Quilt's meta is not ordered.
    out.sort_by(|a, b| version_cmp(&b.version, &a.version));
    Ok(out)
}

/// Compare `1.2.3[-pre.N]` style versions; a release sorts above its pre-releases.
pub fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    fn split(v: &str) -> (Vec<u64>, Option<Vec<String>>) {
        let (base, pre) = match v.split_once(['-', '+']) {
            Some((b, p)) => (b, Some(p)),
            None => (v, None),
        };
        (base.split('.').map(|n| n.parse().unwrap_or(0)).collect(), pre.map(|p| p.split('.').map(str::to_string).collect()))
    }
    let (ab, ap) = split(a);
    let (bb, bp) = split(b);
    ab.cmp(&bb).then_with(|| match (ap, bp) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(x), Some(y)) => {
            for (p, q) in x.iter().zip(&y) {
                let o = match (p.parse::<u64>(), q.parse::<u64>()) {
                    (Ok(m), Ok(n)) => m.cmp(&n),
                    _ => p.cmp(q),
                };
                if o.is_ne() {
                    return o;
                }
            }
            x.len().cmp(&y.len())
        }
    })
}

/// Best default loader version: newest stable, else newest.
pub async fn recommended(ctx: &Ctx, loader: Loader, mc: &str) -> Result<String> {
    let v = versions(ctx, loader, mc).await?;
    match v.iter().find(|v| v.stable).or(v.first()) {
        Some(v) => Ok(v.version.clone()),
        None => bail!("{} nie obsługuje jeszcze wersji {mc}", "{} does not support version {mc} yet", loader.label()),
    }
}

/// Fetch (or read the cached) loader profile for `mc` + `loader_version`.
pub async fn profile(ctx: &Ctx, loader: Loader, mc: &str, loader_version: &str) -> Result<VersionJson> {
    let Some(meta) = loader.meta() else { bail!("Vanilla nie ma profilu loadera", "Vanilla has no loader profile") };
    let id = format!("{}-loader-{loader_version}-{mc}", loader.label().to_lowercase());
    let path = ctx.paths.version_json(&id);
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(v) = serde_json::from_slice::<VersionJson>(&bytes) {
            return Ok(v);
        }
    }
    let url = format!("{meta}/versions/loader/{mc}/{loader_version}/profile/json");
    let bytes = http::get_bytes(&ctx.http, &url).await?;
    let mut v: VersionJson = serde_json::from_slice(&bytes)?;
    v.id = id;
    crate::paths::write_atomic(&path, &serde_json::to_vec_pretty(&v)?)?;
    Ok(v)
}

#[cfg(test)]
mod tests {
    #[test]
    fn ordering() {
        let mut v = vec!["0.20.0-beta.9", "0.24.0", "0.20.0-beta.10", "0.20.0", "0.16.1-beta.3", "0.29.2"];
        v.sort_by(|a, b| super::version_cmp(b, a));
        assert_eq!(v, ["0.29.2", "0.24.0", "0.20.0", "0.20.0-beta.10", "0.20.0-beta.9", "0.16.1-beta.3"]);
    }
}

