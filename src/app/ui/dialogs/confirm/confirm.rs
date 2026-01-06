use super::utils::render_confirm_buttons;
use crate::app::{
    ui::{
        dialogs::dialog::{Dialog, DialogResult},
        renderer::state::WidgetPosition,
    },
    utils::colors::theme::*,
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

// Selection options
#[derive(Clone, PartialEq)]
pub enum ConfirmOption {
    Yes,
    Cancel,
}

// Main confirm window
pub struct Confirm {
    pub message: String,
    pub select: ConfirmOption,
    pub position: WidgetPosition,

    pub styles: ConfirmStyles,
}

// Dialog trait implementation
impl Dialog for Confirm {
    // Default constructor
    fn new() -> Self {
        Self {
            message: "".to_string(),
            select: ConfirmOption::Cancel,
            position: WidgetPosition::Center,

            styles: ConfirmStyles {
                border_color: TEXT_PRIMARY,
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

        let (top_len, bottom_len): (usize, usize) = self.titles_len();
        let (width, height): (u16, u16) = calculate_content_size(
            frame_area,
            &self.message,
            top_len,
            bottom_len,
            self.styles.padding,
            60.0,
        );

        position_area(frame_area, width, height, self.position.clone())
    }

    // Calculate titles length
    fn titles_len(&self) -> (usize, usize) {
        (0, render_confirm_buttons(ConfirmOption::Yes).width())
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
            .fg(TEXT_PRIMARY)
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

        let buttons: Line = render_confirm_buttons(self.select.clone());

        let buttons_widget: Paragraph = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, buttons_area);
    }

    // Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult> {
        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.select = ConfirmOption::Yes;
                None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.select = ConfirmOption::Cancel;
                None
            }
            KeyCode::Enter => Some(match self.select {
                ConfirmOption::Yes => DialogResult::Confirmed,
                ConfirmOption::Cancel => DialogResult::Cancelled,
            }),
            KeyCode::Esc => Some(DialogResult::Cancelled),
            KeyCode::Char('y') => {
                self.select = ConfirmOption::Yes;
                Some(DialogResult::Confirmed)
            }
            KeyCode::Char('n') => {
                self.select = ConfirmOption::Cancel;
                Some(DialogResult::Cancelled)
            }
            _ => None,
        }
    }
}

// Other methods implementation
impl Confirm {
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
