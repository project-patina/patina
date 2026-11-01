//! Microsoft → Xbox Live → Minecraft authentication.
//!
//! Uses the consumer `login.live.com` OAuth endpoints with the public client
//! id of the official Minecraft launcher, so no Azure app registration is
//! needed. The player signs in on Microsoft's own page; we only ever see the
//! resulting tokens.

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{bail, Error, Result};

pub const CLIENT_ID: &str = "00000000402b5328";
pub const REDIRECT_URI: &str = "https://login.live.com/oauth20_desktop.srf";
pub const SCOPE: &str = "service::user.auth.xboxlive.com::MBI_SSL";

const TOKEN_URL: &str = "https://login.live.com/oauth20_token.srf";
const DEVICE_URL: &str = "https://login.live.com/oauth20_connect.srf";

pub fn authorize_url() -> String {
    format!(
        "https://login.live.com/oauth20_authorize.srf?client_id={CLIENT_ID}&response_type=code&scope={}&redirect_uri={}&prompt=select_account",
        enc(SCOPE),
        enc(REDIRECT_URI)
    )
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Pull `code` out of the redirect URL the login page navigates to.
/// Returns `Err` when Microsoft redirected with an error instead.
pub fn code_from_redirect(url: &str) -> Option<Result<String>> {
    if !url.starts_with(REDIRECT_URI) {
        return None;
    }
    let query = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    let mut code = None;
    let mut error = None;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let v = dec(v);
        match k {
            "code" => code = Some(v),
            "error_description" => error = Some(v),
            "error" if error.is_none() => error = Some(v),
            _ => {}
        }
    }
    match (code, error) {
        (Some(c), _) => Some(Ok(c)),
        (None, Some(e)) => Some(Err(Error::msg(format!("Microsoft odrzucił logowanie: {e}")))),
        (None, None) => None,
    }
}

fn dec(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
                out.push(b'%');
            }
            b'+' => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[derive(Debug, Clone, Deserialize)]
pub struct MsTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Deserialize)]
struct OAuthError {
    error: String,
    #[serde(default)]
    error_description: Option<String>,
}

async fn token_request(http: &reqwest::Client, form: &[(&str, &str)]) -> Result<std::result::Result<MsTokens, OAuthError>> {
    let r = http.post(TOKEN_URL).form(form).send().await?;
    let status = r.status();
    let body = r.bytes().await?;
    if status.is_success() {
        return Ok(Ok(serde_json::from_slice(&body)?));
    }
    match serde_json::from_slice::<OAuthError>(&body) {
        Ok(e) => Ok(Err(e)),
        Err(_) => bail!("Microsoft zwrócił błąd {status}"),
    }
}

fn oauth_fail(e: OAuthError) -> Error {
    Error::msg(format!("Logowanie Microsoft nie powiodło się: {}", e.error_description.unwrap_or(e.error)))
}

pub async fn exchange_code(http: &reqwest::Client, code: &str) -> Result<MsTokens> {
    token_request(
        http,
        &[("client_id", CLIENT_ID), ("code", code), ("grant_type", "authorization_code"), ("redirect_uri", REDIRECT_URI), ("scope", SCOPE)],
    )
    .await?
    .map_err(oauth_fail)
}

pub async fn refresh(http: &reqwest::Client, refresh_token: &str) -> Result<MsTokens> {
    token_request(
        http,
        &[("client_id", CLIENT_ID), ("refresh_token", refresh_token), ("grant_type", "refresh_token"), ("redirect_uri", REDIRECT_URI), ("scope", SCOPE)],
    )
    .await?
    .map_err(|e| Error::msg(format!("Sesja Microsoft wygasła — zaloguj się ponownie ({})", e.error_description.unwrap_or(e.error))))
}

// ---------------------------------------------------------------------------
// Device code (fallback when the embedded login window cannot be used)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCode {
    #[serde(alias = "user_code")]
    pub user_code: String,
    #[serde(alias = "device_code")]
    pub device_code: String,
    #[serde(alias = "verification_uri")]
    pub verification_uri: String,
    #[serde(default = "five")]
    pub interval: u64,
    #[serde(alias = "expires_in")]
    pub expires_in: u64,
}

fn five() -> u64 {
    5
}

pub async fn device_code_start(http: &reqwest::Client) -> Result<DeviceCode> {
    let r = http
        .post(DEVICE_URL)
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPE), ("response_type", "device_code")])
        .send()
        .await?
        .error_for_status()?;
    Ok(r.json().await?)
}

/// Poll until the player confirms the code in their browser.
pub async fn device_code_wait(http: &reqwest::Client, dc: &DeviceCode, cancel: &tokio_util::sync::CancellationToken) -> Result<MsTokens> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(dc.expires_in);
    let mut interval = dc.interval.max(1);
    loop {
        tokio::select! {
            _ = tokio::time::sleep(std::time::Duration::from_secs(interval)) => {}
            _ = cancel.cancelled() => return Err(Error::Cancelled),
        }
        if std::time::Instant::now() > deadline {
            bail!("Kod logowania wygasł — spróbuj ponownie");
        }
        let res = token_request(
            http,
            &[("client_id", CLIENT_ID), ("device_code", &dc.device_code), ("grant_type", "urn:ietf:params:oauth:grant-type:device_code")],
        )
        .await?;
        match res {
            Ok(t) => return Ok(t),
            Err(e) if e.error == "authorization_pending" => {}
            Err(e) if e.error == "slow_down" => interval += 5,
            Err(e) => return Err(oauth_fail(e)),
        }
    }
}

// ---------------------------------------------------------------------------
// Xbox Live + Minecraft services

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct XblResponse {
    token: String,
    display_claims: DisplayClaims,
}

