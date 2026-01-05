use super::confirm::ConfirmOption;
use ratatui::{style::Modifier, text::Line};

// Render confirm buttons
pub fn render_confirm_buttons(selected: ConfirmOption) -> Line<'static> {
    use ratatui::style::{Color, Style};
    use ratatui::text::Span;

    const TEXT_NORMAL: Color = Color::Rgb(252, 252, 252);
    const TEXT_DIMMED: Color = Color::Rgb(150, 150, 150);
    const YES_SELECTED: Color = Color::Rgb(155, 201, 166);
    const CANCEL_SELECTED: Color = Color::Rgb(201, 155, 155);

    let (yes_style, cancel_style): (Style, Style) = match selected {
        ConfirmOption::Yes => (
            Style::default()
                .fg(YES_SELECTED)
                .add_modifier(Modifier::BOLD),
            Style::default().fg(TEXT_DIMMED),
        ),
        ConfirmOption::Cancel => (
            Style::default().fg(TEXT_DIMMED),
            Style::default()
                .fg(CANCEL_SELECTED)
                .add_modifier(Modifier::BOLD),
        ),
    };

    Line::from(vec![
        Span::styled("[ ", Style::default().fg(TEXT_NORMAL)),
        Span::styled("Yes", yes_style),
        Span::styled(" ]", Style::default().fg(TEXT_NORMAL)),
        Span::raw("   "),
        Span::styled("[ ", Style::default().fg(TEXT_NORMAL)),
        Span::styled("Cancel", cancel_style),
        Span::styled(" ]", Style::default().fg(TEXT_NORMAL)),
    ])
}
