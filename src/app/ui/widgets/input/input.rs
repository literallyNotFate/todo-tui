use crate::app::ui::state::WidgetPosition;
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, style::Color, widgets::Padding};

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
    pub show_title: bool,
}

// Main input modal window
#[derive(Default, Clone)]
pub struct Input {
    pub title: Option<String>,
    pub buffer: String,
    pub cursor: usize,
    pub mode: InputMode,
    pub position: WidgetPosition,

    pub styles: InputStyles,
}

// Input methods inplementation
impl Input {
    // Maximum characters input value
    pub const MAX_CHARS: usize = 46;

    // Create insert input modal
    pub fn insert() -> Self {
        Self {
            buffer: "".to_string(),
            title: None,
            cursor: 0,
            mode: InputMode::Insert,
            position: WidgetPosition::Center,
            styles: InputStyles {
                fg_color: Color::Rgb(245, 161, 145),
                padding: Padding::new(1, 1, 0, 0),
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
            position: WidgetPosition::Center,
            styles: InputStyles {
                fg_color: Color::Rgb(234, 141, 165),
                padding: Padding::new(1, 1, 0, 0),
                show_title: true,
            },
        }
    }

    // Calculate area for input
    pub fn area(&self, frame_area: Rect) -> Rect {
        use crate::app::utils::layout::position_area;
        position_area(frame_area, 50, 3, self.position.clone())
    }

    // Render
    pub fn render(self, frame: &mut Frame, area: Rect) {
        use super::utils::get_input_titles;
        use ratatui::{
            layout::Position,
            style::{Color, Style},
            text::Line,
            widgets::{Block, BorderType, Paragraph},
        };

        let titles: (Line, Line) = get_input_titles(
            self.title,
            self.mode,
            self.styles.show_title,
            self.buffer.chars().count(),
            Self::MAX_CHARS,
        );

        let input_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.styles.fg_color))
            .padding(self.styles.padding)
            .title(titles.0)
            .title_bottom(titles.1)
            .title_style(Style::default().fg(Color::Rgb(252, 252, 252)));

        let input = Paragraph::new(self.buffer)
            .style(Style::default().fg(self.styles.fg_color))
            .block(input_block);

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
                if self.buffer.len() < Self::MAX_CHARS {
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

    pub fn position(mut self, position: WidgetPosition) -> Self {
        self.position = position;
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

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.styles.padding = padding;
        self
    }
}
