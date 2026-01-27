use crate::{
    theme::ThemeColors, traits::Input, ui::WidgetResponse, utils::constants::TEXT_INPUT_MAX_CHARS,
};
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect};

#[derive(Debug, Default, Clone)]
pub struct TextInput {
    pub title: String,
    pub buffer: String,
    pub cursor: usize,
    pub max_chars: usize,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            buffer: String::default(),
            title: String::default(),
            cursor: 0,
            max_chars: TEXT_INPUT_MAX_CHARS,
        }
    }

    pub fn from(buffer: impl Into<String>) -> Self {
        let initial_buffer: String = buffer.into();
        let cursor_value: usize = initial_buffer.len();

        Self {
            buffer: initial_buffer,
            title: String::default(),
            cursor: cursor_value,
            max_chars: TEXT_INPUT_MAX_CHARS,
        }
    }

    fn scroll(&self, area_width: usize) -> usize {
        if self.cursor < area_width {
            0
        } else {
            self.cursor - area_width + 1
        }
    }

    fn displayed_content(&self, area_width: usize, scroll: usize) -> String {
        self.buffer.chars().skip(scroll).take(area_width).collect()
    }
}

impl Input for TextInput {
    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse {
        match key {
            KeyCode::Enter => return WidgetResponse::Submit,
            KeyCode::Esc => return WidgetResponse::Cancel,
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.buffer.remove(self.cursor);
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Char(c) => {
                if self.buffer.len() < self.max_chars {
                    self.buffer.insert(self.cursor, *c);
                    self.cursor += 1;
                }
            }
            _ => {}
        }

        WidgetResponse::Continue
    }

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &ThemeColors) {
        use ratatui::{
            layout::Position,
            style::Style,
            widgets::{Block, Paragraph},
        };

        let width: usize = area.width.saturating_sub(2) as usize;
        let scroll: usize = self.scroll(width);
        let text: String = self.displayed_content(width, scroll);

        let focused_style: Style = if focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.border)
        };

        let input_block = Block::bordered()
            .border_style(focused_style)
            .title(self.title.as_str())
            .title_style(Style::default().fg(theme.text_primary));

        let paragraph = Paragraph::new(text).block(input_block);
        frame.render_widget(paragraph, area);

        if focused {
            frame.set_cursor_position(Position::new(
                area.x + (self.cursor - scroll) as u16 + 1,
                area.y + 1,
            ));
        }
    }
}

// Unit-tests for text input
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;

    #[test]
    fn should_handle_input_key_input() {
        let mut input = TextInput::new();

        input.handle_key(&KeyCode::Char('T'));
        input.handle_key(&KeyCode::Char('a'));
        input.handle_key(&KeyCode::Char('s'));
        input.handle_key(&KeyCode::Char('k'));
        assert_eq!(input.buffer, "Task");
        assert_eq!(input.cursor, 4);

        input.handle_key(&KeyCode::Backspace);
        assert_eq!(input.buffer, "Tas");
        assert_eq!(input.cursor, 3);

        input.cursor = 1;
        input.handle_key(&KeyCode::Char('e'));
        assert_eq!(input.buffer, "Teas");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn should_handle_input_from() {
        let mut input = TextInput::from("Task");

        assert_eq!(input.buffer, "Task");
        assert_eq!(input.cursor, 4);

        input.handle_key(&KeyCode::Backspace);
        assert_eq!(input.buffer, "Tas");
        assert_eq!(input.cursor, 3);
    }

    #[test]
    fn should_test_max_chars_of_input() {
        let mut input = TextInput::new();
        input.max_chars = 2;

        input.handle_key(&KeyCode::Char('a'));
        input.handle_key(&KeyCode::Char('b'));
        input.handle_key(&KeyCode::Char('c'));

        assert_eq!(input.buffer, "ab");
        assert_eq!(input.buffer.len(), 2);
    }

    #[test]
    fn should_handle_cursor_movement() {
        let mut input = TextInput::from("Hi");

        input.handle_key(&KeyCode::Right);
        assert_eq!(input.cursor, 2);

        input.handle_key(&KeyCode::Left);
        assert_eq!(input.cursor, 1);

        input.handle_key(&KeyCode::Left);
        assert_eq!(input.cursor, 0);

        input.handle_key(&KeyCode::Left);
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn should_scroll_input_and_display_text() {
        let mut input = TextInput::from("12345678");
        input.cursor = 8;

        let width = 5;
        let scroll = input.scroll(width);
        assert_eq!(scroll, 4);

        let display = input.displayed_content(width, scroll);
        assert_eq!(display, "5678");
    }

    #[test]
    fn should_be_no_scroll_when_text_fits() {
        let input = TextInput::from("ABC");
        let width = 10;
        let scroll = input.scroll(width);

        assert_eq!(scroll, 0);
        assert_eq!(input.displayed_content(width, scroll), "ABC");
    }
}
