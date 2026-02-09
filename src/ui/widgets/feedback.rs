use crate::{
    theme::ThemeColors,
    ui::{MIN_HEIGHT, MIN_WIDTH, center},
};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};

/// Type of feedback message
#[derive(Debug, PartialEq)]
pub enum FeedbackKind {
    EmptyList,
    NoResults(String),
    SmallTerminal,
}

pub struct FeedbackWidget<'a> {
    kind: FeedbackKind,
    theme: &'a ThemeColors,
}

impl<'a> FeedbackWidget<'a> {
    pub fn new(kind: FeedbackKind, theme: &'a ThemeColors) -> Self {
        Self { kind, theme }
    }
}

impl Widget for FeedbackWidget<'_> {
    /// Feedback rendering
    fn render(self, area: Rect, buf: &mut Buffer) {
        let result_area: Rect = center(area, 50, 50);
        let colors: (Color, Color) = self.dimension_colors(&area);
        let message: Vec<Line>;

        if self.kind == FeedbackKind::SmallTerminal {
            message = self.message(&area.width, &area.height, colors);

            Clear.render(area, buf);
            Paragraph::new(Text::from(message))
                .style(Style::default().fg(self.theme.text_primary))
                .wrap(Wrap { trim: false })
                .block(Block::default())
                .render(result_area, buf);

            return;
        }

        message = self.message(&area.width, &area.height, colors);

        Block::bordered()
            .title(" Tasks ")
            .border_style(self.theme.warning)
            .render(area, buf);

        Paragraph::new(message)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(result_area, buf);
    }
}

impl<'a> FeedbackWidget<'a> {
    /// Return message based on feedback kind
    fn message(&self, width: &u16, height: &u16, colors: (Color, Color)) -> Vec<Line<'static>> {
        use ratatui::text::Span;

        let message: Vec<Line> = match &self.kind {
            FeedbackKind::EmptyList => vec![
                Line::from("All clear!").fg(self.theme.warning).bold(),
                Line::from(""),
                Line::from("Press 'a' to add a new task").fg(self.theme.text_dim),
            ],
            FeedbackKind::NoResults(q) => vec![
                Line::from("No matches").fg(self.theme.error).bold(),
                Line::from(format!("Nothing found for '{}'", q)).fg(self.theme.text_dim),
                Line::from("Try another query").fg(self.theme.text_dim),
            ],
            FeedbackKind::SmallTerminal => vec![
                Line::styled(
                    "Terminal is too small!",
                    Style::default().fg(self.theme.error),
                )
                .centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Required: ", Style::default().fg(self.theme.text_dim)),
                    Span::styled(
                        format!("{}", MIN_WIDTH),
                        Style::default().fg(self.theme.error).bold(),
                    ),
                    Span::raw(" x "),
                    Span::styled(
                        format!("{}", MIN_HEIGHT),
                        Style::default().fg(self.theme.error).bold(),
                    ),
                ])
                .centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Current:  ", Style::default().fg(self.theme.text_dim)),
                    Span::styled(format!("{}", width), Style::default().fg(colors.0).bold()),
                    Span::raw(" x "),
                    Span::styled(format!("{}", height), Style::default().fg(colors.1).bold()),
                ])
                .centered(),
            ],
        };

        message
    }

    /// Get colors for width/height while resized
    fn dimension_colors(&self, area: &Rect) -> (Color, Color) {
        let width_color = if area.width >= MIN_WIDTH {
            self.theme.success
        } else {
            self.theme.error
        };

        let height_color = if area.height >= MIN_HEIGHT {
            self.theme.success
        } else {
            self.theme.error
        };

        (width_color, height_color)
    }
}

/// Unit-tests for feedback widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use ratatui::{layout::Alignment, text::Span};

    #[test]
    fn should_render_message_for_empty_list() {
        let feedback: FeedbackWidget =
            FeedbackWidget::new(FeedbackKind::EmptyList, &ThemeColors::GRUVBOX);

        assert_eq!(feedback.kind, FeedbackKind::EmptyList);
        assert_eq!(feedback.theme, &ThemeColors::GRUVBOX);

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> = feedback.message(
            &width,
            &height,
            (feedback.theme.error, feedback.theme.success),
        );

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].spans[0], Span::from("All clear!"));
        assert_eq!(
            result[2].spans[0],
            Span::from("Press 'a' to add a new task")
        );
    }

    #[test]
    fn should_render_message_for_invalid_query() {
        let query: String = String::from("Test");
        let feedback: FeedbackWidget = FeedbackWidget::new(
            FeedbackKind::NoResults(query.clone()),
            &ThemeColors::GRUVBOX,
        );

        assert_eq!(feedback.kind, FeedbackKind::NoResults(query.clone()));
        assert_eq!(feedback.theme, &ThemeColors::GRUVBOX);

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> = feedback.message(
            &width,
            &height,
            (feedback.theme.error, feedback.theme.success),
        );

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].spans[0], Span::from("No matches"));
        assert_eq!(result[2].spans[0], Span::from("Try another query"));
    }

    #[test]
    fn should_render_message_for_small_terminal() {
        let feedback: FeedbackWidget =
            FeedbackWidget::new(FeedbackKind::SmallTerminal, &ThemeColors::GRUVBOX);

        assert_eq!(feedback.kind, FeedbackKind::SmallTerminal);
        assert_eq!(feedback.theme, &ThemeColors::GRUVBOX);

        let width: u16 = 80;
        let height: u16 = 24;

        let theme: ThemeColors = Theme::Gruvbox.colors();
        let colors: (Color, Color) = (theme.error, theme.success);
        let result: Vec<Line> = feedback.message(&width, &height, colors);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].spans[0], Span::from("Terminal is too small!"));
        assert_eq!(result[0].alignment, Some(Alignment::Center));
        assert_eq!(
            result[2].spans,
            vec![
                Span::styled("Required: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default().fg(theme.error).bold()
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default().fg(theme.error).bold()
                ),
            ],
        );
        assert_eq!(result[2].alignment, Some(Alignment::Center));
        assert_eq!(
            result[4].spans,
            vec![
                Span::styled("Current:  ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{}", width), Style::default().fg(colors.0).bold()),
                Span::raw(" x "),
                Span::styled(format!("{}", height), Style::default().fg(colors.1).bold()),
            ],
        );
        assert_eq!(result[4].alignment, Some(Alignment::Center));
    }
}
