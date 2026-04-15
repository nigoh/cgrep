use crate::app::{App, AvailableOptions, FilterState};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let popup = crate::ui::centered_rect(60, 70, area);
    f.render_widget(Clear, popup);

    let tab = app.active_tab();
    let filter = &tab.filter;
    let opts = &app.options;

    let mut items: Vec<ListItem> = Vec::new();
    let flat_items: Vec<FilterItem> = build_filter_items(filter, opts);

    for (i, fi) in flat_items.iter().enumerate() {
        let is_sel = i == app.overlay_index;
        let check = if fi.enabled { "✅" } else { "□" };
        let text = format!("  {} {}", check, fi.label);
        let style = if is_sel {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else if fi.is_header {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🔧 フィルタ  [Esc:閉じる] ")
        .style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    state.select(Some(app.overlay_index));

    f.render_stateful_widget(
        List::new(items).block(block),
        popup,
        &mut state,
    );
}

#[derive(Debug)]
pub struct FilterItem {
    pub label: String,
    pub enabled: bool,
    pub is_header: bool,
    pub kind: FilterItemKind,
}

#[derive(Debug, Clone)]
pub enum FilterItemKind {
    Header,
    Source(SourceToggle),
    Space(String),
    Project(String),
    Status(String),
    Repo(String),
    GerritCommits,
    GerritTags,
}

#[derive(Debug, Clone)]
pub enum SourceToggle {
    Confluence,
    Jira,
    Gerrit,
}

pub fn build_filter_items(filter: &FilterState, opts: &AvailableOptions) -> Vec<FilterItem> {
    let mut items = Vec::new();

    // Source section
    items.push(FilterItem {
        label: "ソース".into(),
        enabled: false,
        is_header: true,
        kind: FilterItemKind::Header,
    });
    items.push(FilterItem {
        label: "Confluence".into(),
        enabled: filter.confluence_enabled,
        is_header: false,
        kind: FilterItemKind::Source(SourceToggle::Confluence),
    });
    items.push(FilterItem {
        label: "Jira".into(),
        enabled: filter.jira_enabled,
        is_header: false,
        kind: FilterItemKind::Source(SourceToggle::Jira),
    });
    items.push(FilterItem {
        label: "Gerrit".into(),
        enabled: filter.gerrit_enabled,
        is_header: false,
        kind: FilterItemKind::Source(SourceToggle::Gerrit),
    });

    // Confluence spaces
    if !opts.spaces.is_empty() {
        items.push(FilterItem {
            label: "Confluenceスペース".into(),
            enabled: false,
            is_header: true,
            kind: FilterItemKind::Header,
        });
        for sp in &opts.spaces {
            items.push(FilterItem {
                label: sp.clone(),
                enabled: filter.spaces.contains(sp),
                is_header: false,
                kind: FilterItemKind::Space(sp.clone()),
            });
        }
    }

    // Jira projects
    if !opts.projects.is_empty() {
        items.push(FilterItem {
            label: "Jiraプロジェクト".into(),
            enabled: false,
            is_header: true,
            kind: FilterItemKind::Header,
        });
        for p in &opts.projects {
            items.push(FilterItem {
                label: p.clone(),
                enabled: filter.projects.contains(p),
                is_header: false,
                kind: FilterItemKind::Project(p.clone()),
            });
        }
    }

    // Jira statuses
    if !opts.statuses.is_empty() {
        items.push(FilterItem {
            label: "Jiraステータス".into(),
            enabled: false,
            is_header: true,
            kind: FilterItemKind::Header,
        });
        for s in &opts.statuses {
            items.push(FilterItem {
                label: s.clone(),
                enabled: filter.statuses.contains(s),
                is_header: false,
                kind: FilterItemKind::Status(s.clone()),
            });
        }
    }

    // Gerrit repos
    if !opts.repos.is_empty() {
        items.push(FilterItem {
            label: "Gerritリポジトリ".into(),
            enabled: false,
            is_header: true,
            kind: FilterItemKind::Header,
        });
        for r in &opts.repos {
            items.push(FilterItem {
                label: r.clone(),
                enabled: filter.repos.contains(r),
                is_header: false,
                kind: FilterItemKind::Repo(r.clone()),
            });
        }
    }

    // Gerrit target types
    items.push(FilterItem {
        label: "Gerrit対象".into(),
        enabled: false,
        is_header: true,
        kind: FilterItemKind::Header,
    });
    items.push(FilterItem {
        label: "コミット".into(),
        enabled: filter.gerrit_commits,
        is_header: false,
        kind: FilterItemKind::GerritCommits,
    });
    items.push(FilterItem {
        label: "タグ".into(),
        enabled: filter.gerrit_tags,
        is_header: false,
        kind: FilterItemKind::GerritTags,
    });

    items
}

/// Apply a toggle at the given overlay index to the filter state.
pub fn toggle_at(index: usize, filter: &mut FilterState, opts: &AvailableOptions) {
    let items = build_filter_items(filter, opts);
    if index >= items.len() {
        return;
    }
    let item = &items[index];
    match &item.kind {
        FilterItemKind::Header => {}
        FilterItemKind::Source(s) => match s {
            SourceToggle::Confluence => filter.confluence_enabled = !filter.confluence_enabled,
            SourceToggle::Jira => filter.jira_enabled = !filter.jira_enabled,
            SourceToggle::Gerrit => filter.gerrit_enabled = !filter.gerrit_enabled,
        },
        FilterItemKind::Space(sp) => {
            if filter.spaces.contains(sp.as_str()) {
                filter.spaces.remove(sp.as_str());
            } else {
                filter.spaces.insert(sp.clone());
            }
        }
        FilterItemKind::Project(p) => {
            if filter.projects.contains(p.as_str()) {
                filter.projects.remove(p.as_str());
            } else {
                filter.projects.insert(p.clone());
            }
        }
        FilterItemKind::Status(s) => {
            if filter.statuses.contains(s.as_str()) {
                filter.statuses.remove(s.as_str());
            } else {
                filter.statuses.insert(s.clone());
            }
        }
        FilterItemKind::Repo(r) => {
            if filter.repos.contains(r.as_str()) {
                filter.repos.remove(r.as_str());
            } else {
                filter.repos.insert(r.clone());
            }
        }
        FilterItemKind::GerritCommits => filter.gerrit_commits = !filter.gerrit_commits,
        FilterItemKind::GerritTags => filter.gerrit_tags = !filter.gerrit_tags,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AvailableOptions, FilterState};

    fn make_opts() -> AvailableOptions {
        AvailableOptions {
            spaces: vec!["DS".into(), "OPS".into()],
            projects: vec!["OPS".into()],
            statuses: vec!["In Progress".into()],
            repos: vec!["infra".into()],
        }
    }

    #[test]
    fn test_build_filter_items_includes_sources() {
        let filter = FilterState::default();
        let opts = make_opts();
        let items = build_filter_items(&filter, &opts);
        let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();
        assert!(labels.contains(&"Confluence"));
        assert!(labels.contains(&"Jira"));
        assert!(labels.contains(&"Gerrit"));
        assert!(labels.contains(&"DS"));
        assert!(labels.contains(&"コミット"));
    }

    #[test]
    fn test_toggle_confluence() {
        let mut filter = FilterState::default();
        assert!(filter.confluence_enabled);
        let opts = make_opts();
        // Index 1 = Confluence source toggle (after header at 0)
        toggle_at(1, &mut filter, &opts);
        assert!(!filter.confluence_enabled);
    }

    #[test]
    fn test_toggle_space() {
        let mut filter = FilterState::default();
        let opts = make_opts();
        let items = build_filter_items(&filter, &opts);
        // Find DS space index
        let ds_idx = items.iter().position(|i| i.label == "DS").unwrap();
        toggle_at(ds_idx, &mut filter, &opts);
        assert!(filter.spaces.contains("DS"));
        toggle_at(ds_idx, &mut filter, &opts);
        assert!(!filter.spaces.contains("DS"));
    }
}
