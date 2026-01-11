use ratatui::{style::Color, text::Line};

// Render fallback message based on frame size
pub fn render_fallback_message(
    width: u16,
    height: u16,
    colors: (Color, Color),
) -> Vec<Line<'static>> {
    use crate::app::utils::constants::{terminal::*, theme::*};
    use ratatui::{
        style::{Modifier, Style},
        text::Span,
    };

    let message: Vec<Line> = vec![
        Line::styled("Terminal is too small!", Style::default().fg(COLOR_RED)).centered(),
        Line::from(""),
        Line::from(vec![
            Span::styled("Required: ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                format!("{}", MIN_WIDTH),
                Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" x "),
            Span::styled(
                format!("{}", MIN_HEIGHT),
                Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
            ),
        ])
        .centered(),
        Line::from(""),
        Line::from(vec![
            Span::raw("Current:  "),
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
