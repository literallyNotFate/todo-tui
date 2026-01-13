use crate::{
    state::Anchor,
    ui::{Dialog, DialogResult},
    utils::{constants::theme::TEXT_PRIMARY, widgets::confirm::render_confirm_buttons},
};
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, style::Color, widgets::Padding};

pub struct ConfirmStyles {
    pub border_color: Color,
    pub padding: Padding,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmOption {
    Yes,
    Cancel,
}

pub struct Confirm {
    pub message: String,
    pub select: ConfirmOption,
    pub anchor: Anchor,
    pub styles: ConfirmStyles,
}

impl Dialog for Confirm {
    fn new() -> Self {
        Self {
            message: "".to_string(),
            select: ConfirmOption::Cancel,
            anchor: Anchor::Center,
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
        use crate::utils::{
            anchored, calculate_content_size, constants::size::CONFIRM_PERCENTAGE_WIDTH,
        };

        let (top_len, bottom_len): (usize, usize) = self.titles_len();
        let (width, height): (u16, u16) = calculate_content_size(
            frame_area,
            &self.message,
            top_len,
            bottom_len,
            self.styles.padding,
            CONFIRM_PERCENTAGE_WIDTH,
        );

        anchored(frame_area, width, height, self.anchor.clone())
    }

    // Calculate titles length
    fn titles_len(&self) -> (usize, usize) {
        (0, render_confirm_buttons(ConfirmOption::Yes).width())
    }

    // Rendering
    fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Alignment, Margin},
            style::{Style, Stylize},
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
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
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

// Unit-tests for confirm widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::theme::{
        COLOR_RED, CONFIRM_CANCEL_FG_ACTIVE, CONFIRM_YES_FG_ACTIVE, TEXT_DIMMED,
    };

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn should_create_default_confirm() {
        let confirm: Confirm = Confirm::new();

        assert_eq!(confirm.message, "");
        assert_eq!(confirm.select, ConfirmOption::Cancel);
        assert_eq!(confirm.anchor, Anchor::Center);
        assert_eq!(confirm.styles.border_color, TEXT_PRIMARY);
        assert_eq!(confirm.styles.padding, Padding::new(2, 2, 3, 3));
    }

    #[test]
    fn should_create_confirm_with_chaining_api() {
        let confirm: Confirm = Confirm::new()
            .with_message("Delete all tasks?")
            .anchor(Anchor::BottomLeft)
            .with_border_color(COLOR_RED)
            .with_padding(Padding::uniform(4));

        assert_eq!(confirm.message, "Delete all tasks?");
        assert_eq!(confirm.anchor, Anchor::BottomLeft);
        assert_eq!(confirm.styles.border_color, COLOR_RED);
        assert_eq!(confirm.styles.padding, Padding::uniform(4));
        assert_eq!(confirm.select, ConfirmOption::Cancel);
    }

    #[test]
    fn should_create_area_for_confirm() {
        let frame: Rect = create_helper_frame();
        let confirm: Confirm =
            Confirm::new().with_message("Are you sure? This action cannot be undone.");

        let area: Rect = confirm.area(frame);

        assert!(area.x > 10 && area.x < 60);
        assert!(area.y > 10 && area.y < 30);
        assert!(area.width > 30 && area.width < 80);
        assert!(area.height > 6 && area.height < 20);
    }

    #[test]
    fn should_calculate_confirm_titles_length() {
        let confirm: Confirm = Confirm::new();
        let (top_len, bottom_len): (usize, usize) = confirm.titles_len();

        assert_eq!(top_len, 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_handle_left_right_key_confirm() {
        let mut confirm: Confirm = Confirm::new();

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
        let mut confirm: Confirm = Confirm::new();

        confirm.select = ConfirmOption::Yes;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(DialogResult::Confirmed)
        );

        confirm.select = ConfirmOption::Cancel;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_esc_confirm() {
        let mut confirm: Confirm = Confirm::new();

        assert_eq!(
            confirm.handle_key(KeyCode::Esc),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_other_keys_confirm() {
        let mut confirm: Confirm = Confirm::new();

        assert_eq!(confirm.handle_key(KeyCode::Char('a')), None);
        assert_eq!(confirm.handle_key(KeyCode::Down), None);
        assert_eq!(confirm.select, ConfirmOption::Cancel);

        assert_eq!(
            confirm.handle_key(KeyCode::Char('y')),
            Some(DialogResult::Confirmed)
        );
        assert_eq!(
            confirm.handle_key(KeyCode::Char('n')),
            Some(DialogResult::Cancelled)
        );
    }

    // Utils
    #[test]
    fn should_render_confirm_buttons_yes_selected() {
        let line = render_confirm_buttons(ConfirmOption::Yes);

        let spans: Vec<&str> = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();

        assert_eq!(spans, vec!["[ ", "Yes", " ]", "   ", "[ ", "Cancel", " ]"]);

        let colors: Vec<Color> = line
            .spans
            .iter()
            .map(|span| span.style.fg.unwrap_or(Color::Reset))
            .collect();

        assert_eq!(colors[1], CONFIRM_YES_FG_ACTIVE);
        assert_eq!(colors[5], TEXT_DIMMED);
    }

    #[test]
    fn should_render_confirm_buttons_cancel_selected() {
        let line = render_confirm_buttons(ConfirmOption::Cancel);

        let colors: Vec<Color> = line
            .spans
            .iter()
            .map(|span| span.style.fg.unwrap_or(Color::Reset))
            .collect();

        assert_eq!(colors[1], TEXT_DIMMED);
        assert_eq!(colors[5], CONFIRM_CANCEL_FG_ACTIVE);
    }
}
