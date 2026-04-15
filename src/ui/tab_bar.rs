use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut spans: Vec<Span> = Vec::new();

    for (i, tab) in app.tabs.iter().enumerate() {
        let is_active = i == app.active_tab;
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        spans.push(Span::styled(format!(" [{}] ", tab.name), style));
        if i + 1 < app.tabs.len() {
            spans.push(Span::raw("  "));
        }
    }

    spans.push(Span::raw("  "));
    spans.push(Span::styled("[+]", Style::default().fg(Color::Green)));
    spans.push(Span::raw("    "));
    spans.push(Span::styled(
        "Ctrl+T:新規  Ctrl+W:閉じる",
        Style::default().fg(Color::DarkGray),
    ));

    let line = Line::from(spans);
    let para = Paragraph::new(line);
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_exists() {
        // Smoke test: ensure the module compiles and App can be created.
        let app = App::new();
        assert_eq!(app.tabs.len(), 1);
        assert_eq!(app.active_tab, 0);
    }
}
