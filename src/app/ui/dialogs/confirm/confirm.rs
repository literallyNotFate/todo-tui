use super::utils::get_confirm_buttons;
use crate::app::ui::{
    dialogs::dialog::{Dialog, DialogResult},
    state::WidgetPosition,
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Stylize},
    widgets::Padding,
};

// Styles for confirm
pub struct ConfirmStyles {
    pub border_color: Color,
    pub padding: Padding,
}

// Main confirm window
pub struct Confirm {
    pub message: String,
    pub selected: bool,
    pub position: WidgetPosition,

    pub styles: ConfirmStyles,
}

// Dialog trait implementation
impl Dialog for Confirm {
    // Default constructor
    fn new() -> Self {
        Self {
            message: "".to_string(),
            selected: false,
            position: WidgetPosition::Center,

            styles: ConfirmStyles {
                border_color: Color::Rgb(252, 252, 252),
                padding: Padding {
                    top: 3,
                    bottom: 3,
                    left: 2,
                    right: 2,
                },
            },
        }
    }

    // Calculate area for confirm
    fn area(&self, frame_area: Rect) -> Rect {
        use crate::app::utils::layout::{calculate_content_size, position_area};

        let (width, height): (u16, u16) = calculate_content_size(
            frame_area,
            &self.message,
            0,
            Confirm::get_buttons_length(),
            self.styles.padding,
            60.0,
        );

        position_area(frame_area, width, height, self.position.clone())
    }

    // Rendering
    fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Alignment, Margin},
            style::Style,
            text::Line,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let confirm_block: Block = Block::bordered()
            .fg(Color::Rgb(252, 252, 252))
            .border_style(Style::default().fg(self.styles.border_color))
            .title_top(Line::from(" Confirm operation ").centered())
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

    // Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult> {
        match key {
            KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                self.selected = !self.selected;
                None
            }
            KeyCode::Enter => Some(if self.selected {
                DialogResult::Confirmed
            } else {
                DialogResult::Cancelled
            }),
            KeyCode::Esc => Some(DialogResult::Cancelled),
            _ => None,
        }
    }
}

// Other methods implementation
impl Confirm {
    // Confirm buttons length
    pub fn get_buttons_length() -> usize {
        get_confirm_buttons(true).iter().len()
    }

    // Chaining API
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn position(mut self, position: WidgetPosition) -> Self {
        self.position = position;
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
