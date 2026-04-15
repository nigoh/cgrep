use crate::app::{App, FocusPane, SearchResult};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tab = app.active_tab();
    let focused = matches!(app.focus, FocusPane::Preview);
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(" プレビュー ");

    match tab.selected_result() {
        None => {
            let para = Paragraph::new("アイテムを選択してください")
                .style(Style::default().fg(Color::DarkGray))
                .block(block);
            f.render_widget(para, area);
        }
        Some(result) => {
            let lines = render_result(result);
            let para = Paragraph::new(lines)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((tab.preview_scroll, 0));
            f.render_widget(para, area);

            // Bookmark/copy hint at bottom-right
            let hint = Span::styled(
                "[b:bookmark]  [y:URLコピー]",
                Style::default().fg(Color::DarkGray),
            );
            let hint_area = Rect {
                x: area.x + area.width.saturating_sub(27),
                y: area.y + area.height.saturating_sub(1),
                width: 26,
                height: 1,
            };
            f.render_widget(Paragraph::new(Line::from(hint)), hint_area);
        }
    }
}

fn render_result(result: &SearchResult) -> Vec<Line<'static>> {
    match result {
        SearchResult::Page(p) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("タイトル: ", Style::default().fg(Color::Yellow)),
                    Span::raw(p.title.clone()),
                ]),
                Line::from(vec![
                    Span::styled("スペース: ", Style::default().fg(Color::Yellow)),
                    Span::raw(p.space_key.clone()),
                ]),
                Line::from(vec![
                    Span::styled("更新日時: ", Style::default().fg(Color::Yellow)),
                    Span::raw(p.last_modified.clone()),
                ]),
                Line::from(vec![
                    Span::styled("URL: ", Style::default().fg(Color::Yellow)),
                    Span::styled(p.url.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from("─".repeat(40)),
            ];
            if let Some(body) = &p.body {
                let plain = html_to_plain(body);
                for l in plain.lines() {
                    lines.push(Line::from(l.to_string()));
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "本文を取得中...",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines
        }
        SearchResult::Issue(i) => {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("チケット: ", Style::default().fg(Color::Yellow)),
                    Span::styled(i.key.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("概要: ", Style::default().fg(Color::Yellow)),
                    Span::raw(i.summary.clone()),
                ]),
                Line::from(vec![
                    Span::styled("プロジェクト: ", Style::default().fg(Color::Yellow)),
                    Span::raw(i.project.clone()),
                ]),
                Line::from(vec![
                    Span::styled("ステータス: ", Style::default().fg(Color::Yellow)),
                    Span::raw(i.status.clone()),
                ]),
                Line::from(vec![
                    Span::styled("URL: ", Style::default().fg(Color::Yellow)),
                    Span::styled(i.url.clone(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from("─".repeat(40)),
            ];
            if let Some(desc) = &i.description {
                for l in desc.lines() {
                    lines.push(Line::from(l.to_string()));
                }
            } else {
                lines.push(Line::from(Span::styled(
                    "説明を取得中...",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines
        }
        SearchResult::Commit(c) => vec![
            Line::from(vec![
                Span::styled("[COMMIT] ", Style::default().fg(Color::Green)),
                Span::raw(c.subject.clone()),
            ]),
            Line::from(vec![
                Span::styled("リポジトリ: ", Style::default().fg(Color::Yellow)),
                Span::raw(c.repo.clone()),
            ]),
            Line::from(vec![
                Span::styled("作成者: ", Style::default().fg(Color::Yellow)),
                Span::raw(c.owner.clone()),
            ]),
            Line::from(vec![
                Span::styled("作成日時: ", Style::default().fg(Color::Yellow)),
                Span::raw(c.created.clone()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Yellow)),
                Span::styled(c.url.clone(), Style::default().fg(Color::Cyan)),
            ]),
        ],
        SearchResult::Tag(t) => vec![
            Line::from(vec![
                Span::styled("[TAG] ", Style::default().fg(Color::Magenta)),
                Span::raw(t.name.clone()),
            ]),
            Line::from(vec![
                Span::styled("リポジトリ: ", Style::default().fg(Color::Yellow)),
                Span::raw(t.repo.clone()),
            ]),
            Line::from(vec![
                Span::styled("リビジョン: ", Style::default().fg(Color::Yellow)),
                Span::raw(t.revision.clone()),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Yellow)),
                Span::styled(t.url.clone(), Style::default().fg(Color::Cyan)),
            ]),
        ],
    }
}

fn html_to_plain(html: &str) -> String {
    html2text::from_read(html.as_bytes(), 80)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PageResult;

    #[test]
    fn test_html_to_plain_basic() {
        let html = "<h1>Hello</h1><p>World</p>";
        let plain = html_to_plain(html);
        assert!(plain.contains("Hello") || plain.contains("World"));
    }

    #[test]
    fn test_render_page_no_body() {
        let r = SearchResult::Page(PageResult {
            id: "1".into(),
            title: "K8s手順".into(),
            space_key: "DS".into(),
            url: "http://ex.com".into(),
            last_modified: "2026-01-01".into(),
            body: None,
        });
        let lines = render_result(&r);
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref().to_string()))
            .collect();
        assert!(text.contains("K8s手順"));
        assert!(text.contains("DS"));
    }
}
