use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List},
};

use crate::app::models::todo::Todo;

// Pre-rendered lines based on todo list
pub fn lines_based_on_list<'a>() -> (Line<'a>, Line<'a>) {
    use ratatui::text::Span;

    let top_line: Line = Line::from(Span::styled(
        " List of what's to complete ",
        Style::default()
            .fg(Color::Rgb(252, 252, 252))
            .add_modifier(Modifier::BOLD),
    ));

    let bottom_line: Line = Line::from(vec![
        Span::styled(" Help", Style::default().fg(Color::Rgb(252, 252, 252))),
        Span::styled(
            " <?> ",
            Style::default()
                .fg(Color::Rgb(165, 252, 115))
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .centered();

    (top_line, bottom_line)
}

// Render list
pub fn generate_stateful_list<'a>(
    todos: &[Todo],
    list_block: Block<'a>,
    highlight_symbol: &'a str,
) -> List<'a> {
    use ratatui::widgets::ListItem;

    List::new(todos.iter().map(|item| {
        let prefix = if item.done { " [✓] " } else { " [ ] " };
        ListItem::new(format!("{}{}", prefix, item.title))
    }))
    .block(list_block)
    .highlight_symbol(highlight_symbol)
    .highlight_style(
        Style::default()
            .fg(Color::Rgb(229, 218, 156))
            .add_modifier(Modifier::BOLD),
    )
}
