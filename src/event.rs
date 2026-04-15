use crate::app::{
    App, AppMode, BookmarkItem, FocusPane, FinishState, SearchResult,
};
use crate::config::Config;
use crate::storage;
use crate::ui::filter_panel::{build_filter_items, toggle_at};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Events that drive the search engine.
#[derive(Debug)]
pub enum SearchEvent {
    ConfluenceResult(Result<Vec<crate::app::PageResult>>),
    JiraResult(Result<Vec<crate::app::IssueResult>>),
    GerritResult(Result<Vec<SearchResult>>),
    PreviewBody { url: String, body: String },
}

/// Handle a key event, mutate app state, and return whether a search should
/// be triggered (for incremental mode).
pub fn handle_key(
    app: &mut App,
    key: KeyEvent,
    config: &Config,
    storage_dir: &std::path::Path,
) -> HandleResult {
    // ── Overlays take priority ─────────────────────────────────────────
    if app.history_panel_open {
        return handle_history_panel(app, key, storage_dir);
    }
    if app.bookmark_panel_open {
        return handle_bookmark_panel(app, key, storage_dir);
    }
    if app.filter_panel_open {
        return handle_filter_panel(app, key);
    }
    if app.help_open {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                app.help_open = false;
            }
            _ => {}
        }
        return HandleResult::None;
    }

    // ── Global shortcuts ───────────────────────────────────────────────
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Char('q'))
        | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.should_quit = true;
            return HandleResult::None;
        }
        (KeyModifiers::NONE, KeyCode::Char('?')) => {
            app.help_open = true;
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            app.new_tab();
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            app.close_tab();
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Left) => {
            app.switch_tab_left();
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Right) => {
            app.switch_tab_right();
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
            let session = app.active_tab().to_session();
            if let Ok(dir) = storage::config_dir() {
                let mut sessions = storage::tabs::load(&dir).unwrap_or_default();
                storage::tabs::upsert(&mut sessions, session);
                let _ = storage::tabs::save(&dir, &sessions);
                app.status_msg.set("✅ タブを保存しました", 7);
            }
            return HandleResult::None;
        }
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.bookmark_panel_open = true;
            app.overlay_index = 0;
            app.focus = FocusPane::BookmarkPanel;
            return HandleResult::None;
        }
        (KeyModifiers::ALT, KeyCode::Char(c)) if c.is_ascii_digit() => {
            if let Some(n) = c.to_digit(10) {
                app.jump_to_tab(n as usize - 1);
            }
            return HandleResult::None;
        }
        _ => {}
    }

    // ── Focus-specific handling ────────────────────────────────────────
    match &app.focus {
        FocusPane::SearchBar => handle_search_bar(app, key, storage_dir),
        FocusPane::ResultList => handle_result_list(app, key, config, storage_dir),
        FocusPane::Preview => handle_preview(app, key),
        _ => HandleResult::None,
    }
}

// ─────────────────────────────────────────────
// Search bar
// ─────────────────────────────────────────────

