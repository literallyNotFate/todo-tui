use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Stylize},
    widgets::Padding,
};

// Defines the status of input (inserting the text, editing existing text)
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Insert,
    Edit,
}

// The result action
pub enum InputResult {
    Continue,
    Submit(String),
    Cancel,
}

// Styles for input
#[derive(Default, Clone)]
pub struct InputStyles {
    pub fg_color: Color,
    pub padding: Padding,
    pub max_chars: usize,
    pub show_title: bool,
}

// Main input modal window
#[derive(Default, Clone)]
pub struct Input {
    pub title: Option<String>,
    pub buffer: String,
    pub cursor: usize,
    pub mode: InputMode,

    pub styles: InputStyles,
}

// Input methods inplementation
impl Input {
    // Create insert input modal
    pub fn insert() -> Self {
        Self {
            buffer: "".to_string(),
            title: None,
            cursor: 0,
            mode: InputMode::Insert,
            styles: InputStyles {
                fg_color: Color::Rgb(245, 161, 145),
                padding: Padding::new(1, 1, 0, 0),
                max_chars: 46,
                show_title: true,
            },
        }
    }

    // Create edit input modal
    pub fn edit(initial: impl Into<String>) -> Self {
        let initial_string: String = initial.into();
        let cursor_value: usize = initial_string.len();

        Self {
            buffer: initial_string,
            title: None,
            cursor: cursor_value,
            mode: InputMode::Edit,
            styles: InputStyles {
                fg_color: Color::Rgb(234, 141, 165),
                padding: Padding::new(1, 1, 0, 0),
                max_chars: 46,
                show_title: true,
            },
        }
    }

    // Calculate area for input
    pub fn area(&self, frame_area: Rect) -> Rect {
        use crate::app::utils::layout::center;
        center(frame_area, 50, 3)
    }

    // Render
    pub fn render(self, frame: &mut Frame, area: Rect) {
        use super::utils::get_input_title;
        use ratatui::{
            layout::Position,
            text::Line,
            widgets::{Block, BorderType, Paragraph},
        };

        let title: Line = get_input_title(self.clone());

        let input = Paragraph::new(self.buffer).fg(self.styles.fg_color).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .padding(self.styles.padding)
                .title(title),
        );

        frame.render_widget(input, area);
        frame.set_cursor_position(Position::new(
            area.x + self.cursor as u16 + self.styles.padding.right + self.styles.padding.left,
            area.y + 1,
        ));
    }

    // Key even handling
    pub fn handle_key(&mut self, key: KeyCode) -> InputResult {
        match key {
            KeyCode::Enter => {
                if !self.buffer.is_empty() {
                    return InputResult::Submit(self.buffer.clone());
                }
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
                if self.buffer.len() < self.styles.max_chars {
                    self.buffer.insert(self.cursor, c);
                    self.cursor += 1;
                }
            }
            _ => {}
        }

        InputResult::Continue
    }

    // Chaining API
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn no_title(mut self) -> Self {
        self.styles.show_title = false;
        self
    }

    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.styles.fg_color = color;
        self
    }

    pub fn with_max_chars(mut self, max: usize) -> Self {
        self.styles.max_chars = max;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.styles.padding = padding;
        self
    }
}
