use std::time::Duration;

use serde::de::DeserializeOwned;

use crate::Result;

pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(format!("{}-Launcher/{}", crate::LAUNCHER_NAME, crate::LAUNCHER_VERSION))
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(60))
        .build()?)
}

/// GET + parse JSON, retrying transient failures.
pub async fn get_json<T: DeserializeOwned>(http: &reqwest::Client, url: &str) -> Result<T> {
    let bytes = get_bytes(http, url).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn get_bytes(http: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let mut last = None;
    for attempt in 0..3u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(400 * 2u64.pow(attempt))).await;
        }
        match http.get(url).send().await.and_then(|r| r.error_for_status()) {
            Ok(r) => match r.bytes().await {
                Ok(b) => return Ok(b.to_vec()),
                Err(e) => last = Some(e),
            },
            // 4xx will not get better on retry.
            Err(e) if e.status().is_some_and(|s| s.is_client_error()) => return Err(e.into()),
            Err(e) => last = Some(e),
        }
    }
    Err(last.expect("at least one attempt").into())
}
