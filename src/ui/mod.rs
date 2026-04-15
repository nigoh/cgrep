pub mod bookmark_panel;
pub mod filter_panel;
pub mod history_panel;
pub mod layout;
pub mod preview;
pub mod result_list;
pub mod search_bar;
pub mod status_bar;
pub mod tab_bar;
pub mod travolta;

use crate::app::App;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let areas = layout::compute(f.area());

    tab_bar::render(f, areas.tab_bar, app);
    search_bar::render(f, areas.search_bar, app);

    let main_chunks = layout::split_main(areas.main);
    result_list::render(f, main_chunks[0], app);
    preview::render(f, main_chunks[1], app);

    let footer_chunks = layout::split_footer(areas.footer);
    travolta::render(f, footer_chunks[0], app);
    status_bar::render(f, footer_chunks[1], app);

    // Overlays (drawn on top)
    if app.filter_panel_open {
        filter_panel::render(f, f.area(), app);
    }
    if app.bookmark_panel_open {
        bookmark_panel::render(f, f.area(), app);
    }
    if app.history_panel_open {
        history_panel::render(f, f.area(), app);
    }
    if app.help_open {
        render_help(f, f.area());
    }
}

fn render_help(f: &mut Frame, area: ratatui::layout::Rect) {
    use ratatui::{
        style::{Color, Style},
        text::Line,
        widgets::{Block, Borders, Clear, Paragraph},
    };

    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);

    let lines: Vec<Line> = vec![
        Line::from(" グローバル"),
        Line::from("  q / Ctrl+C  終了"),
        Line::from("  Ctrl+T      新規タブ"),
        Line::from("  Ctrl+W      タブを閉じる"),
        Line::from("  Ctrl+←/→    タブ切替"),
        Line::from("  Ctrl+S      タブを保存"),
        Line::from("  Alt+1-9     タブ番号で切替"),
        Line::from("  Ctrl+B      ブックマーク一覧"),
        Line::from(""),
        Line::from(" 検索バー"),
        Line::from("  Enter       タグ化 / 検索実行"),
        Line::from("  ↑           検索履歴パネルを開く"),
        Line::from("  Backspace   最後のタグを削除"),
        Line::from("  Ctrl+M      AND / OR 切替"),
        Line::from("  /           Incremental / Normal 切替"),
        Line::from("  Tab         フィルタパネル"),
        Line::from(""),
        Line::from(" 結果リスト"),
        Line::from("  ↑/↓         移動"),
        Line::from("  Enter       ブラウザで開く"),
        Line::from("  Space       折りたたみ/展開"),
        Line::from("  y           URLをコピー"),
        Line::from("  b           ブックマーク追加/削除"),
        Line::from("  p           プレビューにフォーカス"),
        Line::from(""),
        Line::from("  Esc / ?     閉じる"),
    ];

    let block = Block::default()
        .title(" ? キーバインドヘルプ ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let para = Paragraph::new(lines).block(block);
    f.render_widget(para, popup_area);
}

pub fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