fn handle_search_bar(
    app: &mut App,
    key: KeyEvent,
    _storage_dir: &std::path::Path,
) -> HandleResult {
    let tab = app.active_tab_mut();
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if !tab.search.input.is_empty() {
                let tag = tab.search.input.clone();
                tab.search.add_tag(tag);
                if matches!(app.mode, AppMode::Incremental) {
                    return HandleResult::TriggerSearch;
                }
            } else if tab.search.has_tags() {
                // Normal mode: fire search on Enter with empty input
                return HandleResult::TriggerSearch;
            }
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            app.history_panel_open = true;
            app.overlay_index = 0;
        }
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            if app.active_tab().search.input.is_empty() {
                app.active_tab_mut().search.delete_last_tag();
                if matches!(app.mode, AppMode::Incremental) {
                    return HandleResult::TriggerSearch;
                }
            } else {
                app.active_tab_mut().search.input.pop();
            }
        }
        (KeyModifiers::CONTROL, KeyCode::Char('m')) => {
            app.active_tab_mut().search.toggle_logic();
            if matches!(app.mode, AppMode::Incremental)
                && app.active_tab().search.has_tags()
            {
                return HandleResult::TriggerSearch;
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('/')) => {
            app.mode = match app.mode {
                AppMode::Incremental => AppMode::Normal,
                AppMode::Normal => AppMode::Incremental,
            };
        }
        (KeyModifiers::NONE, KeyCode::Tab) => {
            app.filter_panel_open = true;
            app.overlay_index = 0;
            app.focus = FocusPane::FilterPanel;
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.focus = FocusPane::ResultList;
        }
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            app.active_tab_mut().search.input.push(c);
            if matches!(app.mode, AppMode::Incremental) {
                return HandleResult::DebounceSearch;
            }
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// Result list
// ─────────────────────────────────────────────

fn handle_result_list(
    app: &mut App,
    key: KeyEvent,
    _config: &Config,
    storage_dir: &std::path::Path,
) -> HandleResult {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.active_tab().selected_index == 0 {
                app.focus = FocusPane::SearchBar;
            } else {
                app.active_tab_mut().move_up();
            }
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.active_tab_mut().move_down();
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if let Some(result) = app.active_tab().selected_result() {
                let url = result.url().to_string();
                open_url(&url);
            }
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            app.active_tab_mut().toggle_collapse_at_cursor();
        }
        (KeyModifiers::NONE, KeyCode::Char('y')) => {
            if let Some(result) = app.active_tab().selected_result() {
                let url = result.url().to_string();
                copy_to_clipboard(&url, app);
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('b')) => {
            if let Some(result) = app.active_tab().selected_result() {
                let url = result.url().to_string();
                let title = result.title();
                let source = match result {
                    SearchResult::Page(_) => "Confluence",
                    SearchResult::Issue(_) => "Jira",
                    _ => "Gerrit",
                }
                .to_string();
                let bm = BookmarkItem {
                    title,
                    url,
                    source,
                    added_at: crate::storage::history::new_entry(vec![], crate::app::SearchLogic::And)
                        .timestamp,
                };
                let added = crate::storage::bookmarks::toggle(&mut app.bookmarks, bm);
                if added {
                    app.status_msg.set("✅ ブックマークに追加しました", 7);
                } else {
                    app.status_msg.set("🗑 ブックマークを削除しました", 7);
                }
                let _ = crate::storage::bookmarks::save(storage_dir, &app.bookmarks);
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('p')) => {
            app.focus = FocusPane::Preview;
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// Preview
// ─────────────────────────────────────────────

fn handle_preview(app: &mut App, key: KeyEvent) -> HandleResult {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Up) => {
            app.active_tab_mut().preview_scroll =
                app.active_tab().preview_scroll.saturating_sub(1);
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            app.active_tab_mut().preview_scroll += 1;
        }
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Char('p')) => {
            app.focus = FocusPane::ResultList;
        }
        (KeyModifiers::NONE, KeyCode::Char('y')) => {
            if let Some(result) = app.active_tab().selected_result() {
                let url = result.url().to_string();
                copy_to_clipboard(&url, app);
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('b')) => {
            app.focus = FocusPane::ResultList;
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// Filter panel
// ─────────────────────────────────────────────

fn handle_filter_panel(app: &mut App, key: KeyEvent) -> HandleResult {
    let item_count = {
        let tab = app.active_tab();
        build_filter_items(&tab.filter, &app.options).len()
    };

    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) | (KeyModifiers::NONE, KeyCode::Tab) => {
            app.filter_panel_open = false;
            app.focus = FocusPane::SearchBar;
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.overlay_index > 0 {
                app.overlay_index -= 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            if app.overlay_index + 1 < item_count {
                app.overlay_index += 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Char(' ')) => {
            let idx = app.overlay_index;
            let opts = app.options.clone();
            toggle_at(idx, &mut app.active_tab_mut().filter, &opts);
            if matches!(app.mode, AppMode::Incremental)
                && app.active_tab().search.has_tags()
            {
                return HandleResult::TriggerSearch;
            }
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// History panel
// ─────────────────────────────────────────────

fn handle_history_panel(
    app: &mut App,
    key: KeyEvent,
    storage_dir: &std::path::Path,
) -> HandleResult {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            app.history_panel_open = false;
            app.focus = FocusPane::SearchBar;
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.overlay_index > 0 {
                app.overlay_index -= 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            if app.overlay_index + 1 < app.history.len() {
                app.overlay_index += 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if let Some(h) = app.history.get(app.overlay_index).cloned() {
                let tab = app.active_tab_mut();
                tab.search.tags = h.tags;
                tab.search.logic = h.logic;
                tab.search.input.clear();
                app.history_panel_open = false;
                app.focus = FocusPane::SearchBar;
                return HandleResult::TriggerSearch;
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('d')) => {
            crate::storage::history::remove_at(&mut app.history, app.overlay_index);
            if app.overlay_index > 0 && app.overlay_index >= app.history.len() {
                app.overlay_index -= 1;
            }
            let _ = crate::storage::history::save(storage_dir, &app.history);
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// Bookmark panel
// ─────────────────────────────────────────────

fn handle_bookmark_panel(
    app: &mut App,
    key: KeyEvent,
    storage_dir: &std::path::Path,
) -> HandleResult {
    match (key.modifiers, key.code) {
        (KeyModifiers::NONE, KeyCode::Esc) => {
            app.bookmark_panel_open = false;
            app.focus = FocusPane::ResultList;
        }
        (KeyModifiers::NONE, KeyCode::Up) => {
            if app.overlay_index > 0 {
                app.overlay_index -= 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Down) => {
            if app.overlay_index + 1 < app.bookmarks.len() {
                app.overlay_index += 1;
            }
        }
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if let Some(bm) = app.bookmarks.get(app.overlay_index) {
                open_url(&bm.url.clone());
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('y')) => {
            if let Some(bm) = app.bookmarks.get(app.overlay_index) {
                let url = bm.url.clone();
                copy_to_clipboard(&url, app);
            }
        }
        (KeyModifiers::NONE, KeyCode::Char('d')) => {
            crate::storage::bookmarks::remove_at(&mut app.bookmarks, app.overlay_index);
            if app.overlay_index > 0 && app.overlay_index >= app.bookmarks.len() {
                app.overlay_index -= 1;
            }
            let _ = crate::storage::bookmarks::save(storage_dir, &app.bookmarks);
        }
        _ => {}
    }
    HandleResult::None
}

// ─────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────

fn open_url(url: &str) {
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}

fn copy_to_clipboard(url: &str, app: &mut App) {
    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(url)) {
        Ok(_) => app.status_msg.set("✅ URLをコピーしました", 7),
        Err(_) => app.status_msg.set("❌ コピー失敗", 7),
    }
}

/// Apply a `SearchEvent` received from a spawned search task.
pub fn apply_search_event(app: &mut App, event: SearchEvent, _storage_dir: &std::path::Path) {
    let tab = app.active_tab_mut();
    match event {
        SearchEvent::ConfluenceResult(Ok(pages)) => {
            let count = pages.len();
            tab.groups[0].results = pages.into_iter().map(crate::app::SearchResult::Page).collect();
            tab.groups[0].status = crate::app::GroupStatus::Done(count);
        }
        SearchEvent::ConfluenceResult(Err(e)) => {
            tab.groups[0].status = crate::app::GroupStatus::Error(e.to_string());
        }
        SearchEvent::JiraResult(Ok(issues)) => {
            let count = issues.len();
            tab.groups[1].results = issues.into_iter().map(crate::app::SearchResult::Issue).collect();
            tab.groups[1].status = crate::app::GroupStatus::Done(count);
        }
        SearchEvent::JiraResult(Err(e)) => {
            tab.groups[1].status = crate::app::GroupStatus::Error(e.to_string());
        }
        SearchEvent::GerritResult(Ok(results)) => {
            let count = results.len();
            tab.groups[2].results = results;
            tab.groups[2].status = crate::app::GroupStatus::Done(count);
        }
        SearchEvent::GerritResult(Err(e)) => {
            tab.groups[2].status = crate::app::GroupStatus::Error(e.to_string());
        }
        SearchEvent::PreviewBody { url, body } => {
            for group in &mut tab.groups {
                for result in &mut group.results {
                    if result.url() == url {
                        match result {
                            crate::app::SearchResult::Page(p) => p.body = Some(body.clone()),
                            crate::app::SearchResult::Issue(i) => {
                                i.description = Some(body.clone())
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Check if all groups are done → animate finish
    let all_done = app
        .active_tab()
        .groups
        .iter()
        .all(|g| !matches!(g.status, crate::app::GroupStatus::Loading));
    if all_done {
        let has_error = app
            .active_tab()
            .groups
            .iter()
            .any(|g| matches!(g.status, crate::app::GroupStatus::Error(_)));
        app.travolta.finish(if has_error {
            FinishState::Error
        } else {
            FinishState::Success
        });
    }
}

/// Record a history entry after a search is triggered.
pub fn record_history(app: &mut App, storage_dir: &std::path::Path) {
    let tab = app.active_tab();
    if tab.search.tags.is_empty() {
        return;
    }
    let entry = crate::storage::history::new_entry(
        tab.search.tags.clone(),
        tab.search.logic.clone(),
    );
    crate::storage::history::push(&mut app.history, entry);
    let _ = crate::storage::history::save(storage_dir, &app.history);
}

/// What the key handler wants the event loop to do next.
#[derive(Debug, PartialEq)]
pub enum HandleResult {
    None,
    /// Trigger search immediately.
    TriggerSearch,
    /// Reset the debounce timer and trigger after 300ms.
    DebounceSearch,
}

// ─────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, SearchLogic};
    use crate::config::Config;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn dummy_config() -> Config {
        Config {
            confluence_url: "http://c.local".into(),
            confluence_user: "u".into(),
            confluence_token: "t".into(),
            confluence_default_spaces: vec![],
            jira_url: "http://j.local".into(),
            jira_user: "u".into(),
            jira_token: "t".into(),
            jira_default_projects: vec![],
            jira_default_statuses: vec![],
            gerrit_url: "http://g.local".into(),
            gerrit_user: "u".into(),
            gerrit_password: "p".into(),
            gerrit_default_repos: vec![],
        }
    }

    fn dummy_dir() -> PathBuf {
        std::env::temp_dir()
    }

    fn make_key(mods: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    #[test]
    fn test_ctrl_m_toggles_logic() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        assert_eq!(app.active_tab().search.logic, SearchLogic::And);
        handle_key(
            &mut app,
            make_key(KeyModifiers::CONTROL, KeyCode::Char('m')),
            &cfg,
            &dir,
        );
        assert_eq!(app.active_tab().search.logic, SearchLogic::Or);
    }

    #[test]
    fn test_enter_tags_input() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        app.active_tab_mut().search.input = "kubernetes".into();
        let result = handle_key(
            &mut app,
            make_key(KeyModifiers::NONE, KeyCode::Enter),
            &cfg,
            &dir,
        );
        assert_eq!(app.active_tab().search.tags, vec!["kubernetes"]);
        assert!(app.active_tab().search.input.is_empty());
        // Incremental mode triggers search after tagging
        assert_eq!(result, HandleResult::TriggerSearch);
    }

    #[test]
    fn test_backspace_deletes_last_tag() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        app.active_tab_mut().search.tags = vec!["k8s".into(), "deploy".into()];
        handle_key(
            &mut app,
            make_key(KeyModifiers::NONE, KeyCode::Backspace),
            &cfg,
            &dir,
        );
        assert_eq!(app.active_tab().search.tags, vec!["k8s"]);
    }

    #[test]
    fn test_ctrl_t_opens_new_tab() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        assert_eq!(app.tabs.len(), 1);
        handle_key(
            &mut app,
            make_key(KeyModifiers::CONTROL, KeyCode::Char('t')),
            &cfg,
            &dir,
        );
        assert_eq!(app.tabs.len(), 2);
    }

    #[test]
    fn test_quit_key() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        assert!(!app.should_quit);
        handle_key(
            &mut app,
            make_key(KeyModifiers::NONE, KeyCode::Char('q')),
            &cfg,
            &dir,
        );
        assert!(app.should_quit);
    }

    #[test]
    fn test_slash_toggles_mode() {
        let mut app = App::new();
        let cfg = dummy_config();
        let dir = dummy_dir();
        assert!(matches!(app.mode, AppMode::Incremental));
        handle_key(
            &mut app,
            make_key(KeyModifiers::NONE, KeyCode::Char('/')),
            &cfg,
            &dir,
        );
        assert!(matches!(app.mode, AppMode::Normal));
        handle_key(
            &mut app,
            make_key(KeyModifiers::NONE, KeyCode::Char('/')),
            &cfg,
            &dir,
        );
        assert!(matches!(app.mode, AppMode::Incremental));
    }

    #[test]
    fn test_alt_digit_switches_tab() {
        let mut app = App::new();
        app.new_tab();
        app.new_tab();
        let cfg = dummy_config();
        let dir = dummy_dir();
        // Switch to tab 2 (index 1)
        handle_key(
            &mut app,
            make_key(KeyModifiers::ALT, KeyCode::Char('2')),
            &cfg,
            &dir,
        );
        assert_eq!(app.active_tab, 1);
    }
}
