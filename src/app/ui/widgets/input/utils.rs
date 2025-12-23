use super::input::InputMode;
use ratatui::text::Line;

// Get input title (based on input type)
pub fn get_input_titles<'a>(
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
