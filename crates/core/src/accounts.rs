//! Saved Microsoft accounts (`accounts.json`, readable only by the user).

use serde::{Deserialize, Serialize};

use crate::auth::{self, McSession, MsTokens};
use crate::{bail, Ctx, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Minecraft profile UUID (no dashes).
    pub uuid: String,
    pub name: String,
    pub ms_refresh_token: String,
    pub mc_access_token: String,
    pub mc_expires_at: i64,
    #[serde(default)]
    pub xuid: Option<String>,
    #[serde(default)]
    pub skin_url: Option<String>,
    #[serde(default)]
    pub skin_slim: bool,
    #[serde(default)]
    pub cape_url: Option<String>,
    pub added_at: i64,
}

/// What the UI gets to see — never the tokens.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountView {
    pub uuid: String,
    pub name: String,
    pub skin_url: Option<String>,
    pub skin_slim: bool,
    pub cape_url: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Accounts {
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub list: Vec<Account>,
}

impl Accounts {
    pub fn load(ctx: &Ctx) -> Self {
        std::fs::read(ctx.paths.accounts_file()).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
    }

    pub fn save(&self, ctx: &Ctx) -> Result<()> {
        crate::paths::write_atomic_mode(&ctx.paths.accounts_file(), &serde_json::to_vec_pretty(self)?, 0o600)?;
        Ok(())
    }

    pub fn views(&self) -> Vec<AccountView> {
        self.list
            .iter()
            .map(|a| AccountView {
                uuid: a.uuid.clone(),
                name: a.name.clone(),
                skin_url: a.skin_url.clone(),
                skin_slim: a.skin_slim,
                cape_url: a.cape_url.clone(),
                active: self.active.as_deref() == Some(&a.uuid),
            })
            .collect()
    }

    pub fn active(&self) -> Option<&Account> {
        let id = self.active.as_deref()?;
        self.list.iter().find(|a| a.uuid == id)
    }

    /// Insert or replace (same UUID) and make it active.
    pub fn upsert(&mut self, acc: Account) {
        self.active = Some(acc.uuid.clone());
        match self.list.iter_mut().find(|a| a.uuid == acc.uuid) {
            Some(a) => *a = acc,
            None => self.list.push(acc),
        }
    }

    pub fn remove(&mut self, uuid: &str) {
        self.list.retain(|a| a.uuid != uuid);
        if self.active.as_deref() == Some(uuid) {
            self.active = self.list.first().map(|a| a.uuid.clone());
        }
    }
}

fn from_session(tokens: MsTokens, s: McSession) -> Account {
    Account {
        uuid: s.uuid,
        name: s.name,
        ms_refresh_token: tokens.refresh_token,
        mc_access_token: s.access_token,
        mc_expires_at: s.expires_at,
        xuid: s.xuid,
        skin_url: s.skin_url,
        skin_slim: s.skin_slim,
        cape_url: s.cape_url,
        added_at: chrono::Utc::now().timestamp(),
    }
}

impl Account {
    /// Adopt skin/cape changes from a freshly fetched profile.
    pub fn apply_profile(&mut self, p: &crate::profile::Profile) {
        self.name = p.name.clone();
        let skin = p.active_skin();
        self.skin_url = skin.map(|s| s.url.clone());
        self.skin_slim = skin.and_then(|s| s.variant.as_deref()).is_some_and(|v| v.eq_ignore_ascii_case("slim"));
        self.cape_url = p.active_cape().map(|c| c.url.clone());
    }
}

/// Finish a login once we have Microsoft tokens.
pub async fn complete_login(ctx: &Ctx, tokens: MsTokens) -> Result<Account> {
    let session = auth::minecraft_login(&ctx.http, &tokens.access_token).await?;
    Ok(from_session(tokens, session))
}

/// Refresh the Minecraft token if it expires within 10 minutes. Also picks up
/// skin/name changes.
pub async fn ensure_fresh(ctx: &Ctx, acc: &Account) -> Result<Option<Account>> {
    if acc.mc_expires_at - chrono::Utc::now().timestamp() > 600 {
        return Ok(None);
    }
    if acc.ms_refresh_token.is_empty() {
        bail!("Konto {} wymaga ponownego zalogowania", acc.name);
    }
    let tokens = auth::refresh(&ctx.http, &acc.ms_refresh_token).await?;
    let mut fresh = complete_login(ctx, tokens).await?;
    fresh.added_at = acc.added_at;
    Ok(Some(fresh))
}

/// Skin PNG as a `data:` URL so the webview can draw it without CORS trouble.
/// Cached on disk by URL.
pub async fn skin_data_url(ctx: &Ctx, url: &str) -> Result<String> {
    use base64::Engine;
    if !url.starts_with("https://textures.minecraft.net/") && !url.starts_with("http://textures.minecraft.net/") {
        bail!("Nieznane źródło skina");
    }
    let name = url.rsplit('/').next().unwrap_or("skin");
    let cache = ctx.paths.root.join("cache/skins").join(format!("{name}.png"));
    let bytes = match std::fs::read(&cache) {
        Ok(b) => b,
        Err(_) => {
            let b = crate::http::get_bytes(&ctx.http, &url.replace("http://", "https://")).await?;
            crate::paths::write_atomic(&cache, &b)?;
            b
        }
    };
    Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
}
