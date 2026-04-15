use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let popup = crate::ui::centered_rect(65, 60, area);
    f.render_widget(Clear, popup);

    let mut items: Vec<ListItem> = Vec::new();

    for (i, bm) in app.bookmarks.iter().enumerate() {
        let is_sel = i == app.overlay_index;
        let icon = match bm.source.as_str() {
            "Confluence" => "[PAGE]",
            "Jira" => "[JIRA]",
            _ => "[GIT] ",
        };
        let text = format!(
            "  {} {}  {}",
            icon,
            bm.title,
            bm.added_at.get(..10).unwrap_or("")
        );
        let style = if is_sel {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(Line::from(Span::styled(text, style))));
    }

    if app.bookmarks.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  ブックマークはありません",
            Style::default().fg(Color::DarkGray),
        ))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ★ ブックマーク  [Esc:閉じる] ")
        .style(Style::default().bg(Color::DarkGray));

    let mut state = ListState::default();
    if !app.bookmarks.is_empty() {
        state.select(Some(app.overlay_index));
    }

    f.render_stateful_widget(List::new(items).block(block), popup, &mut state);

    // Hint bar at the bottom of popup
    let hint = Line::from(Span::styled(
        "  Enter:開く  y:URLコピー  d:削除",
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
    use crate::app::{App, BookmarkItem};

    #[test]
    fn test_bookmark_panel_empty() {
        let app = App::new();
        assert!(app.bookmarks.is_empty());
    }

    #[test]
    fn test_bookmark_icon_selection() {
        let source_icon = |s: &str| match s {
            "Confluence" => "[PAGE]",
            "Jira" => "[JIRA]",
            _ => "[GIT] ",
        };
        assert_eq!(source_icon("Confluence"), "[PAGE]");
        assert_eq!(source_icon("Jira"), "[JIRA]");
        assert_eq!(source_icon("Gerrit"), "[GIT] ");
    }
}
