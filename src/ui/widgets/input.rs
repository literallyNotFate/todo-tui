use crate::{
    state::Anchor,
    utils::{
        constants::{
            size::{INPUT_HEIGHT, INPUT_MAX_CHARS, INPUT_WIDTH},
            theme::{INPUT_ADD_FG, INPUT_EDIT_FG, TEXT_PRIMARY},
        },
        widgets::input::render_input_titles,
    },
};
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, style::Color, widgets::Padding};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Insert,
    Edit,
}

#[derive(Debug, PartialEq)]
pub enum InputResult {
    Continue,
    Submit(String),
    Cancel,
}

#[derive(Default, Clone)]
pub struct InputStyles {
    pub border_color: Color,
    pub padding: Padding,
    pub show_title: bool,
}

#[derive(Default, Clone)]
pub struct Input {
    pub title: Option<String>,
    pub buffer: String,
    pub cursor: usize,
    pub mode: InputMode,
    pub anchor: Anchor,
    pub styles: InputStyles,
}

impl Input {
    pub fn insert() -> Self {
        Self {
            buffer: "".to_string(),
            title: None,
            cursor: 0,
            mode: InputMode::Insert,
            anchor: Anchor::Center,
            styles: InputStyles {
                border_color: INPUT_ADD_FG,
                padding: Padding::new(1, 1, 0, 0),
                show_title: true,
            },
        }
    }

    pub fn edit(buffer: impl Into<String>) -> Self {
        let initial_buffer: String = buffer.into();
        let cursor_value: usize = initial_buffer.len();

        Self {
            buffer: initial_buffer,
            title: None,
            cursor: cursor_value,
            mode: InputMode::Edit,
            anchor: Anchor::Center,
            styles: InputStyles {
                border_color: INPUT_EDIT_FG,
                padding: Padding::new(1, 1, 0, 0),
                show_title: true,
            },
        }
    }

    // Calculate area for input
    pub fn area(&self, frame_area: Rect) -> Rect {
        use crate::utils::anchored;
        anchored(frame_area, INPUT_WIDTH, INPUT_HEIGHT, self.anchor.clone())
    }

    // Rendering
    pub fn render(self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::Position,
            style::Style,
            text::Line,
            widgets::{Block, BorderType, Paragraph},
        };

        let titles: (Line, Line) = render_input_titles(
            self.title,
            self.mode,
            self.styles.show_title,
            self.buffer.chars().count(),
            INPUT_MAX_CHARS,
        );

        let input_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.styles.border_color))
            .padding(self.styles.padding)
            .title(titles.0)
            .title_bottom(titles.1)
            .title_style(Style::default().fg(TEXT_PRIMARY));

        let input = Paragraph::new(self.buffer)
            .style(Style::default().fg(self.styles.border_color))
            .block(input_block);

        frame.render_widget(input, area);
        frame.set_cursor_position(Position::new(
            area.x + self.cursor as u16 + self.styles.padding.right + self.styles.padding.left,
            area.y + 1,
        ));
    }

    // Key event handling
    pub fn handle_key(&mut self, key: KeyCode) -> InputResult {
        match key {
            KeyCode::Enter => {
                return InputResult::Submit(self.buffer.clone());
            }
            KeyCode::Esc => {
                return InputResult::Cancel;
            }
            KeyCode::Delete => {
                if self.cursor < self.buffer.len() {
                    self.buffer.remove(self.cursor);
                }
            }
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
                if self.buffer.len() < INPUT_MAX_CHARS {
                    self.buffer.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
            _ => {}
        }

        InputResult::Continue
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn no_title(mut self) -> Self {
        self.styles.show_title = false;
        self
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.styles.border_color = color;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.styles.padding = padding;
        self
    }
}

// Unit-tests for input widget
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Alignment;

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 30)
    }

    #[test]
    fn should_create_input_insert() {
        let input: Input = Input::insert();

        assert_eq!(input.buffer, "");
        assert_eq!(input.cursor, 0);
        assert_eq!(input.mode, InputMode::Insert);
        assert_eq!(input.anchor, Anchor::Center);
        assert!(input.title.is_none());
        assert!(input.styles.show_title);
        assert_eq!(input.styles.border_color, INPUT_ADD_FG);
    }

    #[test]
    fn should_create_input_insert_with_value() {
        let input: Input = Input::edit("Hello World");

        assert_eq!(input.buffer, "Hello World");
        assert_eq!(input.cursor, 11);
        assert_eq!(input.mode, InputMode::Edit);
        assert_eq!(input.styles.border_color, INPUT_EDIT_FG);
    }

    #[test]
    fn should_create_input_with_chaining_api() {
        let input: Input = Input::insert()
            .title("Custom Title")
            .no_title()
            .with_border_color(Color::Yellow)
            .with_padding(Padding::new(2, 3, 1, 1))
            .anchor(Anchor::TopLeft);

        assert_eq!(input.title, Some("Custom Title".to_string()));
        assert!(!input.styles.show_title);
        assert_eq!(input.styles.border_color, Color::Yellow);
        assert_eq!(input.styles.padding, Padding::new(2, 3, 1, 1));
        assert_eq!(input.anchor, Anchor::TopLeft);
    }

    #[test]
    fn should_create_area_for_input() {
        let input: Input = Input::insert();
        let frame_area: Rect = create_helper_frame();

        let area: Rect = input.area(frame_area);

        assert_eq!(area.width, INPUT_WIDTH);
        assert_eq!(area.height, INPUT_HEIGHT);
        assert_eq!(area.x, 25);
        assert_eq!(area.y, 14);
    }

    #[test]
    fn should_input_handle_key_char_insert() {
        let mut input: Input = Input::insert();
        let result: InputResult = input.handle_key(KeyCode::Char('a'));

        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.buffer, "a");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn should_input_handle_key_backspace() {
        let mut input: Input = Input::edit("abc");

        input.handle_key(KeyCode::Left);
        input.handle_key(KeyCode::Left);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.buffer, "bc");
        assert_eq!(input.cursor, 0);

        input.handle_key(KeyCode::Right);
        input.handle_key(KeyCode::Right);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.buffer, "b");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn should_input_handle_key_enter_and_esc() {
        let mut input: Input = Input::edit("Test");

        let result_enter: InputResult = input.handle_key(KeyCode::Enter);
        assert!(matches!(result_enter, InputResult::Submit(s) if s == "Test"));

        let mut input_cancel: Input = Input::insert();
        let result_esc: InputResult = input_cancel.handle_key(KeyCode::Esc);
        assert_eq!(result_esc, InputResult::Cancel);
    }

    #[test]
    fn should_test_max_chars_limit_for_input() {
        let mut input: Input = Input::insert();

        for _ in 0..INPUT_MAX_CHARS {
            input.handle_key(KeyCode::Char('x'));
        }

        assert_eq!(input.buffer.len(), INPUT_MAX_CHARS);
        assert_eq!(input.cursor, INPUT_MAX_CHARS);

        let result = input.handle_key(KeyCode::Char('y'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.buffer.len(), INPUT_MAX_CHARS);
        assert_eq!(input.cursor, INPUT_MAX_CHARS);
    }

    #[test]
    fn should_render_input_titles() {
        let (top, bottom) =
            render_input_titles(Some("Test".to_string()), InputMode::Insert, true, 10, 46);

        assert_eq!(top.spans.len(), 1);
        assert_eq!(top.spans[0].content, " Test ");

        assert_eq!(bottom.spans.len(), 1);
        assert_eq!(bottom.spans[0].content, " 10 / 46 ");
        assert_eq!(bottom.alignment, Some(Alignment::Right));
    }
}
