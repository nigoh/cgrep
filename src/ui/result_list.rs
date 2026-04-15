use crate::app::{App, FocusPane, GroupStatus, SearchResult, SourceKind};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tab = app.active_tab();
    let focused = matches!(app.focus, FocusPane::ResultList);
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_idx = 0usize;

    for group in &tab.groups {
        // Section header
        let status_str = match &group.status {
            GroupStatus::Loading => "取得中...".to_string(),
            GroupStatus::Done(n) => format!("{n}件"),
            GroupStatus::Error(e) => format!("❌ {e}"),
        };
        let collapse_icon = if group.collapsed { "▶" } else { "▼" };
        let header_text = format!(
            " {} [— {} ({}) —] ",
            collapse_icon,
            group.source.label(),
            status_str
        );
        let is_selected = flat_idx == tab.selected_index;
        let header_style = if is_selected && focused {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };
        items.push(ListItem::new(Line::from(Span::styled(
            header_text,
            header_style,
        ))));
        flat_idx += 1;

        if !group.collapsed {
            for result in &group.results {
                let is_sel = flat_idx == tab.selected_index;
                let line = format_result_line(result, &group.source);
                let style = if is_sel && focused {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if is_sel { "▶ " } else { "  " };
                let full = format!("{prefix}{line}");
                items.push(ListItem::new(Line::from(Span::styled(full, style))));
                flat_idx += 1;
            }
        }
    }

    // Total count line
    let total: usize = tab.groups.iter().map(|g| g.results.len()).sum();
    if total > 0 {
        items.push(ListItem::new(Line::from(Span::styled(
            format!(" 計{total}件"),
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" 結果 ");

    let mut state = ListState::default();
    state.select(Some(tab.selected_index));

    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn format_result_line(result: &SearchResult, _source: &SourceKind) -> String {
    match result {
        SearchResult::Page(p) => format!("{}  {}", p.space_key, p.title),
        SearchResult::Issue(i) => format!("{}  {}", i.key, i.summary),
        SearchResult::Commit(c) => format!("[COMMIT] {}: {}", c.repo, truncate(&c.subject, 40)),
        SearchResult::Tag(t) => format!("[TAG] {}: {}", t.repo, t.name),
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    let mut chars = s.chars();
    let mut result: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        result.push_str("..");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{CommitResult, PageResult, SearchResult};

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 5), "hello..");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("exact", 5), "exact");
    }

    #[test]
    fn test_truncate_japanese() {
        // All 5 chars fit within max_chars=5
        assert_eq!(truncate("あいうえお", 5), "あいうえお");
        // 6 chars, max=5 → first 5 + ".."
        assert_eq!(truncate("あいうえおか", 5), "あいうえお..");
        // Mixed ASCII + Japanese: 9 chars, max=6 → first 6 + ".."
        assert_eq!(truncate("K8s手順確認テスト", 6), "K8s手順確..");
    }

    #[test]
    fn test_format_result_line_page() {
        let r = SearchResult::Page(PageResult {
            id: "1".into(),
            title: "K8s手順".into(),
            space_key: "DS".into(),
            url: "http://ex.com".into(),
            last_modified: "2026-01-01".into(),
            body: None,
        });
        let line = format_result_line(&r, &SourceKind::Confluence);
        assert!(line.contains("DS"));
        assert!(line.contains("K8s手順"));
    }

    #[test]
    fn test_format_result_line_commit() {
        let r = SearchResult::Commit(CommitResult {
            change_id: "abc".into(),
            subject: "fix: k8s timeout".into(),
            repo: "infra".into(),
            owner: "dev".into(),
            created: "2026-01-01".into(),
            url: "http://gerrit.example.com".into(),
        });
        let line = format_result_line(&r, &SourceKind::Gerrit);
        assert!(line.contains("[COMMIT]"));
        assert!(line.contains("infra"));
    }
}
