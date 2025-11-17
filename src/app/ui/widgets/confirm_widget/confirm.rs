use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Alignment, Margin, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Wrap},
};

use super::action::ConfirmAction;

pub struct Confirm {
    pub message: String,
    pub selected: bool,
    pub action: Option<ConfirmAction>,
}

impl Confirm {
    pub fn new() -> Self {
        Self {
            message: "".to_string(),
            selected: false,
            action: None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let confirm_block = Block::bordered()
            .fg(Color::Rgb(252, 252, 252))
            .border_type(BorderType::Rounded);

        frame.render_widget(confirm_block, area);
        let msg_area = area.inner(Margin {
            vertical: 2,
            horizontal: 2,
        });

        let msg = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(msg, msg_area);

        let btn_area = Rect {
            x: msg_area.x,
            y: msg_area.y + msg_area.height.saturating_sub(2),
            width: msg_area.width,
            height: 2,
        };

        let buttons = if self.selected {
            Line::from(vec![
                Span::styled("[ Yes ]", Style::default().fg(Color::Rgb(155, 201, 166))),
                Span::raw("   "),
                Span::styled("Cancel", Style::default().fg(Color::Rgb(252, 252, 252))),
            ])
        } else {
            Line::from(vec![
                Span::styled("Yes", Style::default().fg(Color::Rgb(252, 252, 252))),
                Span::raw("   "),
                Span::styled("[ Cancel ]", Style::default().fg(Color::Rgb(201, 155, 155))),
            ])
        };

        let btns_widget = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(btns_widget, btn_area);
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

    // Chaining API
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn action(mut self, action: ConfirmAction) -> Self {
        self.action = Some(action);
        self
    }
}
