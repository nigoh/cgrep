use crate::app::{App, FocusPane, SearchLogic};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tab = app.active_tab();
    let search = &tab.search;

    let focused = matches!(app.focus, FocusPane::SearchBar);
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let logic_label = search.logic.label();
    let logic_style = match search.logic {
        SearchLogic::And => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        SearchLogic::Or => Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    };

    let mut spans: Vec<Span> = vec![
        Span::raw("🔍 "),
        Span::styled(logic_label, logic_style),
        Span::raw(" │ "),
    ];

    for tag in &search.tags {
        spans.push(Span::styled(
            format!("❰{}❱ ", tag),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }

    // Show cursor/input
    if !search.input.is_empty() {
        spans.push(Span::raw(&search.input));
    }
    spans.push(Span::styled("_", Style::default().fg(Color::White)));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" 検索 ");

    let line = Line::from(spans);
    let para = Paragraph::new(line).block(block);
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use crate::app::SearchState;

    #[test]
    fn test_search_state_display_logic() {
        let mut state = SearchState::default();
        assert_eq!(state.logic.label(), "AND");
        state.toggle_logic();
        assert_eq!(state.logic.label(), "OR");
    }

    #[test]
    fn test_search_state_input_cycle() {
        let mut state = SearchState::default();
        state.input = "kubernetes".into();
        assert!(!state.has_tags());
        state.add_tag(state.input.clone());
        assert!(state.has_tags());
        assert!(state.input.is_empty());
    }
}
