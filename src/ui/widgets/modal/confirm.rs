use crate::{
    traits::{Modal, ModalResult},
    ui::center,
    utils::constants::{
        size::{CONFIRM_HEIGHT, CONFIRM_WIDTH},
        theme::{TEXT_DIMMED, TEXT_PRIMARY},
    },
};
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, style::Modifier, text::Span};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmOption {
    Yes,
    Cancel,
}

pub struct Confirm {
    pub message: String,
    pub select: ConfirmOption,
}

impl Confirm {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            select: ConfirmOption::Cancel,
        }
    }
}

impl Modal for Confirm {
    // Calculate area for confirm
    fn area(&self, frame_area: Rect) -> Rect {
        center(CONFIRM_WIDTH, CONFIRM_HEIGHT, frame_area)
    }

    // Rendering
    fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Alignment, Constraint, Direction, Layout},
            style::{Color, Style, Stylize},
            text::Line,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let confirm_block = Block::bordered()
            .fg(TEXT_PRIMARY)
            .border_style(Style::default())
            .title_top(Line::from(" Confirm Operation ").centered())
            .border_type(BorderType::Rounded);

        let inner_area = confirm_block.inner(area);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Message
                Constraint::Length(1), // Buttons
            ])
            .margin(1)
            .split(inner_area);

        frame.render_widget(confirm_block, area);

        let message = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message, chunks[0]);

        let (yes_style, cancel_style): (Style, Style) = match self.select {
            ConfirmOption::Yes => (
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(TEXT_DIMMED),
            ),
            ConfirmOption::Cancel => (
                Style::default().fg(TEXT_DIMMED),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        };

        let buttons: Line = Line::from(vec![
            Span::styled("[ ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("Yes", yes_style),
            Span::styled(" ]", Style::default().fg(TEXT_PRIMARY)),
            Span::raw("   "),
            Span::styled("[ ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("Cancel", cancel_style),
            Span::styled(" ]", Style::default().fg(TEXT_PRIMARY)),
        ]);

        let buttons_widget = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, chunks[1]);
    }

    // Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<ModalResult> {
        match key {
            KeyCode::Char('y') => return Some(ModalResult::Confirmed),
            KeyCode::Char('n') | KeyCode::Esc => return Some(ModalResult::Cancelled),
            KeyCode::Enter => {
                return Some(match self.select {
                    ConfirmOption::Yes => ModalResult::Confirmed,
                    ConfirmOption::Cancel => ModalResult::Cancelled,
                });
            }
            KeyCode::Right | KeyCode::Char('l') => self.select = ConfirmOption::Cancel,
            KeyCode::Left | KeyCode::Char('h') => self.select = ConfirmOption::Yes,
            _ => {}
        }

        None
    }
}

// Unit-tests for confirm widget
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_default_confirm() {
        let confirm: Confirm = Confirm::new("Test");

        assert_eq!(confirm.message, "Test");
        assert_eq!(confirm.select, ConfirmOption::Cancel);
    }

    #[test]
    fn should_create_area_for_confirm() {
        let frame: Rect = create_helper_frame();
        let confirm: Confirm = Confirm::new("Test");

        let area: Rect = confirm.area(frame);

        let expected_x = (100 - CONFIRM_WIDTH) / 2;
        let expected_y = (100 - CONFIRM_HEIGHT) / 2;

        assert_eq!(area.x, expected_x as u16);
        assert_eq!(area.y, expected_y as u16);
    }

    #[test]
    fn should_handle_left_right_key_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        assert_eq!(confirm.select, ConfirmOption::Cancel);

        confirm.handle_key(KeyCode::Left);
        assert_eq!(confirm.select, ConfirmOption::Yes);

        confirm.handle_key(KeyCode::Right);
        assert_eq!(confirm.select, ConfirmOption::Cancel);

        confirm.handle_key(KeyCode::Char('h'));
        assert_eq!(confirm.select, ConfirmOption::Yes);

        confirm.handle_key(KeyCode::Char('l'));
        assert_eq!(confirm.select, ConfirmOption::Cancel);
    }

    #[test]
    fn should_handle_key_enter_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        confirm.select = ConfirmOption::Yes;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(ModalResult::Confirmed)
        );

        confirm.select = ConfirmOption::Cancel;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_esc_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        assert_eq!(
            confirm.handle_key(KeyCode::Esc),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_other_keys_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        assert_eq!(confirm.handle_key(KeyCode::Char('a')), None);
        assert_eq!(confirm.handle_key(KeyCode::Down), None);
        assert_eq!(confirm.select, ConfirmOption::Cancel);

        assert_eq!(
            confirm.handle_key(KeyCode::Char('y')),
            Some(ModalResult::Confirmed)
        );
        assert_eq!(
            confirm.handle_key(KeyCode::Char('n')),
            Some(ModalResult::Cancelled)
        );
    }
}
