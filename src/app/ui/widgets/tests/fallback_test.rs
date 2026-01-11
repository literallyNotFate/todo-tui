// Unit-tests for fallback widget
#[cfg(test)]
mod tests {
    use crate::app::{
        ui::widgets::fallback::{fallback::Fallback, utils::render_fallback_message},
        utils::constants::{terminal::*, theme::*},
    };
    use ratatui::{
        Terminal,
        backend::TestBackend,
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
    };

    #[test]
    fn should_render_fallback_on_small_terminal() {
        let backend: TestBackend = TestBackend::new(50, 15);
        let mut terminal = Terminal::new(backend).unwrap();

        let frame_area: Rect = Rect::new(0, 0, 50, 15);

        terminal
            .draw(|frame| {
                Fallback::render(frame, frame_area);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer.content();
        let text = content.iter().map(|cell| cell.symbol()).collect::<String>();

        assert!(text.contains("Terminal is too small!"));
        assert!(text.contains("Required:"));
        assert!(text.contains(&format!("{}", MIN_WIDTH)));
        assert!(text.contains(&format!("{}", MIN_HEIGHT)));
        assert!(text.contains("Current:"));
        assert!(text.contains(&format!("{} x {}", frame_area.width, frame_area.height)));
    }

    #[test]
    fn should_render_fallback_message() {
        let width: u16 = 80;
        let height: u16 = 24;
        let colors: (Color, Color) = (COLOR_RED, COLOR_GREEN);

        let result: Vec<Line> = render_fallback_message(width, height, colors);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].spans[0], Span::from("Terminal is too small!"));
        assert_eq!(result[0].alignment, Some(Alignment::Center));
        assert_eq!(
            result[2].spans,
            vec![
                Span::styled("Required: ", Style::default().fg(TEXT_PRIMARY)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD)
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD)
                ),
            ],
        );
        assert_eq!(result[2].alignment, Some(Alignment::Center));
        assert_eq!(
            result[4].spans,
            vec![
                Span::raw("Current:  "),
                Span::styled(
                    format!("{}", width),
                    Style::default().fg(colors.0).add_modifier(Modifier::BOLD)
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", height),
                    Style::default().fg(colors.1).add_modifier(Modifier::BOLD)
                ),
            ],
        );
        assert_eq!(result[4].alignment, Some(Alignment::Center));
    }
}
