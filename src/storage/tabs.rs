use crate::app::TabSession;
use anyhow::Result;
use std::path::Path;

pub fn load(dir: &Path) -> Result<Vec<TabSession>> {
    let path = dir.join("tabs.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)?;
    let items: Vec<TabSession> = serde_json::from_str(&data)?;
    Ok(items)
}

pub fn save(dir: &Path, sessions: &[TabSession]) -> Result<()> {
    let path = dir.join("tabs.json");
    let data = serde_json::to_string_pretty(sessions)?;
    std::fs::write(&path, data)?;
    Ok(())
}

/// Upsert a session by id (replace existing or append).
pub fn upsert(sessions: &mut Vec<TabSession>, session: TabSession) {
    if let Some(pos) = sessions.iter().position(|s| s.id == session.id) {
        sessions[pos] = session;
    } else {
        sessions.push(session);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{FilterState, SearchLogic, SearchState};
    use tempfile::TempDir;

    fn make_session(id: usize, name: &str) -> TabSession {
        TabSession {
            id,
            name: name.into(),
            search: SearchState {
                tags: vec!["kubernetes".into()],
                input: String::new(),
                logic: SearchLogic::And,
            },
            filter: FilterState::default(),
        }
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let sessions = vec![
            make_session(1, "k8s調査"),
            make_session(2, "監視設定"),
        ];
        save(dir.path(), &sessions).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].name, "k8s調査");
        assert_eq!(loaded[1].id, 2);
        assert_eq!(loaded[0].search.tags, vec!["kubernetes"]);
    }

    #[test]
    fn test_load_missing_file() {
        let dir = TempDir::new().unwrap();
        let sessions = load(dir.path()).unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_upsert_new() {
        let mut sessions: Vec<TabSession> = Vec::new();
        upsert(&mut sessions, make_session(1, "Tab 1"));
        assert_eq!(sessions.len(), 1);
    }

    #[test]
    fn test_upsert_existing() {
        let mut sessions = vec![make_session(1, "Tab 1")];
        let updated = TabSession {
            id: 1,
            name: "Updated".into(),
            search: SearchState::default(),
            filter: FilterState::default(),
        };
        upsert(&mut sessions, updated);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "Updated");
    }

    #[test]
    fn test_round_trip_filter_state() {
        let dir = TempDir::new().unwrap();
        let mut session = make_session(1, "test");
        session.filter.spaces.insert("DS".into());
        session.filter.projects.insert("OPS".into());
        save(dir.path(), &[session]).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert!(loaded[0].filter.spaces.contains("DS"));
        assert!(loaded[0].filter.projects.contains("OPS"));
    }
}
