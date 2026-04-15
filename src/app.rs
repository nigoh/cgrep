use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ────────────────────────────────────────────
// Enums
// ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Search fires automatically 300ms after tag/filter change.
    Incremental,
    /// Search fires only on Enter with empty input.
    Normal,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Incremental
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchLogic {
    And,
    Or,
}

impl Default for SearchLogic {
    fn default() -> Self {
        SearchLogic::And
    }
}

impl SearchLogic {
    pub fn toggle(&self) -> Self {
        match self {
            SearchLogic::And => SearchLogic::Or,
            SearchLogic::Or => SearchLogic::And,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SearchLogic::And => "AND",
            SearchLogic::Or => "OR",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FocusPane {
    SearchBar,
    ResultList,
    Preview,
    FilterPanel,
    BookmarkPanel,
    HistoryPanel,
}

impl Default for FocusPane {
    fn default() -> Self {
        FocusPane::SearchBar
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SourceKind {
    Confluence,
    Jira,
    Gerrit,
}

impl SourceKind {
    pub fn label(&self) -> &'static str {
        match self {
            SourceKind::Confluence => "Confluence",
            SourceKind::Jira => "Jira",
            SourceKind::Gerrit => "Gerrit",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GroupStatus {
    Loading,
    Done(usize),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FinishState {
    Success,
    Error,
}

// ────────────────────────────────────────────
// Search result types
// ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PageResult {
    pub id: String,
    pub title: String,
    pub space_key: String,
    pub url: String,
    pub last_modified: String,
    /// Fetched lazily on selection.
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IssueResult {
    pub key: String,
    pub summary: String,
    pub project: String,
    pub status: String,
    pub url: String,
    /// Fetched lazily on selection.
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommitResult {
    pub change_id: String,
    pub subject: String,
    pub repo: String,
    pub owner: String,
    pub created: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct TagResult {
    pub name: String,
    pub repo: String,
    pub revision: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Page(PageResult),
    Issue(IssueResult),
    Commit(CommitResult),
    Tag(TagResult),
}

impl SearchResult {
    pub fn url(&self) -> &str {
        match self {
            SearchResult::Page(p) => &p.url,
            SearchResult::Issue(i) => &i.url,
            SearchResult::Commit(c) => &c.url,
            SearchResult::Tag(t) => &t.url,
        }
    }

    pub fn title(&self) -> String {
        match self {
            SearchResult::Page(p) => p.title.clone(),
            SearchResult::Issue(i) => format!("{}: {}", i.key, i.summary),
            SearchResult::Commit(c) => format!("[COMMIT] {}: {}", c.repo, c.subject),
            SearchResult::Tag(t) => format!("[TAG] {}: {}", t.repo, t.name),
        }
    }

    pub fn source_icon(&self) -> &'static str {
        match self {
            SearchResult::Page(_) => "[PAGE]",
            SearchResult::Issue(_) => "[JIRA]",
            SearchResult::Commit(_) => "[COMMIT]",
            SearchResult::Tag(_) => "[TAG]",
        }
    }
}

// ────────────────────────────────────────────
// Result group
// ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ResultGroup {
    pub source: SourceKind,
    pub results: Vec<SearchResult>,
    pub collapsed: bool,
    pub status: GroupStatus,
}

impl ResultGroup {
    pub fn new(source: SourceKind) -> Self {
        Self {
            source,
            results: Vec::new(),
            collapsed: false,
            status: GroupStatus::Loading,
        }
    }

    pub fn item_count(&self) -> usize {
        if self.collapsed {
            0
        } else {
            self.results.len()
        }
    }
}

// ────────────────────────────────────────────
// Search and filter state
// ────────────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchState {
    pub tags: Vec<String>,
    pub input: String,
    pub logic: SearchLogic,
}

impl SearchState {
    pub fn add_tag(&mut self, tag: String) {
        let tag = tag.trim().to_string();
        if !tag.is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self.input.clear();
    }

    pub fn delete_last_tag(&mut self) {
        self.tags.pop();
    }

    pub fn toggle_logic(&mut self) {
        self.logic = self.logic.toggle();
    }

    pub fn has_tags(&self) -> bool {
        !self.tags.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterState {
    pub confluence_enabled: bool,
    pub jira_enabled: bool,
    pub gerrit_enabled: bool,
    pub spaces: HashSet<String>,
    pub projects: HashSet<String>,
    pub statuses: HashSet<String>,
    pub repos: HashSet<String>,
    pub gerrit_commits: bool,
    pub gerrit_tags: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            confluence_enabled: true,
            jira_enabled: true,
            gerrit_enabled: true,
            spaces: HashSet::new(),
            projects: HashSet::new(),
            statuses: HashSet::new(),
            repos: HashSet::new(),
            gerrit_commits: true,
            gerrit_tags: true,
        }
    }
}

// ────────────────────────────────────────────
// Tab
// ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabSession {
    pub id: usize,
    pub name: String,
    pub search: SearchState,
    pub filter: FilterState,
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: usize,
    pub name: String,
    pub search: SearchState,
    pub filter: FilterState,
    pub groups: Vec<ResultGroup>,
    /// Flat index across all visible items (headers + results).
    pub selected_index: usize,
    pub preview_scroll: u16,
}

impl Tab {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: format!("Tab {id}"),
            search: SearchState::default(),
            filter: FilterState::default(),
            groups: vec![
                ResultGroup::new(SourceKind::Confluence),
                ResultGroup::new(SourceKind::Jira),
                ResultGroup::new(SourceKind::Gerrit),
            ],
            selected_index: 0,
            preview_scroll: 0,
        }
    }

    pub fn to_session(&self) -> TabSession {
        TabSession {
            id: self.id,
            name: self.name.clone(),
            search: self.search.clone(),
            filter: self.filter.clone(),
        }
    }

    /// Returns the currently selected SearchResult, if any.
    pub fn selected_result(&self) -> Option<&SearchResult> {
        let mut flat_idx = 0usize;
        for group in &self.groups {
            // header counts as one item
            if flat_idx == self.selected_index {
                return None; // header selected
            }
            flat_idx += 1;
            if !group.collapsed {
                for result in &group.results {
                    if flat_idx == self.selected_index {
                        return Some(result);
                    }
                    flat_idx += 1;
                }
            }
        }
        None
    }

    /// Total number of visible flat items (headers + non-collapsed results).
    pub fn visible_item_count(&self) -> usize {
        self.groups.iter().fold(0, |acc, g| {
            acc + 1 + if g.collapsed { 0 } else { g.results.len() }
        })
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.preview_scroll = 0;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.visible_item_count().saturating_sub(1);
        if self.selected_index < max {
            self.selected_index += 1;
            self.preview_scroll = 0;
        }
    }

    pub fn toggle_collapse_at_cursor(&mut self) {
        let mut flat_idx = 0usize;
        for group in &mut self.groups {
            if flat_idx == self.selected_index {
                group.collapsed = !group.collapsed;
                return;
            }
            flat_idx += 1;
            if !group.collapsed {
                flat_idx += group.results.len();
            }
        }
    }
}

// ────────────────────────────────────────────
// Travolta animation
// ────────────────────────────────────────────

pub const TRAVOLTA_FRAMES: &[&str] = &[
    "  o  \n /|>\n/ \\ ",
    " \\o/ \n  |  \n/ \\ ",
    "  o/ \n <|\\ \n/ \\ ",
    " \\o  \n  |  \n/ \\ ",
    "  o  \n \\|  \n/ \\ ",
    "  o  \n /|>\n/ \\ ",
    "  o  \n <|\\ \n/ \\ ",
    "  o  \n \\|/ \n/ \\ ",
];
pub const TRAVOLTA_SUCCESS: &str = " \u{2728}o\u{2728} ";
pub const TRAVOLTA_ERROR: &str = " x_x  ";

#[derive(Debug, Clone, Default)]
pub struct TravoltaAnimation {
    pub frame_index: usize,
    pub is_active: bool,
    pub finish_state: Option<FinishState>,
    /// Ticks remaining to show finish state before clearing.
    pub finish_ticks: u8,
}

impl TravoltaAnimation {
    pub fn start(&mut self) {
        self.is_active = true;
        self.finish_state = None;
        self.frame_index = 0;
    }

    pub fn tick(&mut self) {
        if self.finish_state.is_some() {
            if self.finish_ticks > 0 {
                self.finish_ticks -= 1;
                if self.finish_ticks == 0 {
                    self.is_active = false;
                    self.finish_state = None;
                }
            }
            return;
        }
        if self.is_active {
            self.frame_index = (self.frame_index + 1) % TRAVOLTA_FRAMES.len();
        }
    }

    pub fn finish(&mut self, state: FinishState) {
        self.finish_state = Some(state);
        // ~500ms at 150ms per tick ≈ 3 ticks
        self.finish_ticks = 3;
    }

    pub fn current_frame(&self) -> &str {
        if let Some(fs) = &self.finish_state {
            return match fs {
                FinishState::Success => TRAVOLTA_SUCCESS,
                FinishState::Error => TRAVOLTA_ERROR,
            };
        }
        if self.is_active {
            TRAVOLTA_FRAMES[self.frame_index]
        } else {
            ""
        }
    }
}

// ────────────────────────────────────────────
// Bookmark & History items
// ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkItem {
    pub title: String,
    pub url: String,
    pub source: String,
    pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryItem {
    pub tags: Vec<String>,
    pub logic: SearchLogic,
    pub timestamp: String,
}

// ────────────────────────────────────────────
// Status message
// ────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct StatusMessage {
    pub text: String,
    pub ticks_remaining: u8,
}

impl StatusMessage {
    pub fn set(&mut self, text: impl Into<String>, ticks: u8) {
        self.text = text.into();
        self.ticks_remaining = ticks;
    }

    pub fn tick(&mut self) {
        if self.ticks_remaining > 0 {
            self.ticks_remaining -= 1;
            if self.ticks_remaining == 0 {
                self.text.clear();
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        !self.text.is_empty()
    }
}

// ────────────────────────────────────────────
// Available filter options (cached at startup)
// ────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct AvailableOptions {
    pub spaces: Vec<String>,
    pub projects: Vec<String>,
    pub statuses: Vec<String>,
    pub repos: Vec<String>,
}

// ────────────────────────────────────────────
// App
// ────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct App {
    pub mode: AppMode,
    pub focus: FocusPane,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub filter_panel_open: bool,
    pub bookmark_panel_open: bool,
    pub history_panel_open: bool,
    pub bookmarks: Vec<BookmarkItem>,
    pub history: Vec<HistoryItem>,
    pub travolta: TravoltaAnimation,
    pub options: AvailableOptions,
    pub status_msg: StatusMessage,
    pub help_open: bool,
    pub pending_storage_warning: Option<String>,
    /// Index within the active overlay list (bookmarks / history / filter).
    pub overlay_index: usize,
    /// Clipboard success tick counter (separate from status_msg for simplicity).
    pub clipboard_ticks: u8,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = App::default();
        app.tabs.push(Tab::new(1));
        app
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn new_tab(&mut self) {
        let id = self.tabs.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        self.tabs.push(Tab::new(id));
        self.active_tab = self.tabs.len() - 1;
        self.focus = FocusPane::SearchBar;
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
    }

    pub fn switch_tab_left(&mut self) {
        if self.active_tab > 0 {
            self.active_tab -= 1;
        }
    }

    pub fn switch_tab_right(&mut self) {
        if self.active_tab + 1 < self.tabs.len() {
            self.active_tab += 1;
        }
    }

    pub fn jump_to_tab(&mut self, n: usize) {
        if n < self.tabs.len() {
            self.active_tab = n;
        }
    }
}

// ────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_logic_toggle() {
        let l = SearchLogic::And;
        assert_eq!(l.toggle(), SearchLogic::Or);
        assert_eq!(l.toggle().toggle(), SearchLogic::And);
    }

    #[test]
    fn test_search_state_add_tag() {
        let mut s = SearchState::default();
        s.input = "kubernetes".into();
        s.add_tag(s.input.clone());
        assert_eq!(s.tags, vec!["kubernetes"]);
        assert!(s.input.is_empty());

        // Duplicate should not be added
        s.input = "kubernetes".into();
        s.add_tag(s.input.clone());
        assert_eq!(s.tags.len(), 1);

        // Empty tag should not be added
        s.input = "  ".into();
        s.add_tag(s.input.clone());
        assert_eq!(s.tags.len(), 1);
    }

    #[test]
    fn test_search_state_delete_last_tag() {
        let mut s = SearchState::default();
        s.tags = vec!["a".into(), "b".into()];
        s.delete_last_tag();
        assert_eq!(s.tags, vec!["a"]);
        s.delete_last_tag();
        assert!(s.tags.is_empty());
        // No panic on empty
        s.delete_last_tag();
    }

    #[test]
    fn test_tab_move() {
        let mut tab = Tab::new(1);
        // Populate some results so we can move
        tab.groups[0].results.push(SearchResult::Page(PageResult {
            id: "1".into(),
            title: "Page".into(),
            space_key: "DS".into(),
            url: "http://ex.com".into(),
            last_modified: "2026-01-01".into(),
            body: None,
        }));
        tab.groups[0].status = GroupStatus::Done(1);

        // visible: header0 + 1 result + header1 + header2 = 4
        assert_eq!(tab.visible_item_count(), 4);
        tab.move_down();
        assert_eq!(tab.selected_index, 1);
        tab.move_up();
        assert_eq!(tab.selected_index, 0);
    }

    #[test]
    fn test_tab_collapse() {
        let mut tab = Tab::new(1);
        tab.groups[0].results.push(SearchResult::Page(PageResult {
            id: "1".into(),
            title: "Page".into(),
            space_key: "DS".into(),
            url: "http://ex.com".into(),
            last_modified: "2026-01-01".into(),
            body: None,
        }));
        tab.groups[0].status = GroupStatus::Done(1);

        assert_eq!(tab.visible_item_count(), 4);
        tab.toggle_collapse_at_cursor(); // collapse group 0 (cursor at 0 = header)
        assert_eq!(tab.visible_item_count(), 3); // result hidden
    }

    #[test]
    fn test_travolta_animation() {
        let mut anim = TravoltaAnimation::default();
        assert_eq!(anim.current_frame(), "");

        anim.start();
        let f0 = anim.current_frame().to_string();
        anim.tick();
        let f1 = anim.current_frame().to_string();
        assert_ne!(f0, f1);

        anim.finish(FinishState::Success);
        assert!(anim.current_frame().contains('\u{2728}'));
        // tick down finish_ticks
        anim.tick();
        anim.tick();
        anim.tick();
        assert!(!anim.is_active);
    }

    #[test]
    fn test_app_tab_management() {
        let mut app = App::new();
        assert_eq!(app.tabs.len(), 1);
        app.new_tab();
        assert_eq!(app.tabs.len(), 2);
        assert_eq!(app.active_tab, 1);

        app.switch_tab_left();
        assert_eq!(app.active_tab, 0);

        app.close_tab();
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn test_app_jump_to_tab() {
        let mut app = App::new();
        app.new_tab();
        app.new_tab();
        app.jump_to_tab(1);
        assert_eq!(app.active_tab, 1);
        app.jump_to_tab(10); // out of bounds, no change
        assert_eq!(app.active_tab, 1);
    }

    #[test]
    fn test_status_message() {
        let mut msg = StatusMessage::default();
        assert!(!msg.is_visible());
        msg.set("hello", 2);
        assert!(msg.is_visible());
        msg.tick();
        assert!(msg.is_visible());
        msg.tick();
        assert!(!msg.is_visible());
    }
}
