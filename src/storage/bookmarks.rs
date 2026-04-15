use crate::app::BookmarkItem;
use anyhow::Result;
use std::path::Path;

pub fn load(dir: &Path) -> Result<Vec<BookmarkItem>> {
    let path = dir.join("bookmarks.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)?;
    let items: Vec<BookmarkItem> = serde_json::from_str(&data)?;
    Ok(items)
}

pub fn save(dir: &Path, items: &[BookmarkItem]) -> Result<()> {
    let path = dir.join("bookmarks.json");
    let data = serde_json::to_string_pretty(items)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Toggle bookmark: adds if not present, removes if already present.
/// Returns `true` if added, `false` if removed.
pub fn toggle(items: &mut Vec<BookmarkItem>, item: BookmarkItem) -> bool {
    if let Some(pos) = items.iter().position(|b| b.url == item.url) {
        items.remove(pos);
        false
    } else {
        items.push(item);
        true
    }
}

pub fn remove_at(items: &mut Vec<BookmarkItem>, index: usize) {
    if index < items.len() {
        items.remove(index);
    }
}

pub fn is_bookmarked(items: &[BookmarkItem], url: &str) -> bool {
    items.iter().any(|b| b.url == url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_bookmark(title: &str, url: &str, source: &str) -> BookmarkItem {
        BookmarkItem {
            title: title.into(),
            url: url.into(),
            source: source.into(),
            added_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let items = vec![
            make_bookmark("Page 1", "https://conf.example.com/1", "Confluence"),
            make_bookmark("OPS-123", "https://jira.example.com/OPS-123", "Jira"),
        ];
        save(dir.path(), &items).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "Page 1");
        assert_eq!(loaded[1].source, "Jira");
    }

    #[test]
    fn test_load_missing_file() {
        let dir = TempDir::new().unwrap();
        let items = load(dir.path()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_toggle_add() {
        let mut items: Vec<BookmarkItem> = Vec::new();
        let bm = make_bookmark("Page 1", "https://example.com/1", "Confluence");
        let added = toggle(&mut items, bm);
        assert!(added);
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_toggle_remove() {
        let bm = make_bookmark("Page 1", "https://example.com/1", "Confluence");
        let mut items = vec![bm.clone()];
        let added = toggle(&mut items, bm);
        assert!(!added);
        assert!(items.is_empty());
    }

    #[test]
    fn test_is_bookmarked() {
        let items = vec![make_bookmark("P", "https://example.com/1", "Confluence")];
        assert!(is_bookmarked(&items, "https://example.com/1"));
        assert!(!is_bookmarked(&items, "https://example.com/2"));
    }

    #[test]
    fn test_remove_at() {
        let mut items = vec![
            make_bookmark("A", "https://a.com", "Confluence"),
            make_bookmark("B", "https://b.com", "Jira"),
        ];
        remove_at(&mut items, 0);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "B");
        // Out of bounds: no panic
        remove_at(&mut items, 99);
    }

    #[test]
    fn test_serialization_round_trip() {
        let dir = TempDir::new().unwrap();
        let items = vec![make_bookmark("Test", "https://test.com", "Gerrit")];
        save(dir.path(), &items).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded[0].title, "Test");
        assert_eq!(loaded[0].url, "https://test.com");
        assert_eq!(loaded[0].source, "Gerrit");
    }
}
