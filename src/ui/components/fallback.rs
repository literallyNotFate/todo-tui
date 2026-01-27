use crate::{
    theme::ThemeColors,
    ui::{MIN_HEIGHT, MIN_WIDTH, center},
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
};

pub struct Fallback;

impl Fallback {
    pub fn render(frame: &mut Frame, frame_area: Rect, theme: &ThemeColors) {
        use ratatui::{
            text::Text,
            widgets::{Block, Clear, Paragraph, Wrap},
        };

        let area: Rect = center(50, 50, frame_area);
        let colors: (Color, Color) = Self::dimension_colors(&frame_area, theme);

        let message: Vec<Line> =
            Self::fallback_message(&frame_area.width, &frame_area.height, colors, theme);

        let paragraph: Paragraph = Paragraph::new(Text::from(message))
            .style(Style::default().fg(theme.text_primary))
            .wrap(Wrap { trim: false })
            .block(Block::default());

        frame.render_widget(Clear, frame_area);
        frame.render_widget(paragraph, area);
    }

    // Render fallback message
    pub(crate) fn fallback_message(
        width: &u16,
        height: &u16,
        colors: (Color, Color),
        theme: &ThemeColors,
    ) -> Vec<Line<'static>> {
        use ratatui::{style::Modifier, text::Span};

        let message: Vec<Line> = vec![
            Line::styled("Terminal is too small!", Style::default().fg(theme.error)).centered(),
            Line::from(""),
            Line::from(vec![
                Span::styled("Required: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current:  ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("{}", width),
                    Style::default().fg(colors.0).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", height),
                    Style::default().fg(colors.1).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
        ];

        message
    }

    // Get colors for width/height while resized
    pub(crate) fn dimension_colors(area: &Rect, theme: &ThemeColors) -> (Color, Color) {
        let width_color = if area.width >= MIN_WIDTH {
            theme.success
        } else {
            theme.error
        };

        let height_color = if area.height >= MIN_HEIGHT {
            theme.success
        } else {
            theme.error
        };

        (width_color, height_color)
    }
}

// Unit-tests for fallback widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::{layout::Alignment, style::Modifier, text::Span};

    #[test]
    fn should_render_fallback_message() {
        let width: u16 = 80;
        let height: u16 = 24;

        let theme: ThemeColors = Theme::Gruvbox.data();
        let colors: (Color, Color) = (theme.error, theme.success);
        let result: Vec<Line> = Fallback::fallback_message(&width, &height, colors, &theme);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].spans[0], Span::from("Terminal is too small!"));
        assert_eq!(result[0].alignment, Some(Alignment::Center));
        assert_eq!(
            result[2].spans,
            vec![
                Span::styled("Required: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD)
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD)
                ),
            ],
        );
        assert_eq!(result[2].alignment, Some(Alignment::Center));
        assert_eq!(
            result[4].spans,
            vec![
                Span::styled("Current:  ", Style::default().fg(theme.text_dim)),
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
