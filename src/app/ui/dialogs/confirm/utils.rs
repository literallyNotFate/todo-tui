use super::confirm::ConfirmOption;
use crate::app::utils::constants::theme::*;
use ratatui::{style::Modifier, text::Line};

// Render confirm buttons
pub fn render_confirm_buttons(selected: ConfirmOption) -> Line<'static> {
    use ratatui::style::Style;
    use ratatui::text::Span;

    let (yes_style, cancel_style): (Style, Style) = match selected {
        ConfirmOption::Yes => (
            Style::default()
                .fg(CONFIRM_YES_FG_ACTIVE)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(TEXT_DIMMED),
        ),
        ConfirmOption::Cancel => (
            Style::default().fg(TEXT_DIMMED),
            Style::default()
                .fg(CONFIRM_CANCEL_FG_ACTIVE)
                .add_modifier(Modifier::BOLD),
        ),
    };

    Line::from(vec![
        Span::styled("[ ", Style::default().fg(TEXT_PRIMARY)),
        Span::styled("Yes", yes_style),
        Span::styled(" ]", Style::default().fg(TEXT_PRIMARY)),
        Span::raw("   "),
        Span::styled("[ ", Style::default().fg(TEXT_PRIMARY)),
        Span::styled("Cancel", cancel_style),
        Span::styled(" ]", Style::default().fg(TEXT_PRIMARY)),
    ])
}
