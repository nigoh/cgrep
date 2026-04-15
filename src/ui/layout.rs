use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct Areas {
    pub tab_bar: Rect,
    pub search_bar: Rect,
    pub main: Rect,
    pub footer: Rect,
}

/// Compute the main layout areas from the full terminal area.
pub fn compute(area: Rect) -> Areas {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tab bar
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // main content
            Constraint::Length(3), // footer (animation + status)
        ])
        .split(area);

    Areas {
        tab_bar: chunks[0],
        search_bar: chunks[1],
        main: chunks[2],
        footer: chunks[3],
    }
}

/// Split the main area into result list (left) and preview (right).
pub fn split_main(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area)
        .to_vec()
}

/// Split the footer area: animation on the left, status bar on the right.
pub fn split_footer(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(0)])
        .split(area)
        .to_vec()
}
