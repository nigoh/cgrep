use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let popup = crate::ui::centered_rect(65, 55, area);
    f.render_widget(Clear, popup);

    let mut items: Vec<ListItem> = Vec::new();

    for (i, h) in app.history.iter().enumerate() {
        let is_sel = i == app.overlay_index;
        let logic = h.logic.label();
        let tags_str = h.tags.join("  ");
        let ts = h.timestamp.get(..10).unwrap_or(&h.timestamp);
        let text = format!("  [{}] {}    {}", logic, tags_str, ts);
        let style = if is_sel {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    if app.history.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  検索履歴はありません",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🕐 検索履歴  [Esc:閉じる] ")
        .style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    if !app.history.is_empty() {
        state.select(Some(app.overlay_index));
    }

    f.render_stateful_widget(List::new(items).block(block), popup, &mut state);

    // Hint bar
    let hint = Line::from(Span::styled(
        "  Enter:再利用  d:削除",
        Style::default().fg(Color::DarkGray),
    ));
    let hint_area = Rect {
        x: popup.x,
        y: popup.y + popup.height.saturating_sub(1),
        width: popup.width,
        height: 1,
    };
    f.render_widget(ratatui::widgets::Paragraph::new(hint), hint_area);
}

#[cfg(test)]
mod tests {
    use crate::app::{App, HistoryItem, SearchLogic};

    #[test]
    fn test_history_panel_empty() {
        let app = App::new();
        assert!(app.history.is_empty());
    }

    #[test]
    fn test_history_label_format() {
        let h = HistoryItem {
            tags: vec!["kubernetes".into(), "deployment".into()],
            logic: SearchLogic::And,
            timestamp: "2026-04-15T10:30:00Z".into(),
        };
        assert_eq!(h.logic.label(), "AND");
        let joined = h.tags.join("  ");
        assert_eq!(joined, "kubernetes  deployment");
        let ts = h.timestamp.get(..10).unwrap_or(&h.timestamp);
        assert_eq!(ts, "2026-04-15");
    }
}
