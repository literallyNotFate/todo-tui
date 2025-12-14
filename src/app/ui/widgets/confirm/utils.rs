use ratatui::text::Line;

// Render confirm buttons
pub fn get_confirm_buttons(selected: bool) -> Line<'static> {
    use ratatui::style::{Color, Style};
    use ratatui::text::Span;

    if selected {
        Line::from(vec![
            Span::styled("[ Yes ]", Style::default().fg(Color::Rgb(155, 201, 166))),
            Span::raw("   "),
            Span::styled("Cancel", Style::default().fg(Color::Rgb(252, 252, 252))),
        ])
    } else {
        Line::from(vec![
            Span::styled("Yes", Style::default().fg(Color::Rgb(252, 252, 252))),
            Span::raw("   "),
            Span::styled("[ Cancel ]", Style::default().fg(Color::Rgb(201, 155, 155))),
        ])
    }
}
