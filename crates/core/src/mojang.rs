//! Mojang version manifest and version JSON format, plus the rule engine that
//! decides which libraries and arguments apply to this machine.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{http, Ctx, Result};

pub const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub latest: Latest,
    pub versions: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub release_time: String,
    #[serde(default)]
    pub sha1: Option<String>,
}

pub async fn manifest(ctx: &Ctx) -> Result<Manifest> {
    let cache = ctx.paths.versions().join("version_manifest_v2.json");
    match http::get_json::<Manifest>(&ctx.http, MANIFEST_URL).await {
        Ok(m) => {
            let _ = crate::paths::write_atomic(&cache, &serde_json::to_vec(&m)?);
            Ok(m)
        }
        // Offline: fall back to the last manifest we saw.
        Err(e) => match std::fs::read(&cache) {
            Ok(b) => Ok(serde_json::from_slice(&b)?),
            Err(_) => Err(e),
        },
    }
}

// ---------------------------------------------------------------------------
// Version JSON

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionJson {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inherits_from: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub main_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minecraft_arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_index: Option<AssetIndexRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assets: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<Downloads>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_version: Option<JavaVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<Logging>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_time: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Arg>,
    #[serde(default)]
    pub jvm: Vec<Arg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Arg {
    Plain(String),
    Ruled { rules: Vec<Rule>, value: OneOrMany },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            OneOrMany::One(s) => vec![s.clone()],
            OneOrMany::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<OsRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub downloads: Option<LibDownloads>,
    /// Maven repository base (Fabric/Quilt style libraries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<Rule>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<Extract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibDownloads {
    #[serde(default)]
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extract {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    #[serde(default)]
    pub client: Option<Artifact>,
    #[serde(default)]
    pub server: Option<Artifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    #[serde(default)]
    pub client: Option<LoggingClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub url: String,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

// ---------------------------------------------------------------------------
// Rules

pub fn os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    }
}

/// Feature flags the launcher advertises when evaluating argument rules.
#[derive(Debug, Clone, Default)]
pub struct Features {
    pub custom_resolution: bool,
}

impl Features {
    fn get(&self, name: &str) -> bool {
        match name {
            "has_custom_resolution" => self.custom_resolution,
            // demo user, quick play, etc. are never enabled.
            _ => false,
        }
    }
}

fn rule_matches(rule: &Rule, features: &Features) -> bool {
    if let Some(os) = &rule.os {
        if let Some(name) = &os.name {
            if name != os_name() {
                return false;
            }
        }
        if let Some(arch) = &os.arch {
            let ours = match std::env::consts::ARCH {
                "x86" => "x86",
                "aarch64" => "arm64",
                other => other,
            };
            if arch != ours {
                return false;
            }
        }
    }
    if let Some(f) = &rule.features {
        if f.iter().any(|(k, v)| features.get(k) != *v) {
            return false;
        }
    }
    true
}

pub fn rules_allow(rules: Option<&[Rule]>, features: &Features) -> bool {
    let Some(rules) = rules else { return true };
    if rules.is_empty() {
        return true;
    }
    let mut allowed = false;
    for r in rules {
        if rule_matches(r, features) {
            allowed = r.action == "allow";
        }
    }
    allowed
}

// ---------------------------------------------------------------------------
// Maven coordinates

pub struct Coord<'a> {
    pub group: &'a str,
    pub artifact: &'a str,
    pub version: &'a str,
    pub classifier: Option<&'a str>,
    pub ext: &'a str,
}

pub fn parse_coord(name: &str) -> Option<Coord<'_>> {
    let (name, ext) = match name.split_once('@') {
        Some((n, e)) => (n, e),
        None => (name, "jar"),
    };
    let mut parts = name.split(':');
    let group = parts.next()?;
    let artifact = parts.next()?;
    let version = parts.next()?;
    let classifier = parts.next();
    Some(Coord { group, artifact, version, classifier, ext })
}

pub fn maven_path(name: &str) -> Option<String> {
    let c = parse_coord(name)?;
    let file = match c.classifier {
        Some(cl) => format!("{}-{}-{}.{}", c.artifact, c.version, cl, c.ext),
        None => format!("{}-{}.{}", c.artifact, c.version, c.ext),
    };
    Some(format!("{}/{}/{}/{}", c.group.replace('.', "/"), c.artifact, c.version, file))
}

