use crate::app::{App, AppMode, GroupStatus};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Top line: per-source progress
    let progress_line = build_progress_line(app);

    // Bottom line: key hints + status message
    let hint_line = if app.status_msg.is_visible() {
        Line::from(Span::styled(
            app.status_msg.text.clone(),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        build_key_hint_line(app)
    };

    let _mode_label = match app.mode {
        AppMode::Incremental => " [I] ",
        AppMode::Normal => " [N] ",
    };

    let block = Block::default().borders(Borders::TOP);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines = vec![progress_line, hint_line];
    let para = Paragraph::new(lines);
    f.render_widget(para, inner);
}

fn build_progress_line(app: &App) -> Line<'static> {
    let tab = app.active_tab();
    let mut spans: Vec<Span> = Vec::new();

    for (i, group) in tab.groups.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  |  "));
        }
        let label = group.source.label();
        let (status_text, status_style) = match &group.status {
            GroupStatus::Loading => (
                format!("{label}: 取得中..."),
                Style::default().fg(Color::Yellow),
            ),
            GroupStatus::Done(n) => (
                format!("{label}: {n}件 ✅"),
                Style::default().fg(Color::Green),
            ),
            GroupStatus::Error(e) => (format!("{label}: ❌ {e}"), Style::default().fg(Color::Red)),
        };
        spans.push(Span::styled(status_text, status_style));
    }

    // Total count
    let total: usize = tab.groups.iter().map(|g| g.results.len()).sum();
    let all_done = tab
        .groups
        .iter()
        .all(|g| !matches!(g.status, GroupStatus::Loading));
    if all_done && total > 0 {
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(
            format!("計{total}件"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    }

    Line::from(spans)
}

fn build_key_hint_line(app: &App) -> Line<'static> {
    let mode_hint = match app.mode {
        AppMode::Incremental => "[I]",
        AppMode::Normal => "[N]",
    };
    Line::from(vec![
        Span::styled(mode_hint, Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled(
            "↑↓:移動  Enter:開く  y:URLコピー  b:ブックマーク  Space:折畳  Tab:フィルタ  Ctrl+M:AND/OR  ?:ヘルプ",
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use crate::app::{App, GroupStatus};

    #[test]
    fn test_progress_line_loading() {
        let app = App::new();
        // All groups start as Loading
        let tab = app.active_tab();
        assert!(tab
            .groups
            .iter()
            .all(|g| matches!(g.status, GroupStatus::Loading)));
    }

    #[test]
    fn test_total_count() {
        let mut app = App::new();
        app.active_tab_mut().groups[0].status = GroupStatus::Done(5);
        app.active_tab_mut().groups[1].status = GroupStatus::Done(3);
        app.active_tab_mut().groups[2].status = GroupStatus::Done(2);
        let total: usize = app
            .active_tab()
            .groups
            .iter()
            .map(|g| g.results.len())
            .sum();
        assert_eq!(total, 0); // No actual results pushed, just status counts
    }
}
