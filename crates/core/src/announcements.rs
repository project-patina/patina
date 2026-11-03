//! Small, read-only GitHub feed. An unset source never makes a network request.
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{bail, Ctx, Result};

const MAX_ITEMS: usize = 3;
const MAX_INDEX: usize = 32 * 1024;
const MAX_MARKDOWN: usize = 96 * 1024;
const CACHE_LIMIT: usize = 384 * 1024;
const CACHE_TTL: i64 = 15 * 60;

#[derive(Debug, Deserialize)]
struct Source {
    repository: Option<String>,
    branch: String,
    directory: String,
}

#[derive(Debug, Deserialize)]
struct Localized {
    title: String,
    summary: String,
    markdown: String,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    date: String,
    image: String,
    pl: Localized,
    en: Localized,
}

impl Entry {
    fn text(&self, language: &str) -> &Localized {
        if language == "pl" { &self.pl } else { &self.en }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: String,
    pub date: String,
    pub title: String,
    pub summary: String,
    pub image: String,
    pub markdown: String,
}

#[derive(Debug, Serialize)]
pub struct Feed {
    pub items: Vec<Announcement>,
    pub source: &'static str,
}

#[derive(Serialize, Deserialize)]
struct Cache {
    source: String,
    fetched_at: i64,
    items: Vec<Announcement>,
}

fn segment(value: &str) -> bool {
    !value.is_empty() && value != "." && value != ".." && value.len() <= 160
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

fn path(value: &str) -> bool {
    value.len() <= 512 && value.split('/').all(segment)
}

impl Source {
    fn base(&self) -> Result<Option<String>> {
        let Some(repository) = &self.repository else { return Ok(None) };
        let parts: Vec<_> = repository.split('/').collect();
        if parts.len() != 2 || !parts.iter().all(|s| segment(s)) || !segment(&self.branch) || !path(&self.directory) {
            bail!("Nieprawidłowe źródło wiadomości GitHub", "Invalid GitHub announcements source");
        }
        Ok(Some(format!("https://raw.githubusercontent.com/{repository}/{}/{}/", self.branch, self.directory)))
    }
}

fn valid_text(text: &Localized) -> bool {
    !text.title.trim().is_empty() && text.title.len() <= 140 && text.summary.len() <= 300 && path(&text.markdown) && text.markdown.ends_with(".md")
}

fn parse_index(bytes: &[u8]) -> Result<Vec<Entry>> {
    if bytes.len() > MAX_INDEX { bail!("Indeks wiadomości jest zbyt duży", "The announcements index is too large"); }
    let mut entries: Vec<Entry> = serde_json::from_slice(bytes)?;
    if entries.len() > 30 { bail!("Zbyt wiele wiadomości w indeksie", "Too many entries in the announcements index"); }
    let mut ids = std::collections::HashSet::new();
    for entry in &entries {
        let dated = entry.date.len() == 10 && chrono::NaiveDate::parse_from_str(&entry.date, "%Y-%m-%d").is_ok();
        let image = path(&entry.image) && [".png", ".jpg", ".jpeg", ".webp", ".svg"].iter().any(|ext| entry.image.ends_with(ext));
        if !segment(&entry.id) || !ids.insert(&entry.id) || !dated || !image || ![&entry.pl, &entry.en].into_iter().all(valid_text) {
            bail!("Nieprawidłowy wpis wiadomości", "Invalid announcement entry");
        }
    }
    entries.sort_by(|a, b| b.date.cmp(&a.date));
    entries.truncate(MAX_ITEMS);
    Ok(entries)
}

fn bundled(language: &str) -> Result<Feed> {
    let entries = parse_index(include_bytes!("../../../resources/announcements/index.json"))?;
    let mut items = Vec::new();
    for entry in entries {
        let (image, markdown) = match (entry.id.as_str(), language) {
            ("first-version", "pl") => ("/announcements/first-version.png", include_str!("../../../resources/announcements/first-version.pl.md")),
            ("first-version", _) => ("/announcements/first-version.png", include_str!("../../../resources/announcements/first-version.en.md")),
            _ => bail!("Brak treści dołączonej wiadomości", "A bundled announcement has no content"),
        };
        let text = entry.text(language);
        items.push(Announcement { id: entry.id.clone(), date: entry.date.clone(), title: text.title.clone(), summary: text.summary.clone(), image: image.into(), markdown: markdown.into() });
    }
    Ok(Feed { items, source: "bundled" })
}

async fn bounded_get(http: &reqwest::Client, url: &str, limit: usize) -> Result<Vec<u8>> {
    let mut response = http.get(url).timeout(Duration::from_secs(5)).send().await?.error_for_status()?;
    if response.content_length().is_some_and(|n| n > limit as u64) { bail!("Wiadomość jest zbyt duża", "The announcement is too large"); }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > limit { bail!("Wiadomość jest zbyt duża", "The announcement is too large"); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn remote(base: &str, language: &str) -> Result<Vec<Announcement>> {
    // Feed content cannot redirect requests to arbitrary services or use account tokens.
    let http = reqwest::Client::builder().user_agent(format!("Patina/{}", crate::LAUNCHER_VERSION))
        .connect_timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::none()).build()?;
    let entries = parse_index(&bounded_get(&http, &format!("{base}index.json"), MAX_INDEX).await?)?;
    let mut items = Vec::new();
    for entry in &entries {
        let text = entry.text(language);
        let bytes = bounded_get(&http, &format!("{base}{}", text.markdown), MAX_MARKDOWN).await?;
        let markdown = String::from_utf8(bytes).map_err(|_| crate::Error::msg(crate::lang::tr("Wiadomość nie jest w UTF-8", "The announcement is not UTF-8")))?;
        items.push(Announcement { id: entry.id.clone(), date: entry.date.clone(), title: text.title.clone(), summary: text.summary.clone(), image: format!("{base}{}", entry.image), markdown });
    }
    Ok(items)
}

/// The feed in `language` (`pl`; anything else means English).
pub async fn list(ctx: &Ctx, language: &str) -> Result<Feed> {
    let language = if language == "pl" { "pl" } else { "en" };
    let source: Source = serde_json::from_str(include_str!("../../../resources/announcements-source.json"))?;
    let Some(base) = source.base()? else { return bundled(language) };
    let cache_path = ctx.paths.root.join(format!("cache/announcements-{language}.json"));
    let now = chrono::Utc::now().timestamp();
    let cached = std::fs::metadata(&cache_path).ok().filter(|m| m.len() <= CACHE_LIMIT as u64)
        .and_then(|_| std::fs::read(&cache_path).ok()).and_then(|bytes| serde_json::from_slice::<Cache>(&bytes).ok())
        .filter(|c| c.source == base && c.items.len() <= MAX_ITEMS
            && c.items.iter().all(|i| i.markdown.len() <= MAX_MARKDOWN && i.image.starts_with(&base)));
    if let Some(c) = &cached {
        if (0..CACHE_TTL).contains(&(now - c.fetched_at)) { return Ok(Feed { items: c.items.clone(), source: "cache" }); }
    }
    match tokio::time::timeout(Duration::from_secs(8), remote(&base, language)).await {
        Ok(Ok(items)) => {
            let cache = Cache { source: base, fetched_at: now, items: items.clone() };
            if let Ok(bytes) = serde_json::to_vec(&cache) { let _ = crate::paths::write_atomic(&cache_path, &bytes); }
            Ok(Feed { items, source: "github" })
        }
        failed => {
            log::warn!("announcements: {}", failed.map_or_else(|e| e.to_string(), |r| r.err().map_or_else(String::new, |e| e.to_string())));
            match cached {
                Some(c) => Ok(Feed { items: c.items, source: "cache" }),
                None => bundled(language),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn serve(responses: Vec<(&'static str, String)>) -> (String, std::thread::JoinHandle<()>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            for (path, body) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                let mut request = [0; 4096];
                let size = stream.read(&mut request).unwrap();
                assert!(String::from_utf8_lossy(&request[..size]).starts_with(&format!("GET /{path} ")));
                // No Content-Length: exercise the streamed limit, too.
                write!(stream, "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{body}").unwrap();
            }
        });
        (format!("http://{address}/"), server)
    }

    #[test]
    fn bundled_feed_is_the_first_version_in_both_languages() {
        for (language, title) in [("pl", "Patina: pierwsza wersja"), ("en", "Patina: first version")] {
            let feed = bundled(language).unwrap();
            assert_eq!(feed.items.len(), 1);
            let item = &feed.items[0];
            assert_eq!(item.id, "first-version");
            assert_eq!(item.title, title);
            assert_eq!(item.image, "/announcements/first-version.png");
            assert!(item.markdown.starts_with(&format!("# {title}")));
        }
    }

    #[test]
    fn source_points_at_the_news_repository_and_path_validation_rejects_escape() {
        let source: Source = serde_json::from_str(include_str!("../../../resources/announcements-source.json")).unwrap();
        assert_eq!(source.base().unwrap().unwrap(), "https://raw.githubusercontent.com/project-patina/patina-news/main/updates/");
        for bad in ["../secret.md", "/token.md", "https://host/file.md", "x%2f..", "x\\file.md", "a//b"] { assert!(!path(bad), "{bad}"); }
        assert!(Source { repository: None, branch: "main".into(), directory: "updates".into() }.base().unwrap().is_none());
    }

    #[test]
    fn rejects_duplicates_invalid_dates_urls_missing_languages_and_oversized_index() {
        let valid = include_str!("../../../resources/announcements/index.json");
        assert!(parse_index(valid.as_bytes()).is_ok());
        let twice = format!("[{0},{0}]", valid.trim().trim_start_matches('[').trim_end_matches(']'));
        for bad in [twice, valid.replace("2026-11-01", "2026-99-99"), valid.replace("first-version.pl.md", "../first-version.pl.md"), valid.replace("images/first-version.png", "https://example.com/pixel.png"), valid.replace("\"en\":", "\"de\":")] {
            assert!(parse_index(bad.as_bytes()).is_err());
        }
        assert!(parse_index(&vec![b' '; MAX_INDEX + 1]).is_err());
    }

    #[tokio::test]
    async fn downloads_index_and_only_the_requested_language() {
        let index = include_str!("../../../resources/announcements/index.json");
        let (base, server) = serve(vec![("index.json", index.into()), ("first-version.en.md", "# Hello from GitHub\n\nA **real** message.".into())]);
        let items = remote(&base, "en").await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Patina: first version");
        assert!(items[0].markdown.contains("**real**"));
        assert_eq!(items[0].image, format!("{base}images/first-version.png"));
        server.join().unwrap();
        let (base, server) = serve(vec![("index.json", index.into()), ("first-version.pl.md", "# Witaj".into())]);
        assert_eq!(remote(&base, "pl").await.unwrap()[0].title, "Patina: pierwsza wersja");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn streamed_body_without_length_cannot_exceed_the_limit() {
        let (base, server) = serve(vec![("large.md", "x".repeat(100))]);
        let http = reqwest::Client::builder().no_proxy().build().unwrap();
        assert!(bounded_get(&http, &format!("{base}large.md"), 10).await.is_err());
        server.join().unwrap();
    }
}