#[derive(Deserialize)]
struct DisplayClaims {
    xui: Vec<Xui>,
}

#[derive(Deserialize)]
struct Xui {
    uhs: String,
    #[serde(default)]
    xid: Option<String>,
}

#[derive(Deserialize)]
struct XErr {
    #[serde(rename = "XErr")]
    xerr: u64,
}

async fn xbox_user_token(http: &reqwest::Client, ms_access: &str) -> Result<String> {
    // Live tokens are passed as `t=<token>`; some accounts only accept the bare
    // token, so fall back to that.
    let mut last = String::new();
    for ticket in [format!("t={ms_access}"), ms_access.to_string()] {
        let r = http
            .post("https://user.auth.xboxlive.com/user/authenticate")
            .header("x-xbl-contract-version", "1")
            .json(&json!({
                "Properties": { "AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com", "RpsTicket": ticket },
                "RelyingParty": "http://auth.xboxlive.com",
                "TokenType": "JWT"
            }))
            .send()
            .await?;
        if r.status().is_success() {
            let x: XblResponse = r.json().await?;
            return Ok(x.token);
        }
        last = r.status().to_string();
    }
    bail!("Xbox Live odrzucił logowanie ({last})")
}

struct Xsts {
    token: String,
    uhs: String,
    xuid: Option<String>,
}

async fn xsts(http: &reqwest::Client, user_token: &str) -> Result<Xsts> {
    let r = http
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("x-xbl-contract-version", "1")
        .json(&json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [user_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT"
        }))
        .send()
        .await?;
    if r.status().as_u16() == 401 {
        let code = r.json::<XErr>().await.map(|e| e.xerr).unwrap_or(0);
        bail!(
            "{}",
            match code {
                2148916233 => "To konto Microsoft nie ma profilu Xbox. Zaloguj się raz na xbox.com, aby go utworzyć.",
                2148916235 => "Xbox Live jest niedostępny w Twoim kraju.",
                2148916236 | 2148916237 => "To konto wymaga weryfikacji wieku na xbox.com.",
                2148916238 => "To konto dziecka — musi zostać dodane do rodziny Microsoft przez dorosłego.",
                _ => "Xbox Live odrzucił autoryzację.",
            }
        );
    }
    let x: XblResponse = r.error_for_status()?.json().await?;
    let xui = x.display_claims.xui.into_iter().next().ok_or_else(|| Error::msg("Brak danych Xbox w odpowiedzi"))?;
    Ok(Xsts { token: x.token, uhs: xui.uhs, xuid: xui.xid })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McSession {
    pub access_token: String,
    /// Unix seconds.
    pub expires_at: i64,
    pub uuid: String,
    pub name: String,
    pub xuid: Option<String>,
    pub skin_url: Option<String>,
    pub skin_slim: bool,
    pub cape_url: Option<String>,
}

#[derive(Deserialize)]
struct McLogin {
    access_token: String,
    expires_in: i64,
}

#[derive(Deserialize)]
struct McProfile {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<McTexture>,
    #[serde(default)]
    capes: Vec<McTexture>,
}

#[derive(Deserialize)]
struct McTexture {
    state: String,
    url: String,
    #[serde(default)]
    variant: Option<String>,
}

/// Turn a Microsoft access token into a Minecraft session (+ profile).
pub async fn minecraft_login(http: &reqwest::Client, ms_access: &str) -> Result<McSession> {
    let user = xbox_user_token(http, ms_access).await?;
    let x = xsts(http, &user).await?;
    let r = http
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .json(&json!({ "identityToken": format!("XBL3.0 x={};{}", x.uhs, x.token) }))
        .send()
        .await?;
    if !r.status().is_success() {
        bail!("Serwery Minecraft odrzuciły logowanie ({})", r.status());
    }
    let login: McLogin = r.json().await?;
    let r = http.get("https://api.minecraftservices.com/minecraft/profile").bearer_auth(&login.access_token).send().await?;
    if r.status().as_u16() == 404 {
        bail!("To konto nie posiada Minecraft: Java Edition.");
    }
    let p: McProfile = r.error_for_status()?.json().await?;
    let skin = p.skins.iter().find(|s| s.state == "ACTIVE");
    Ok(McSession {
        access_token: login.access_token,
        expires_at: chrono::Utc::now().timestamp() + login.expires_in,
        uuid: p.id,
        name: p.name,
        xuid: x.xuid,
        skin_url: skin.map(|s| s.url.clone()),
        skin_slim: skin.and_then(|s| s.variant.as_deref()).is_some_and(|v| v.eq_ignore_ascii_case("slim")),
        cape_url: p.capes.iter().find(|c| c.state == "ACTIVE").map(|c| c.url.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_parsing() {
        assert!(code_from_redirect("https://example.com/?code=x").is_none());
        let c = code_from_redirect("https://login.live.com/oauth20_desktop.srf?code=M.C5_BAY.2.U.abc%21def&lc=1045").unwrap().unwrap();
        assert_eq!(c, "M.C5_BAY.2.U.abc!def");
        let e = code_from_redirect("https://login.live.com/oauth20_desktop.srf?error=access_denied&error_description=The+user+denied").unwrap();
        assert!(e.unwrap_err().to_string().contains("The user denied"));
        // The page itself, before redirecting, has no query.
        assert!(code_from_redirect("https://login.live.com/oauth20_desktop.srf").is_none());
    }

    #[test]
    fn authorize_url_is_encoded() {
        let u = authorize_url();
        assert!(u.contains("scope=service%3A%3Auser.auth.xboxlive.com%3A%3AMBI_SSL"));
        assert!(u.contains("redirect_uri=https%3A%2F%2Flogin.live.com%2Foauth20_desktop.srf"));
    }
}
