//! Release notes shipped with this build: offline, with no tracking or remote HTML.
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseNote {
    pub version: String,
    pub date: String,
    pub title: String,
    pub summary: String,
    pub sections: Vec<NewsSection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewsSection {
    pub title: String,
    pub items: Vec<String>,
}

pub fn list() -> crate::Result<Vec<ReleaseNote>> {
    Ok(serde_json::from_str(include_str!("../../../resources/news.json"))?)
}

#[cfg(test)]
mod tests {
    #[test]
    fn bundled_news_matches_build_and_has_valid_content() {
        let notes = super::list().unwrap();
        assert_eq!(notes[0].version, crate::LAUNCHER_VERSION);
        let mut versions = std::collections::HashSet::new();
        let mut previous = "9999-12-31";
        for note in &notes {
            assert!(versions.insert(&note.version));
            chrono::NaiveDate::parse_from_str(&note.date, "%Y-%m-%d").unwrap();
            assert!(note.date.as_str() <= previous);
            previous = &note.date;
            assert!(!note.title.trim().is_empty());
            assert!(!note.summary.trim().is_empty());
            assert!(!note.sections.is_empty());
            for section in &note.sections {
                assert!(!section.title.trim().is_empty());
                assert!(!section.items.is_empty());
                assert!(section.items.iter().all(|s| !s.trim().is_empty()));
            }
        }
    }
}
