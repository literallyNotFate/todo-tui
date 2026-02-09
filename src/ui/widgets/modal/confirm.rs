use crate::{
    theme::ThemeColors,
    traits::{Modal, ModalResult},
    ui::center,
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
};

pub const CONFIRM_WIDTH: u16 = 30;
pub const CONFIRM_HEIGHT: u16 = 25;

/// Confirm selection options
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmOption {
    Yes,
    Cancel,
}

/// Popup modal widget
pub struct Confirm {
    pub message: String,
    pub select: ConfirmOption,
}

impl Confirm {
    /// New confirm widget
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            select: ConfirmOption::Cancel,
        }
    }

    /// Vertical layout for inner content
    fn vertical_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .split(area)
    }

    /// Horizontal layout for inner content
    fn horizontal_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10), // Left
                Constraint::Percentage(80),
                Constraint::Percentage(10), // Right
            ])
            .split(area)
    }

    /// Style for buttons based on selection
    fn button_styles(&self, theme: &ThemeColors) -> (Style, Style) {
        match self.select {
            ConfirmOption::Yes => (
                Style::default().fg(theme.success).bold(),
                Style::default().fg(theme.text_dim),
            ),
            ConfirmOption::Cancel => (
                Style::default().fg(theme.text_dim),
                Style::default().fg(theme.error).bold(),
            ),
        }
    }

    /// Render buttons
    fn button_line(&self, styles: (Style, Style), theme: &ThemeColors) -> Line<'static> {
        Line::from(vec![
            Span::styled("[ ", Style::default().fg(theme.text_primary)),
            Span::styled("Yes", styles.0),
            Span::styled(" ]", Style::default().fg(theme.text_primary)),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(theme.text_primary)),
            Span::styled("Cancel", styles.1),
            Span::styled(" ]", Style::default().fg(theme.text_primary)),
        ])
    }
}

impl Modal for Confirm {
    /// Calculate area for confirm
    fn area(&self, frame_area: Rect) -> Rect {
        center(frame_area, CONFIRM_WIDTH, CONFIRM_HEIGHT)
    }

    /// Confirm rendering
    fn render(&self, frame: &mut Frame, area: Rect, theme: &ThemeColors) {
        use ratatui::{
            layout::Alignment,
            style::Stylize,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let confirm_block: Block = Block::bordered()
            .fg(theme.text_primary)
            .border_style(Style::default().fg(theme.accent))
            .title_top(Line::from(" Confirm Action ").centered())
            .border_type(BorderType::Rounded);

        let inner_area: Rect = confirm_block.inner(area);
        frame.render_widget(confirm_block.clone(), area);

        let vertical_chunks: std::rc::Rc<[Rect]> = self.vertical_layout(inner_area);

        let message_area: Rect = self.horizontal_layout(vertical_chunks[1])[1];
        let buttons_area: Rect = self.horizontal_layout(vertical_chunks[3])[1];

        let message: Paragraph = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(message, message_area);

        let button_styles: (Style, Style) = self.button_styles(theme);
        let buttons: Line = self.button_line(button_styles, theme);

        let buttons_widget = Paragraph::new(buttons).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, buttons_area);
    }

    /// Key event handling
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

/// Unit-tests for confirm widget
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

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

        assert_eq!(area.x, expected_x);
        assert_eq!(area.y, expected_y);
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

    #[test]
    fn should_return_proper_styles_for_buttons_with_theme() {
        let mut confirm: Confirm = Confirm::new("Test");
        let mut styles: (Style, Style) = confirm.button_styles(&ThemeColors::GRUVBOX);

        assert_eq!(styles.0, Style::default().fg(Color::Rgb(168, 153, 132)));
        assert_eq!(
            styles.1,
            Style::default().fg(Color::Rgb(251, 73, 52)).bold()
        );

        confirm.select = ConfirmOption::Yes;
        styles = confirm.button_styles(&ThemeColors::GRUVBOX);

        assert_eq!(
            styles.0,
            Style::default().fg(Color::Rgb(184, 187, 38)).bold()
        );
        assert_eq!(styles.1, Style::default().fg(Color::Rgb(168, 153, 132)));
    }

    #[test]
    fn should_generate_buttons_with_theme() {
        let mut confirm: Confirm = Confirm::new("Test");
        let mut styles: (Style, Style) = confirm.button_styles(&ThemeColors::GRUVBOX);
        let mut buttons: Line = confirm.button_line(styles, &ThemeColors::GRUVBOX);

        let mut expected_line: Line = Line::from(vec![
            Span::styled("[ ", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::styled("Yes", Style::default().fg(Color::Rgb(168, 153, 132))),
            Span::styled(" ]", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::styled(
                "Cancel",
                Style::default().fg(Color::Rgb(251, 73, 52)).bold(),
            ),
            Span::styled(" ]", Style::default().fg(Color::Rgb(235, 219, 178))),
        ]);

        assert_eq!(buttons, expected_line);

        confirm.select = ConfirmOption::Yes;
        styles = confirm.button_styles(&ThemeColors::GRUVBOX);
        buttons = confirm.button_line(styles, &ThemeColors::GRUVBOX);

        expected_line = Line::from(vec![
            Span::styled("[ ", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::styled("Yes", Style::default().fg(Color::Rgb(184, 187, 38)).bold()),
            Span::styled(" ]", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::styled("Cancel", Style::default().fg(Color::Rgb(168, 153, 132))),
            Span::styled(" ]", Style::default().fg(Color::Rgb(235, 219, 178))),
        ]);

        assert_eq!(buttons, expected_line);
    }
}
