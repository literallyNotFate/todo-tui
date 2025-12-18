use super::input::Input;
use ratatui::text::Line;

// Get input title (based on input type)
pub fn get_input_title(input: Input) -> Line<'static> {
    use super::input::InputMode;
    use ratatui::{
        style::{Modifier, Style},
        text::Span,
    };

    if input.styles.show_title {
        if let Some(ref user_title) = input.title {
            Line::from(Span::styled(
                format!(" {} ", user_title),
                Style::default()
                    .fg(input.styles.fg_color)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            let defaults: String = match input.mode {
                InputMode::Edit => " Rename a todo ".to_string(),
                InputMode::Insert => " Append a todo ".to_string(),
            };

            Line::from(defaults)
        }
    } else {
        Line::default()
    }
}
