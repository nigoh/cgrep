use crate::app::{App, FinishState};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let anim = &app.travolta;
    if !anim.is_active && anim.finish_state.is_none() {
        return;
    }

    let frame_str = anim.current_frame();
    let style = match &anim.finish_state {
        Some(FinishState::Success) => Style::default().fg(Color::Green),
        Some(FinishState::Error) => Style::default().fg(Color::Red),
        None => Style::default().fg(Color::Yellow),
    };

    let lines: Vec<Line> = frame_str
        .lines()
        .map(|l| Line::styled(l.to_string(), style))
        .collect();

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}

#[cfg(test)]
mod tests {
    use crate::app::{FinishState, TravoltaAnimation, TRAVOLTA_FRAMES};

    #[test]
    fn test_frame_count() {
        assert_eq!(TRAVOLTA_FRAMES.len(), 8);
    }

    #[test]
    fn test_animation_advance() {
        let mut anim = TravoltaAnimation::default();
        anim.start();
        let f0 = anim.frame_index;
        anim.tick();
        assert_eq!(anim.frame_index, (f0 + 1) % TRAVOLTA_FRAMES.len());
    }

    #[test]
    fn test_animation_wraps() {
        let mut anim = TravoltaAnimation::default();
        anim.start();
        for _ in 0..16 {
            anim.tick();
        }
        assert!(anim.is_active);
    }

    #[test]
    fn test_finish_success() {
        let mut anim = TravoltaAnimation::default();
        anim.start();
        anim.finish(FinishState::Success);
        let frame = anim.current_frame();
        assert!(frame.contains('\u{2728}'));
    }

    #[test]
    fn test_finish_error() {
        let mut anim = TravoltaAnimation::default();
        anim.start();
        anim.finish(FinishState::Error);
        assert_eq!(anim.current_frame(), " x_x  ");
    }

    #[test]
    fn test_finish_clears_after_ticks() {
        let mut anim = TravoltaAnimation::default();
        anim.start();
        anim.finish(FinishState::Success);
        anim.tick();
        anim.tick();
        anim.tick();
        assert!(!anim.is_active);
        assert!(anim.finish_state.is_none());
    }

    #[test]
    fn test_inactive_returns_empty() {
        let anim = TravoltaAnimation::default();
        assert_eq!(anim.current_frame(), "");
    }
}
