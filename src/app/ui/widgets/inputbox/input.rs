use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph},
};

use super::state::{InputMode, InputResult};

#[derive(Clone)]
pub struct InputBoxStyles {
    pub fg_color: Color,
    pub padding: Padding,
    pub max_chars: usize,
    pub show_title: bool,
}

#[derive(Clone)]
pub struct InputBox {
    pub title: Option<String>,
    pub buffer: String,
    pub cursor: usize,
    pub mode: InputMode,

    pub styles: InputBoxStyles,
}

impl InputBox {
    pub fn insert() -> Self {
        Self {
            buffer: "".to_string(),
            title: None,
            cursor: 0,
            mode: InputMode::Insert,
            styles: InputBoxStyles {
                fg_color: Color::Rgb(245, 161, 145),
                padding: Padding::new(1, 1, 0, 0),
                max_chars: 46,
                show_title: true,
            },
        }
    }

    pub fn edit(initial: impl Into<String>) -> Self {
        let initial_string: String = initial.into();
        let cursor_value: usize = initial_string.len();

        Self {
            buffer: initial_string,
            title: None,
            cursor: cursor_value,
            mode: InputMode::Edit,
            styles: InputBoxStyles {
                fg_color: Color::Rgb(234, 141, 165),
                padding: Padding::new(1, 1, 0, 0),
                max_chars: 46,
                show_title: true,
            },
        }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let title: Line = if self.styles.show_title {
            if let Some(ref user_title) = self.title {
                Line::from(Span::styled(
                    format!(" {} ", user_title),
                    Style::default()
                        .fg(self.styles.fg_color)
                        .add_modifier(Modifier::BOLD),
                ))
            } else {
                let defaults: String = match self.mode {
                    InputMode::Edit => " Rename a todo ".to_string(),
                    InputMode::Insert => " Append a todo ".to_string(),
                };

                Line::from(defaults)
            }
        } else {
            Line::default()
        };

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
