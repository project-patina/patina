//! Instances: a name, a game version, a loader and their own game directory.

use serde::{Deserialize, Serialize};

use crate::loaders::Loader;
use crate::{bail, Ctx, Error, Result};

/// Per-instance values that win over the global settings when set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Overrides {
    pub memory_min_mb: Option<u32>,
    pub memory_max_mb: Option<u32>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub game_width: Option<u32>,
    pub game_height: Option<u32>,
    pub fullscreen: Option<bool>,
    pub env_vars: Option<String>,
    pub wrapper_command: Option<String>,
    /// `host[:port]` to join right after start (1.20+ quick play, older `--server`).
    pub join_server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    /// Icon preset key the UI understands (`grass`, `diamond`, …).
    #[serde(default = "default_icon")]
    pub icon: String,
    /// Banner gradient preset key.
    #[serde(default = "default_banner")]
    pub banner: String,
    pub created_at: i64,
    #[serde(default)]
    pub last_played: Option<i64>,
    #[serde(default)]
    pub play_time_secs: u64,
    #[serde(default)]
    pub overrides: Overrides,
    /// Set once the first full install (incl. performance mods) succeeded.
    #[serde(default)]
    pub installed: bool,
}

fn default_icon() -> String {
    "grass".into()
}
fn default_banner() -> String {
    "aurora".into()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInstance {
    pub name: String,
    pub mc_version: String,
    pub loader: Loader,
    #[serde(default)]
    pub loader_version: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub banner: Option<String>,
}

fn slug(name: &str) -> String {
    let s: String = name
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'ą' => 'a',
            'ć' => 'c',
            'ę' => 'e',
            'ł' => 'l',
            'ń' => 'n',
            'ó' => 'o',
            'ś' => 's',
            'ź' | 'ż' => 'z',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        })
        .collect();
    let s = s.split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    let s: String = s.chars().take(32).collect();
    if s.is_empty() { "instancja".into() } else { s }
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Version IDs become directory and file names. Keep historical IDs with spaces.
pub fn validate_version_id(id: &str) -> Result<()> {
    if id.trim().is_empty() || id.len() > 128 || id == "." || id == ".."
        || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b" ._+-".contains(&b)) {
        bail!("Niepoprawny identyfikator wersji gry");
    }
    Ok(())
}

impl Instance {
    pub fn load(ctx: &Ctx, id: &str) -> Result<Self> {
        if !valid_id(id) {
            bail!("Niepoprawny identyfikator instancji");
        }
        let path = ctx.paths.instance_dir(id).join("instance.json");
        let bytes = std::fs::read(&path).map_err(|_| Error::msg(format!("Nie znaleziono instancji {id}")))?;
        let inst: Self = serde_json::from_slice(&bytes)?;
        if inst.id != id {
            bail!("Identyfikator instancji nie zgadza się z folderem");
        }
        validate_version_id(&inst.mc_version)?;
        Ok(inst)
    }

    pub fn save(&self, ctx: &Ctx) -> Result<()> {
        if !valid_id(&self.id) {
            bail!("Niepoprawny identyfikator instancji");
        }
        crate::paths::write_atomic(&ctx.paths.instance_dir(&self.id).join("instance.json"), &serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    pub fn list(ctx: &Ctx) -> Vec<Self> {
        let mut out: Vec<Self> = std::fs::read_dir(ctx.paths.instances())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| Self::load(ctx, &e.file_name().to_string_lossy()).ok())
            .collect();
        // Most recently played first, then newest.
        out.sort_by_key(|i| std::cmp::Reverse(i.last_played.unwrap_or(0).max(i.created_at)));
        out
    }

    pub fn create(ctx: &Ctx, req: NewInstance) -> Result<Self> {
        let name = req.name.trim();
        if name.is_empty() {
            bail!("Podaj nazwę instancji");
        }
        if req.loader != Loader::Rust {
            validate_version_id(&req.mc_version)?;
        }
        let base = slug(name);
        let mut id = base.clone();
        let mut n = 2;
        while ctx.paths.instance_dir(&id).exists() {
            id = format!("{base}-{n}");
            n += 1;
        }
        let inst = Instance {
            id,
            name: name.chars().take(48).collect(),
            mc_version: if req.loader == Loader::Rust { crate::engine::MC_VERSION.into() } else { req.mc_version },
            loader: req.loader,
            loader_version: if req.loader.is_plain() { None } else { req.loader_version.filter(|v| !v.is_empty()) },
            icon: req.icon.unwrap_or_else(default_icon),
            banner: req.banner.unwrap_or_else(default_banner),
            created_at: chrono::Utc::now().timestamp(),
            last_played: None,
            play_time_secs: 0,
            overrides: Overrides::default(),
            installed: false,
        };
        for sub in ["mods", "resourcepacks", "shaderpacks", "saves", "screenshots"] {
            std::fs::create_dir_all(ctx.paths.game_dir(&inst.id).join(sub))?;
        }
        inst.save(ctx)?;
        Ok(inst)
    }

    /// Move the instance folder to the system trash (recoverable).
    pub fn delete(ctx: &Ctx, id: &str) -> Result<()> {
        if !valid_id(id) {
            bail!("Niepoprawny identyfikator instancji");
        }
        let dir = ctx.paths.instance_dir(id);
        if !dir.exists() {
            return Ok(());
        }
        trash::delete(&dir).map_err(|e| Error::msg(format!("Nie udało się przenieść instancji do kosza. Pliki pozostały na miejscu: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_ids_cannot_escape_the_version_directory() {
        for id in ["", "..", "../other", "a/b", "a\\b", "/tmp/game", "C:game"] {
            assert!(super::validate_version_id(id).is_err(), "accepted {id:?}");
        }
        for id in ["1.21.1", "24w14a", "1.14 Pre-Release 1", "rd-132211"] {
            assert!(super::validate_version_id(id).is_ok(), "rejected {id:?}");
        }
    }
    #[test]
    fn slugs() {
        assert_eq!(super::slug("Mój Świat 1.21!"), "moj-swiat-1-21");
        assert_eq!(super::slug("???"), "instancja");
    }
}