/// `group:artifact[:classifier]` — two libraries with the same key are the
/// same thing in different versions, and the loader's one must win.
pub fn library_key(name: &str) -> String {
    match parse_coord(name) {
        Some(c) => match c.classifier {
            Some(cl) => format!("{}:{}:{}", c.group, c.artifact, cl),
            None => format!("{}:{}", c.group, c.artifact),
        },
        None => name.to_string(),
    }
}

/// Fold a child profile (Fabric, Quilt) onto its parent (vanilla).
pub fn merge(parent: VersionJson, child: VersionJson) -> VersionJson {
    let mut seen = std::collections::HashSet::new();
    let mut libraries = Vec::new();
    for lib in child.libraries.into_iter().chain(parent.libraries) {
        if seen.insert(library_key(&lib.name)) {
            libraries.push(lib);
        }
    }
    let arguments = match (parent.arguments, child.arguments) {
        (Some(mut p), Some(c)) => {
            p.game.extend(c.game);
            p.jvm.extend(c.jvm);
            Some(p)
        }
        (p, c) => c.or(p),
    };
    VersionJson {
        id: child.id,
        inherits_from: None,
        kind: child.kind.or(parent.kind),
        main_class: child.main_class.or(parent.main_class),
        minecraft_arguments: child.minecraft_arguments.or(parent.minecraft_arguments),
        arguments,
        libraries,
        asset_index: child.asset_index.or(parent.asset_index),
        assets: child.assets.or(parent.assets),
        downloads: child.downloads.or(parent.downloads),
        java_version: child.java_version.or(parent.java_version),
        logging: child.logging.or(parent.logging),
        release_time: parent.release_time.or(child.release_time),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maven_paths() {
        assert_eq!(maven_path("net.fabricmc:fabric-loader:0.16.9").unwrap(), "net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar");
        assert_eq!(
            maven_path("org.lwjgl:lwjgl:3.3.3:natives-linux").unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-linux.jar"
        );
        assert_eq!(maven_path("a.b:c:1@zip").unwrap(), "a/b/c/1/c-1.zip");
    }

    #[test]
    fn rules() {
        let f = Features::default();
        let only_osx = vec![Rule { action: "allow".into(), os: Some(OsRule { name: Some("osx".into()), arch: None, version: None }), features: None }];
        assert_eq!(rules_allow(Some(&only_osx), &f), os_name() == "osx");
        let not_osx = vec![
            Rule { action: "allow".into(), os: None, features: None },
            Rule { action: "disallow".into(), os: Some(OsRule { name: Some("osx".into()), arch: None, version: None }), features: None },
        ];
        assert_eq!(rules_allow(Some(&not_osx), &f), os_name() != "osx");
        let demo = vec![Rule { action: "allow".into(), os: None, features: Some([("is_demo_user".to_string(), true)].into()) }];
        assert!(!rules_allow(Some(&demo), &f));
    }

    #[test]
    fn merge_prefers_child_libraries() {
        let lib = |n: &str| Library { name: n.into(), downloads: None, url: None, rules: None, natives: None, extract: None, sha1: None, size: None };
        let parent = VersionJson { id: "1.21.1".into(), main_class: Some("net.minecraft.client.main.Main".into()), libraries: vec![lib("org.ow2.asm:asm:9.3"), lib("org.lwjgl:lwjgl:3.3.3"), lib("org.lwjgl:lwjgl:3.3.3:natives-linux")], ..Default::default() };
        let child = VersionJson { id: "fabric".into(), main_class: Some("net.fabricmc.loader.impl.launch.knot.KnotClient".into()), libraries: vec![lib("org.ow2.asm:asm:9.7")], ..Default::default() };
        let m = merge(parent, child);
        let names: Vec<_> = m.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(names, ["org.ow2.asm:asm:9.7", "org.lwjgl:lwjgl:3.3.3", "org.lwjgl:lwjgl:3.3.3:natives-linux"]);
        assert_eq!(m.main_class.unwrap(), "net.fabricmc.loader.impl.launch.knot.KnotClient");
    }
}
