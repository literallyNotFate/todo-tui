use crate::utils::constants::theme::*;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

// Helper functions for popup
pub mod popup {
    use super::*;
    use crate::ui::{PopupCloseBehavior, PopupKind};

    pub fn color_based_on_popup_kind(kind: PopupKind) -> Color {
        match kind {
            PopupKind::Error => ERROR_POPUP_FG,
            PopupKind::Success => SUCCESS_POPUP_FG,
            PopupKind::Help => HELP_POPUP_FG,
            PopupKind::Info => INFO_POPUP_FG,
        }
    }

    pub fn render_lines_based_on_popup<'a>(
        title: Option<String>,
        kind: PopupKind,
        close_behavior: PopupCloseBehavior,
        show_title: bool,
    ) -> (Line<'a>, Line<'a>) {
        let top_line: Line = if show_title {
            if let Some(title) = title {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                let defaults: &str = match kind {
                    PopupKind::Help => " Help ",
                    PopupKind::Error => " Error ",
                    PopupKind::Success => " Success ",
                    PopupKind::Info => " Info ",
                };
                Line::from(Span::styled(
                    defaults,
                    Style::default()
                        .fg(TEXT_PRIMARY)
                        .add_modifier(Modifier::BOLD),
                ))
            }
        } else {
            Line::default()
        };

        let key: String = match close_behavior {
            PopupCloseBehavior::AnyKey => "any key".to_string(),
            PopupCloseBehavior::Specific(c) => format!("<{}>", c),
            _ => "".to_string(),
        };

        let bottom_line: Line = if close_behavior != PopupCloseBehavior::None {
            Line::from(vec![
                Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
                Span::styled(
                    key,
                    Style::default()
                        .fg(COLOR_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
            ])
        } else {
            Line::default()
        };

        (top_line, bottom_line)
    }
}

// Helper functions for confirm
pub mod confirm {
    use super::*;
    use crate::ui::ConfirmOption;

    pub fn render_confirm_buttons(selected: ConfirmOption) -> Line<'static> {
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
}

// Helper functions for input
pub mod input {
    use super::*;
    use crate::ui::InputMode;

    pub fn render_input_titles<'a>(
        title: Option<String>,
        mode: InputMode,
        show_title: bool,
        buffer_len: usize,
        max_chars: usize,
    ) -> (Line<'a>, Line<'a>) {
        use ratatui::{
            layout::Alignment,
            style::{Modifier, Style},
        };

        let chars_count_line: Line = Line::from(format!(" {} / {} ", buffer_len, max_chars))
            .alignment(Alignment::Right)
            .style(Style::default().add_modifier(Modifier::BOLD));

        if show_title {
            if let Some(title) = title {
                (
                    Line::from(format!(" {} ", title))
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                    chars_count_line,
                )
            } else {
                let defaults: String = match mode {
                    InputMode::Edit => " Rename a todo ".to_string(),
                    InputMode::Insert => " Append a todo ".to_string(),
                };

                (
                    Line::from(defaults).style(Style::default().add_modifier(Modifier::BOLD)),
                    chars_count_line,
                )
            }
        } else {
            (Line::default(), chars_count_line)
        }
    }
}

// Helper functions for notification
pub mod notification {
    use super::*;

    pub fn render_lines_based_on_notifcation<'a>(seconds: u64) -> (Line<'a>, Line<'a>) {
        use ratatui::{
            style::{Modifier, Style},
            text::Span,
        };

        let top_line: Line = Line::styled(
            " Notification ",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )
        .centered();

        let bottom_line: Line = Line::from(vec![
            Span::styled(" Closes in ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                format!("{} seconds ", seconds),
                Style::default()
                    .fg(COLOR_YELLOW)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .centered();

        (top_line, bottom_line)
    }
}

// Helper functions for todo_list
pub mod todo_list {
    use ratatui::{
        Frame,
        layout::Rect,
        widgets::{Block, List},
    };

    use super::*;
    use crate::models::Todo;

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
}

// Helper functions for fallback
pub mod fallback {
    use super::*;
    use crate::utils::constants::terminal::{MIN_HEIGHT, MIN_WIDTH};

    pub fn render_fallback_message(
        width: u16,
        height: u16,
        colors: (Color, Color),
    ) -> Vec<Line<'static>> {
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
}
