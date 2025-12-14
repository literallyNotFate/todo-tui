use super::utils::get_confirm_buttons;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Stylize},
    widgets::Padding,
};

pub enum ConfirmAction {
    Remove,
    Append(String),
    Rename(String),
}

pub struct ConfirmStyles {
    pub border_color: Color,
    pub padding: Padding,
    pub max_width: Option<u16>,
}

pub struct Confirm {
    pub message: String,
    pub selected: bool,
    pub action: Option<ConfirmAction>,

    pub styles: ConfirmStyles,
}

impl Confirm {
    pub fn new() -> Self {
        Self {
            message: "".to_string(),
            selected: false,
            action: None,

            styles: ConfirmStyles {
                border_color: Color::Rgb(252, 252, 252),
                padding: Padding {
                    top: 3,
                    bottom: 3,
                    left: 2,
                    right: 2,
                },
                max_width: None,
            },
        }
    }

    // Render
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Alignment, Margin},
            style::Style,
            text::Line,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let confirm_block: Block = Block::bordered()
            .fg(Color::Rgb(252, 252, 252))
            .border_style(Style::default().fg(self.styles.border_color))
            .padding(self.styles.padding)
            .border_type(BorderType::Rounded);

        frame.render_widget(confirm_block, area);

        let inner_area: Rect = area.inner(Margin {
            vertical: 2,
            horizontal: 2,
        });

        let message: Paragraph = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(message, inner_area);

        let buttons_area = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height.saturating_sub(2),
            width: inner_area.width,
            height: 2,
        };

        let buttons: Line = get_confirm_buttons(self.selected);

        let buttons_widget: Paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, buttons_area);
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<bool> {
        match key {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.selected = !self.selected;
                None
            }
            KeyCode::Enter => Some(self.selected),
            KeyCode::Esc => Some(false),
            _ => None,
        }
    }

    pub fn get_buttons_length() -> usize {
        get_confirm_buttons(true).iter().len()
    }

    // Chaining API
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn action(mut self, action: ConfirmAction) -> Self {
        self.action = Some(action);
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

    pub fn with_max_width(mut self, width: u16) -> Self {
        self.styles.max_width = Some(width);
        self
    }
}
