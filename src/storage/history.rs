use crate::app::{HistoryItem, SearchLogic};
use anyhow::Result;
use std::path::Path;

const MAX_HISTORY: usize = 100;

pub fn load(dir: &Path) -> Result<Vec<HistoryItem>> {
    let path = dir.join("history.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)?;
    let items: Vec<HistoryItem> = serde_json::from_str(&data)?;
    Ok(items)
}

pub fn save(dir: &Path, items: &[HistoryItem]) -> Result<()> {
    let path = dir.join("history.json");
    let data = serde_json::to_string_pretty(items)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Add a new history entry, deduplicating by (tags, logic).
/// Keeps newest at index 0. Trims to MAX_HISTORY.
pub fn push(items: &mut Vec<HistoryItem>, entry: HistoryItem) {
    // Remove existing entry with same tags+logic
    items.retain(|h| !(h.tags == entry.tags && h.logic == entry.logic));
    items.insert(0, entry);
    items.truncate(MAX_HISTORY);
}

pub fn remove_at(items: &mut Vec<HistoryItem>, index: usize) {
    if index < items.len() {
        items.remove(index);
    }
}

fn now_utc() -> String {
    // chrono is not in the deps; use a simple approach
    // In production this returns a timestamp; in tests we use a fixed value.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Format as ISO8601-ish (simplified)
    format_unix_ts(secs)
}

fn format_unix_ts(secs: u64) -> String {
    // Very simple formatter: YYYY-MM-DDTHH:MM:SSZ
    let s = secs;
    let sec = s % 60;
    let min = (s / 60) % 60;
    let hr = (s / 3600) % 24;
    let days = s / 86400;
    // Days since epoch → date (Gregorian, approximate)
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{hr:02}:{min:02}:{sec:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Gregorian calendar approximation
    let mut y = 1970u64;
    loop {
        let leap = is_leap(y);
        let dy = if leap { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        y += 1;
    }
    let leap = is_leap(y);
    let months = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 1u64;
    for dm in months {
        if days < dm {
            break;
        }
        days -= dm;
        mo += 1;
    }
    (y, mo, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

pub fn new_entry(tags: Vec<String>, logic: SearchLogic) -> HistoryItem {
    HistoryItem {
        tags,
        logic,
        timestamp: now_utc(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::SearchLogic;
    use std::fs;
    use tempfile::TempDir;

    fn make_item(tags: &[&str], logic: SearchLogic) -> HistoryItem {
        HistoryItem {
            tags: tags.iter().map(|s| s.to_string()).collect(),
            logic,
            timestamp: "2026-01-01T00:00:00Z".into(),
        }
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let items = vec![
            make_item(&["kubernetes", "deployment"], SearchLogic::And),
            make_item(&["monitoring"], SearchLogic::Or),
        ];
        save(dir.path(), &items).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].tags, vec!["kubernetes", "deployment"]);
        assert_eq!(loaded[1].logic, SearchLogic::Or);
    }

    #[test]
    fn test_load_missing_file() {
        let dir = TempDir::new().unwrap();
        let items = load(dir.path()).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_push_deduplication() {
        let mut items: Vec<HistoryItem> = vec![make_item(&["k8s"], SearchLogic::And)];
        let new = make_item(&["k8s"], SearchLogic::And);
        push(&mut items, new);
        // Dedup: still 1 item, moved to front
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tags, vec!["k8s"]);
    }

    #[test]
    fn test_push_max_limit() {
        let mut items: Vec<HistoryItem> = (0..100)
            .map(|i| make_item(&[&format!("tag{i}")], SearchLogic::And))
            .collect();
        let new = make_item(&["newest"], SearchLogic::And);
        push(&mut items, new);
        assert_eq!(items.len(), 100);
        assert_eq!(items[0].tags, vec!["newest"]);
    }

    #[test]
    fn test_push_newest_first() {
        let mut items: Vec<HistoryItem> = vec![make_item(&["old"], SearchLogic::And)];
        push(&mut items, make_item(&["new"], SearchLogic::Or));
        assert_eq!(items[0].tags, vec!["new"]);
        assert_eq!(items[1].tags, vec!["old"]);
    }

    #[test]
    fn test_remove_at() {
        let mut items = vec![
            make_item(&["a"], SearchLogic::And),
            make_item(&["b"], SearchLogic::And),
        ];
        remove_at(&mut items, 0);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].tags, vec!["b"]);
        // Out of bounds: no panic
        remove_at(&mut items, 99);
    }

    #[test]
    fn test_format_unix_ts() {
        // Unix epoch = 1970-01-01T00:00:00Z
        assert_eq!(format_unix_ts(0), "1970-01-01T00:00:00Z");
    }
}
