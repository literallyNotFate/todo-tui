use ratatui::text::Line;

// Pre-rendered lines based on notifications
pub fn lines_based_on_notifcation<'a>(seconds: u64) -> (Line<'a>, Line<'a>) {
    use ratatui::{
        style::{Color, Modifier, Style},
        text::Span,
    };

    let top_line: Line = Line::styled(
        " Notification ",
        Style::default()
            .fg(Color::Rgb(252, 252, 252))
            .add_modifier(Modifier::BOLD),
    )
    .centered();

    let bottom_line: Line = Line::from(vec![
        Span::styled(
            " Closes in ",
            Style::default().fg(Color::Rgb(252, 252, 252)),
        ),
        Span::styled(
            format!("{} seconds ", seconds),
            Style::default()
                .fg(Color::Rgb(252, 244, 0))
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .centered();

    (top_line, bottom_line)
}
