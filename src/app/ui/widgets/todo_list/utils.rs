use crate::app::models::todo::Todo;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, List},
};

// Pre-rendered lines based on todo list
pub fn lines_based_on_list<'a>(
    todos: &[Todo],
    selected: Option<usize>,
) -> (Line<'a>, Line<'a>, Line<'a>) {
    use ratatui::text::Span;

    let top_line: Line = Line::from(Span::styled(
        " List of what's to complete ",
        Style::default()
            .fg(Color::Rgb(252, 252, 252))
            .add_modifier(Modifier::BOLD),
    ));

    let bottom_center_line: Line = Line::from(vec![
        Span::styled(" Help", Style::default().fg(Color::Rgb(252, 252, 252))),
        Span::styled(
            " <?> ",
            Style::default()
                .fg(Color::Rgb(165, 252, 115))
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .centered();

    let todos_length: usize = todos.len();

    let effective_index: usize = selected
        .map(|i| i.min(todos_length.saturating_sub(1)))
        .unwrap_or(0);

    let bottom_left_line: Line = if todos_length == 0 {
        Line::from(Span::styled(
            " 0 / 0 ",
            Style::default().fg(Color::Rgb(252, 252, 252)),
        ))
    } else {
        let current_num = if todos_length == 0 {
            0
        } else {
            effective_index + 1
        };

        let status_text = if todos_length == 0 {
            ""
        } else {
            todos
                .get(effective_index)
                .map(|todo| {
                    if todo.done {
                        "Completed "
                    } else {
                        "Incomplete "
                    }
                })
                .unwrap_or("")
        };

        let status_color: Color = if status_text == "Completed " {
            Color::Rgb(165, 252, 115)
        } else {
            Color::Rgb(255, 180, 180)
        };

        Line::from(vec![
            Span::styled(
                format!(" {} / {} ", current_num, todos_length),
                Style::default().fg(Color::Rgb(252, 252, 252)),
            ),
            Span::raw(" -> "),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    }
    .left_aligned();

    (top_line, bottom_center_line, bottom_left_line)
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
