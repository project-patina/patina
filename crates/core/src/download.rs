use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::stream::{self, StreamExt};
use sha1::{Digest, Sha1};
use tokio::io::AsyncWriteExt;
use tokio_util::sync::CancellationToken;

use crate::progress::Reporter;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct DlItem {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub executable: bool,
    /// Hash an existing file before trusting it. Assets are content-addressed by
    /// their hash already, so a size check is enough for them.
    pub verify_existing: bool,
}

impl DlItem {
    pub fn new(url: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self { url: url.into(), path: path.into(), sha1: None, size: None, executable: false, verify_existing: true }
    }
    pub fn sha1(mut self, sha1: Option<String>) -> Self {
        self.sha1 = sha1.filter(|s| !s.is_empty());
        self
    }
    pub fn size(mut self, size: Option<u64>) -> Self {
        self.size = size.filter(|s| *s > 0);
        self
    }
}

pub fn sha1_file(path: &Path) -> std::io::Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut h = Sha1::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

/// Whether the file on disk already matches what we would download.
fn is_up_to_date(item: &DlItem) -> bool {
    let Ok(meta) = std::fs::metadata(&item.path) else { return false };
    if !meta.is_file() {
        return false;
    }
    if let Some(size) = item.size {
        if meta.len() != size {
            return false;
        }
    }
    match (&item.sha1, item.verify_existing) {
        (Some(expected), true) => sha1_file(&item.path).map(|h| h.eq_ignore_ascii_case(expected)).unwrap_or(false),
        _ => true,
    }
}

async fn fetch_one(http: &reqwest::Client, item: &DlItem, cancel: &CancellationToken) -> Result<()> {
    if let Some(p) = item.path.parent() {
        tokio::fs::create_dir_all(p).await?;
    }
    // Shared libraries can be downloaded by several instances at once.
    let part = item.path.with_extension(format!("{}.part", uuid::Uuid::new_v4().simple()));
    let result: Result<()> = async {
        let mut resp = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(Error::Cancelled),
            response = http.get(&item.url).send() => response?.error_for_status()?,
        };
        let mut file = tokio::fs::File::create(&part).await?;
        let mut h = Sha1::new();
        let mut received = 0u64;
        loop {
            let chunk = tokio::select! {
                biased;
                _ = cancel.cancelled() => return Err(Error::Cancelled),
                c = resp.chunk() => c?,
            };
            let Some(chunk) = chunk else { break };
            received += chunk.len() as u64;
            h.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        drop(file);
        if let Some(expected) = item.size {
            if received != expected {
                return Err(Error::msg(crate::tr!("Niepełny plik {} (oczekiwano {expected} bajtów, pobrano {received})", "Incomplete file {} (expected {expected} bytes, got {received})", item.url)));
            }
        }
        if let Some(expected) = &item.sha1 {
            let got = hex::encode(h.finalize());
            if !got.eq_ignore_ascii_case(expected) {
                return Err(Error::msg(crate::tr!("Suma SHA1 się nie zgadza dla {} (oczekiwano {expected}, jest {got})", "SHA1 mismatch for {} (expected {expected}, got {got})", item.url)));
            }
        }
        #[cfg(unix)]
        if item.executable {
            use std::os::unix::fs::PermissionsExt;
            tokio::fs::set_permissions(&part, std::fs::Permissions::from_mode(0o755)).await?;
        }
        if cancel.is_cancelled() { return Err(Error::Cancelled); }
        tokio::fs::rename(&part, &item.path).await?;
        Ok(())
    }.await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&part).await;
    }
    result
}

async fn fetch_with_retry(http: &reqwest::Client, item: &DlItem, cancel: &CancellationToken) -> Result<()> {
    let mut last = None;
    for attempt in 0..4u32 {
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if attempt > 0 {
            tokio::select! {
                _ = cancel.cancelled() => return Err(Error::Cancelled),
                _ = tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt - 1))) => {},
            }
        }
        match fetch_one(http, item, cancel).await {
            Ok(()) => return Ok(()),
            Err(Error::Cancelled) => return Err(Error::Cancelled),
            Err(e) => {
                log::warn!("download attempt {} failed for {}: {e}", attempt + 1, item.url);
                last = Some(e);
            }
        }
    }
    Err(last.unwrap())
}

