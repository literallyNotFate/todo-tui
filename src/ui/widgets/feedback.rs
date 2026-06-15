use crate::{
    core::Action,
    theme::ThemePalette,
    ui::{MIN_HEIGHT, MIN_WIDTH, RenderContext, center},
};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
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

    /// Feedback rendering
    pub fn render(self, ctx: &mut RenderContext, area: Rect) {
        let result_area: Rect = center(area, 50, 50);
        let palette: ThemePalette = ctx.palette();
        let colors: (Color, Color) = self.dimension_colors(&area, &palette);

        if self.kind == FeedbackKind::SmallTerminal {
            let message = self.message(&area.width, &area.height, colors, &palette, false);

            ctx.render_widget(Clear, area);
            ctx.render_widget(Block::default().bg(palette.bg), area);

            let p: Paragraph = Paragraph::new(Text::from(message))
                .style(Style::default().fg(palette.fg))
                .wrap(Wrap { trim: false })
                .centered();

            ctx.render_widget(p, result_area);
            return;
        }

        let message = self.message(&area.width, &area.height, colors, &palette, ctx.is_dimmed);
        let hotkeys = if self.kind == FeedbackKind::EmptyList {
            Line::from(self.render_hotkeys(ctx)).centered()
        } else {
            Line::from("")
        };

        let block = Block::bordered()
            .title(" Tasks ")
            .bg(palette.bg)
            .title_top(
                Line::styled(" toodles ", Style::default().fg(ctx.color(palette.fg)))
                    .right_aligned(),
            )
            .title_bottom(hotkeys)
            .border_style(Style::default().fg(ctx.color(palette.warning)))
            .border_type(ctx.config.ui.border_type.into());

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
        palette: &ThemePalette,
        is_dimmed: bool,
    ) -> Vec<Line<'static>> {
        let color = |c: Color| {
            if is_dimmed { palette.muted } else { c }
        };

        match &self.kind {
            FeedbackKind::EmptyList => vec![
                Line::from("All clear!").fg(color(palette.warning)).bold(),
                Line::from(""),
                Line::from("Press 'a' to add a new task").fg(palette.muted),
            ],
            FeedbackKind::NoResults(q) => vec![
                Line::from("No matches").fg(color(palette.error)).bold(),
                Line::from(format!("Nothing found for '{}'", q)).fg(palette.muted),
                Line::from("Try another query").fg(palette.muted),
            ],
            FeedbackKind::SmallTerminal => vec![
                Line::styled("Terminal is too small!", Style::default().fg(palette.error))
                    .centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Required: ", Style::default().fg(palette.muted)),
                    Span::styled(
                        format!("{}", MIN_WIDTH),
                        Style::default().fg(palette.error).bold(),
                    ),
                    Span::raw(" x "),
                    Span::styled(
                        format!("{}", MIN_HEIGHT),
                        Style::default().fg(palette.error).bold(),
                    ),
                ])
                .centered(),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Current:  ", Style::default().fg(palette.muted)),
                    Span::styled(format!("{}", width), Style::default().fg(colors.0).bold()),
                    Span::raw(" x "),
                    Span::styled(format!("{}", height), Style::default().fg(colors.1).bold()),
                ])
                .centered(),
            ],
        }
    }

    /// Get colors for width/height while resized
    fn dimension_colors(&self, area: &Rect, palette: &ThemePalette) -> (Color, Color) {
        let width_color = if area.width >= MIN_WIDTH {
            palette.success
        } else {
            palette.error
        };

        let height_color = if area.height >= MIN_HEIGHT {
            palette.success
        } else {
            palette.error
        };

        (width_color, height_color)
    }

    /// Render hotkeys for feedback (empty list)
    fn render_hotkeys(&self, ctx: &RenderContext) -> Line<'static> {
        let mut spans = Vec::new();
        let palette: ThemePalette = ctx.palette();

        spans.extend(ctx.key_hint(Action::ShowHelp, "help", palette.accent));
        spans.extend(ctx.key_hint(Action::AddTask, "add", palette.secondary));
        spans.extend(ctx.key_hint(Action::MoveLeft, "sidebar", palette.success));
        spans.extend(ctx.key_hint(Action::Quit, "exit", palette.error));

        Line::from(spans).centered()
    }
}

/// Unit-tests for feedback widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;
    use ratatui::{layout::Alignment, text::Span};

    #[test]
    fn should_render_message_for_empty_list() {
        let feedback: FeedbackWidget = FeedbackWidget::new(FeedbackKind::EmptyList);
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();

        assert_eq!(feedback.kind, FeedbackKind::EmptyList);

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> = feedback.message(
            &width,
            &height,
            (palette.error, palette.success),
            &palette,
            false,
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
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let feedback: FeedbackWidget = FeedbackWidget::new(FeedbackKind::NoResults(query.clone()));

        assert_eq!(feedback.kind, FeedbackKind::NoResults(query.clone()));

        let width: u16 = 100;
        let height: u16 = 50;

        let result: Vec<Line> = feedback.message(
            &width,
            &height,
            (palette.error, palette.success),
            &palette,
            false,
        );

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

        let palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let colors: (Color, Color) = (palette.error, palette.success);
        let result: Vec<Line> = feedback.message(&width, &height, colors, &palette, false);

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].spans[0], Span::from("Terminal is too small!"));
        assert_eq!(result[0].alignment, Some(Alignment::Center));
        assert_eq!(
            result[2].spans,
            vec![
                Span::styled("Required: ", Style::default().fg(palette.muted)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default().fg(palette.error).bold()
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default().fg(palette.error).bold()
                ),
            ],
        );
        assert_eq!(result[2].alignment, Some(Alignment::Center));
        assert_eq!(
            result[4].spans,
            vec![
                Span::styled("Current:  ", Style::default().fg(palette.muted)),
                Span::styled(format!("{}", width), Style::default().fg(colors.0).bold()),
                Span::raw(" x "),
                Span::styled(format!("{}", height), Style::default().fg(colors.1).bold()),
            ],
        );
        assert_eq!(result[4].alignment, Some(Alignment::Center));
    }
}
