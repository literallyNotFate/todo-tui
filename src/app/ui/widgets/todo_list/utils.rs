use crate::app::{models::todo::Todo, utils::constants::theme::*};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, List},
};

// Pre-rendered lines based on todo list
pub fn render_lines_based_on_list<'a>(
    todos: &[Todo],
    selected: Option<usize>,
) -> (Line<'a>, Line<'a>, Line<'a>) {
    use ratatui::text::Span;

    let top_line: Line = Line::from(Span::styled(
        " List of what's to complete ",
        Style::default()
            .fg(TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD),
    ));

    let bottom_center_line: Line = Line::from(vec![
        Span::styled(" Help", Style::default().fg(TEXT_PRIMARY)),
        Span::styled(
            " <?> ",
            Style::default()
                .fg(COLOR_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .centered();

    let todos_length: usize = todos.len();

    let bottom_left_line: Line = if selected.is_some() {
        let effective_index: usize = selected
            .map(|i| i.min(todos_length.saturating_sub(1)))
            .unwrap_or(0);
        let current_num = effective_index + 1;

        let status_text = todos
            .get(effective_index)
            .map(|todo| if todo.done { "Done " } else { "Undone " })
            .unwrap_or("");

        let status_color = if status_text == "Done " {
            COLOR_GREEN
        } else {
            COLOR_RED
        };

        if todos_length != 0 {
            Line::from(vec![
                Span::styled(
                    format!(" {current_num} / {todos_length} "),
                    Style::default().fg(TEXT_PRIMARY),
                ),
                Span::raw(" -> "),
                Span::styled(
                    status_text,
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
            .left_aligned()
        } else {
            Line::default()
        }
    } else {
        Line::default()
    };

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
            .fg(ITEM_LIST_SELECTED)
            .add_modifier(Modifier::BOLD),
    )
}

// Generate scrollbar for list if too many items (or small screen)
pub fn render_scrollbar_if_needed(
    frame: &mut Frame,
    area: Rect,
    content_lines: usize,
    visible_lines: usize,
    offset: usize,
) {
    use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

    if content_lines < visible_lines {
        return;
    }

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(content_lines)
        .viewport_content_length(visible_lines)
        .position(offset);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"))
        .track_symbol(Some("│"))
        .thumb_symbol("▉");

    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
}
