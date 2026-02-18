use crate::{
    theme::ThemeColors,
    ui::{MIN_HEIGHT, MIN_WIDTH, RenderContext, center},
};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph, Wrap},
};

/// Type of feedback message
#[derive(Debug, PartialEq)]
pub enum FeedbackKind {
    EmptyList,
    NoResults(String),
    SmallTerminal,
}

pub struct FeedbackWidget {
    kind: FeedbackKind,
}

impl FeedbackWidget {
    pub fn new(kind: FeedbackKind) -> Self {
        Self { kind }
    }
}

impl FeedbackWidget {
    /// Feedback rendering
    pub fn render(self, ctx: &mut RenderContext, area: Rect) {
        let result_area: Rect = center(area, 50, 50);
        let colors: (Color, Color) = self.dimension_colors(&area, &ctx.theme);
        let message: Vec<Line>;

        if self.kind == FeedbackKind::SmallTerminal {
            message = self.message(&area.width, &area.height, colors, &ctx.theme);
            ctx.render_widget(Clear, area);

            let p: Paragraph = Paragraph::new(Text::from(message))
                .style(Style::default().fg(ctx.theme.text_primary))
                .wrap(Wrap { trim: false })
                .block(Block::default());

            ctx.render_widget(p, result_area);
            return;
        }

        message = self.message(&area.width, &area.height, colors, &ctx.theme);

        let block = Block::bordered()
            .title(" Tasks ")
            .title_top(
                Line::styled(" todo-tui ", Style::default().fg(ctx.theme.text_primary))
                    .right_aligned(),
            )
            .border_style(ctx.theme.warning);

        ctx.render_widget(block, area);

        ctx.render_widget(
            Paragraph::new(message).wrap(Wrap { trim: true }).centered(),
            result_area,
        );
    }

    /// Return message based on feedback kind
    fn message(
        &self,
        width: &u16,
        height: &u16,
        colors: (Color, Color),
        theme: &ThemeColors,
    ) -> Vec<Line<'static>> {
        use ratatui::text::Span;

        let message: Vec<Line> = match &self.kind {
            FeedbackKind::EmptyList => vec![
                Line::from("All clear!").fg(theme.warning).bold(),
                Line::from(""),
                Line::from("Press 'a' to add a new task").fg(theme.text_dim),
            ],
            FeedbackKind::NoResults(q) => vec![
                Line::from("No matches").fg(theme.error).bold(),
                Line::from(format!("Nothing found for '{}'", q)).fg(theme.text_dim),
                Line::from("Try another query").fg(theme.text_dim),
            ],
            FeedbackKind::SmallTerminal => vec![
                Line::styled("Terminal is too small!", Style::default().fg(theme.error)).centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Required: ", Style::default().fg(theme.text_dim)),
                    Span::styled(
                        format!("{}", MIN_WIDTH),
                        Style::default().fg(theme.error).bold(),
                    ),
                    Span::raw(" x "),
                    Span::styled(
                        format!("{}", MIN_HEIGHT),
                        Style::default().fg(theme.error).bold(),
                    ),
                ])
                .centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Current:  ", Style::default().fg(theme.text_dim)),
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
    fn dimension_colors(&self, area: &Rect, theme: &ThemeColors) -> (Color, Color) {
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

/// Unit-tests for feedback widget
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{layout::Alignment, text::Span};

    #[test]
    fn should_render_message_for_empty_list() {
        let feedback: FeedbackWidget = FeedbackWidget::new(FeedbackKind::EmptyList);
        let theme: ThemeColors = ThemeColors::GRUVBOX;

        assert_eq!(feedback.kind, FeedbackKind::EmptyList);

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> =
            feedback.message(&width, &height, (theme.error, theme.success), &theme);

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
        let theme: ThemeColors = ThemeColors::GRUVBOX;
        let feedback: FeedbackWidget = FeedbackWidget::new(FeedbackKind::NoResults(query.clone()));

        assert_eq!(feedback.kind, FeedbackKind::NoResults(query.clone()));

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> =
            feedback.message(&width, &height, (theme.error, theme.success), &theme);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].spans[0], Span::from("No matches"));
        assert_eq!(result[2].spans[0], Span::from("Try another query"));
    }

    #[test]
    fn should_render_message_for_small_terminal() {
        let feedback: FeedbackWidget = FeedbackWidget::new(FeedbackKind::SmallTerminal);

        assert_eq!(feedback.kind, FeedbackKind::SmallTerminal);

        let width: u16 = 80;
        let height: u16 = 24;

        let theme: ThemeColors = ThemeColors::GRUVBOX;
        let colors: (Color, Color) = (theme.error, theme.success);
        let result: Vec<Line> = feedback.message(&width, &height, colors, &theme);

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
