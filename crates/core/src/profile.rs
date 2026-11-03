//! Minecraft profile: skins and capes of a signed-in account.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{bail, Ctx, Result};

const PROFILE: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    pub id: String,
    /// `ACTIVE` or `INACTIVE`.
    pub state: String,
    pub url: String,
    #[serde(default)]
    pub variant: Option<String>,
    /// Cape name, e.g. "Migrator".
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skins: Vec<Texture>,
    #[serde(default)]
    pub capes: Vec<Texture>,
}

impl Profile {
    pub fn active_skin(&self) -> Option<&Texture> {
        self.skins.iter().find(|s| s.state == "ACTIVE")
    }
    pub fn active_cape(&self) -> Option<&Texture> {
        self.capes.iter().find(|s| s.state == "ACTIVE")
    }
}

async fn check(r: reqwest::Response, what: &str) -> Result<Profile> {
    let status = r.status();
    if status.as_u16() == 401 {
        bail!("Sesja wygasła — zaloguj się ponownie", "Session expired, sign in again");
    }
    if status.as_u16() == 429 {
        bail!("Za dużo zmian naraz — Mojang pozwala na kilka na minutę, spróbuj za chwilę", "Too many changes at once. Mojang allows a few per minute, try again in a moment");
    }
    if !status.is_success() {
        let body = r.text().await.unwrap_or_default();
        log::warn!("{what} failed: {status} {body}");
        bail!("Nie udało się {what} ({status})", "Could not {what} ({status})");
    }
    Ok(r.json().await?)
}

pub async fn get(ctx: &Ctx, token: &str) -> Result<Profile> {
    check(ctx.http.get(PROFILE).bearer_auth(token).send().await?, crate::lang::tr("pobrać profilu", "fetch the profile")).await
}

/// Show a cape (`Some(id)`) or hide all capes (`None`).
pub async fn set_cape(ctx: &Ctx, token: &str, cape_id: Option<&str>) -> Result<Profile> {
    let url = format!("{PROFILE}/capes/active");
    let r = match cape_id {
        Some(id) => ctx.http.put(url).bearer_auth(token).json(&json!({ "capeId": id })).send().await?,
        None => ctx.http.delete(url).bearer_auth(token).send().await?,
    };
    check(r, crate::lang::tr("zmienić peleryny", "change the cape")).await
}

/// Upload a skin PNG; `slim` selects the Alex arm model.
pub async fn upload_skin(ctx: &Ctx, token: &str, png: Vec<u8>, slim: bool) -> Result<Profile> {
    match crate::wallpapers::png_size(&png) {
        Some((64, 64)) | Some((64, 32)) => {}
        _ => bail!("Skin musi być plikiem PNG 64×64", "A skin must be a 64×64 PNG file"),
    }
    let part = reqwest::multipart::Part::bytes(png).file_name("skin.png").mime_str("image/png")?;
    let form = reqwest::multipart::Form::new().text("variant", if slim { "slim" } else { "classic" }).part("file", part);
    check(ctx.http.post(format!("{PROFILE}/skins")).bearer_auth(token).multipart(form).send().await?, crate::lang::tr("zmienić skina", "change the skin")).await
}

/// Switch only the arm model of the current skin by re-uploading it.
pub async fn set_variant(ctx: &Ctx, token: &str, skin_url: &str, slim: bool) -> Result<Profile> {
    let r = ctx.http.post(format!("{PROFILE}/skins")).bearer_auth(token).json(&json!({ "variant": if slim { "slim" } else { "classic" }, "url": skin_url })).send().await?;
    check(r, crate::lang::tr("zmienić modelu postaci", "change the character model")).await
}
