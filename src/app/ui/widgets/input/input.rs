use crate::app::{
    ui::renderer::state::Anchor,
    utils::constants::{
        size::{INPUT_HEIGHT, INPUT_MAX_CHARS, INPUT_WIDTH},
        theme::*,
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
        use crate::app::utils::layout::anchored;
        anchored(frame_area, INPUT_WIDTH, INPUT_HEIGHT, self.anchor.clone())
    }

    // Rendering
    pub fn render(self, frame: &mut Frame, area: Rect) {
        use super::utils::render_input_titles;
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
