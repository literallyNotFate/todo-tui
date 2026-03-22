use crate::{
    theme::ThemePalette,
    ui::{RenderContext, WidgetResponse, widgets::input::Input},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Style, Stylize},
};

pub const TEXT_INPUT_MAX_CHARS: usize = 256;

/// Input text widget
#[derive(Debug, Clone)]
pub struct TextInput {
    pub title: String,
    pub buffer: String,
    pub cursor: usize,
    pub max_chars: usize,
}

impl TextInput {
    /// New text input
    pub fn new() -> Self {
        Self {
            buffer: String::default(),
            title: String::default(),
            cursor: 0,
            max_chars: TEXT_INPUT_MAX_CHARS,
        }
    }

    /// New edit text input
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

    /// Scrolls if text too big
    fn scroll(&self, area_width: usize) -> usize {
        if self.cursor < area_width {
            0
        } else {
            self.cursor - area_width + 1
        }
    }

    /// Displays the buffer content using scroll
    fn displayed_content(&self, area_width: usize, scroll: usize) -> String {
        self.buffer.chars().skip(scroll).take(area_width).collect()
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for TextInput {
    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Key event handling
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

    /// Resetting input
    fn reset(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    /// Text input rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect, focused: bool) {
        use ratatui::{
            layout::Position,
            widgets::{Block, Paragraph},
        };

        let palette: ThemePalette = ctx.palette();

        let width: usize = area.width.saturating_sub(2) as usize;
        let scroll: usize = self.scroll(width);
        let text: String = self.displayed_content(width, scroll);

        let (border_style, text_style) = self.on_focused(focused, &palette);

        let input_block = Block::bordered()
            .border_style(border_style)
            .border_type(ctx.config.border_type.into())
            .style(text_style)
            .title(self.title.as_str())
            .bg(palette.bg)
            .fg(palette.fg);

        let paragraph = Paragraph::new(text).block(input_block);

        ctx.render_widget(paragraph, area);

        if focused {
            ctx.set_cursor_position(Position::new(
                area.x + (self.cursor - scroll) as u16 + 1,
                area.y + 1,
            ));
        }
    }

    /// Return styles if input is being focused
    fn on_focused(&self, focused: bool, palette: &ThemePalette) -> (Style, Style) {
        if focused {
            (
                Style::default().fg(palette.accent),
                Style::default().fg(palette.fg),
            )
        } else {
            (
                Style::default().fg(palette.muted),
                Style::default().fg(palette.muted),
            )
        }
    }
}

/// Unit-tests for text input
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;
    use ratatui::{crossterm::event::KeyCode, style::Color};

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

    #[test]
    fn should_return_styles_if_focused() {
        let input = TextInput::new();
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut styles: (Style, Style) = input.on_focused(false, &palette);
        assert_eq!(
            styles,
            (
                Style::default().fg(Color::Rgb(146, 131, 116)),
                Style::default().fg(Color::Rgb(146, 131, 116))
            )
        );

        styles = input.on_focused(true, &palette);
        assert_eq!(
            styles,
            (
                Style::default().fg(Color::Rgb(250, 189, 47)),
                Style::default().fg(Color::Rgb(235, 219, 178))
            )
        );
    }

    #[test]
    fn should_handle_reset_text_input() {
        let mut input = TextInput::new();

        input.handle_key(&KeyCode::Char('a'));
        input.handle_key(&KeyCode::Char('b'));
        input.handle_key(&KeyCode::Char('c'));

        assert_eq!(input.buffer, "abc");
        assert_eq!(input.buffer.len(), 3);
        assert_eq!(input.cursor, 3);

        input.reset();
        assert_eq!(input.buffer, "");
        assert_eq!(input.buffer.len(), 0);
        assert_eq!(input.cursor, 0);
    }
}