/// Download everything that is missing or stale, `concurrency` files at a time.
pub async fn download_all(
    http: &reqwest::Client,
    items: Vec<DlItem>,
    concurrency: usize,
    reporter: &Reporter,
    stage: &str,
    label: &str,
    cancel: &CancellationToken,
) -> Result<()> {
    // Checking hashes is CPU work; do it off the async threads.
    let pending: Vec<DlItem> = tokio::task::spawn_blocking(move || items.into_iter().filter(|i| !is_up_to_date(i)).collect())
        .await
        .map_err(|e| Error::msg(e.to_string()))?;
    let total = pending.len() as u64;
    if total == 0 {
        return Ok(());
    }
    log::info!("{stage}: downloading {total} files");
    let done = Arc::new(AtomicU64::new(0));
    let last_emit = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(1)));
    reporter.report(stage, 0, total, format!("{label} ({n}/{total})", n = 0));

    // Every future owns its inputs; borrowing ones are not `Send` enough for Tauri commands.
    let futures = pending.into_iter().map(|item| {
        let (http, cancel, reporter) = (http.clone(), cancel.clone(), reporter.clone());
        let (done, last_emit) = (done.clone(), last_emit.clone());
        let (stage, label) = (stage.to_string(), label.to_string());
        async move {
            let r = fetch_with_retry(&http, &item, &cancel).await;
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            let emit = {
                let mut le = last_emit.lock().unwrap();
                let due = n == total || le.elapsed() > Duration::from_millis(80);
                if due {
                    *le = Instant::now();
                }
                due
            };
            if emit {
                reporter.report(&stage, n, total, format!("{label} ({n}/{total})"));
            }
            r
        }
    });
    let results: Vec<Result<()>> = stream::iter(futures).buffer_unordered(concurrency.max(1)).collect().await;

    if cancel.is_cancelled() {
        return Err(Error::Cancelled);
    }
    let errors: Vec<Error> = results.into_iter().filter_map(|r| r.err()).collect();
    if let Some(first) = errors.into_iter().next() {
        return Err(first);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn serve(body: &'static str) -> (String, std::thread::JoinHandle<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let thread = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut request = [0; 2048];
            let _ = stream.read(&mut request);
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        });
        (format!("http://{addr}/file"), thread)
    }

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("patina-download-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[tokio::test]
    async fn wrong_size_preserves_existing_file_and_cleans_partial() {
        let (url, server) = serve("short");
        let dir = temp_dir();
        let path = dir.join("game.jar");
        std::fs::write(&path, "original").unwrap();
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let result = fetch_one(&client, &DlItem::new(url, &path).size(Some(10)), &CancellationToken::new()).await;
        assert!(result.is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        server.join().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn checksum_failure_does_not_publish_download() {
        let (url, server) = serve("corrupt");
        let dir = temp_dir();
        let path = dir.join("game.jar");
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let item = DlItem::new(url, &path).sha1(Some("0000000000000000000000000000000000000000".into()));
        assert!(fetch_one(&client, &item, &CancellationToken::new()).await.is_err());
        assert!(!path.exists());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        server.join().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn successful_download_checks_size_and_publishes_complete_file() {
        let (url, server) = serve("complete");
        let dir = temp_dir();
        let path = dir.join("game.jar");
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        fetch_one(&client, &DlItem::new(url, &path).size(Some(8)), &CancellationToken::new()).await.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "complete");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        server.join().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn cancellation_interrupts_waiting_for_response_headers() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (accepted_tx, accepted_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let _ = accepted_tx.send(());
            let _ = release_rx.recv_timeout(Duration::from_secs(3));
            drop(stream);
        });
        let dir = temp_dir();
        let cancel = CancellationToken::new();
        let signal = cancel.clone();
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let item = DlItem::new(format!("http://{addr}/file"), dir.join("game.jar"));
        let operation = fetch_one(&client, &item, &cancel);
        let cancelled = async move { accepted_rx.await.unwrap(); signal.cancel(); };
        let (result, ()) = tokio::time::timeout(Duration::from_secs(2), async { tokio::join!(operation, cancelled) }).await.unwrap();
        assert!(matches!(result, Err(Error::Cancelled)));
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let _ = release_tx.send(());
        server.join().unwrap();
        std::fs::remove_dir_all(dir).unwrap();
    }
}
