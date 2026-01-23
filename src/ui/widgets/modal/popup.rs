use crate::{
    traits::{Modal, ModalResult},
    ui::center,
    utils::constants::{
        size::{POPUP_HEIGHT, POPUP_WIDTH},
        theme::TEXT_PRIMARY,
    },
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
};

#[derive(Debug, Clone, PartialEq)]
pub enum PopupCloseBehavior {
    AnyKey,
    Specific(KeyCode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupKind {
    Info,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub message: String,
    pub title: String,
    pub kind: PopupKind,
    pub close_behavior: PopupCloseBehavior,
    pub border_style: Style,
}

impl Popup {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Info,
            message: message.into(),
            title: String::default(),
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
            border_style: Style::default(),
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Success,
            ..Self::info(message)
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Error,
            ..Self::info(message)
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn close_on_any_key(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::AnyKey;
        self
    }

    pub fn close_on(mut self, key: KeyCode) -> Self {
        self.close_behavior = PopupCloseBehavior::Specific(key);
        self
    }
}

impl Modal for Popup {
    // Calculate area for popup
    fn area(&self, frame_area: Rect) -> Rect {
        center(POPUP_WIDTH, POPUP_HEIGHT, frame_area)
    }

    // Rendering
    fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::Alignment,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let key: String = match self.close_behavior {
            PopupCloseBehavior::AnyKey => "any key".to_string(),
            PopupCloseBehavior::Specific(c) => format!("<{}>", c),
        };

        let bottom_line: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                key,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ]);

        let border_style: Style = match self.kind {
            PopupKind::Info => Style::default().fg(Color::Blue),
            PopupKind::Success => Style::default().fg(Color::Green),
            PopupKind::Error => Style::default().fg(Color::Red),
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(self.title.as_str())
            .title_bottom(bottom_line)
            .border_style(border_style)
            .fg(TEXT_PRIMARY);

        let paragraph = Paragraph::new(self.message.as_str())
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    // Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<ModalResult> {
        match self.close_behavior {
            PopupCloseBehavior::AnyKey => Some(ModalResult::Cancelled),
            PopupCloseBehavior::Specific(k) if k == key => Some(ModalResult::Cancelled),
            _ => None,
        }
    }
}

// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_default_popup() {
        let mut popup: Popup = Popup::success("Success");

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Success");
        assert_eq!(popup.title, "");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );

        popup = Popup::info("Info");
        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.message, "Info");

        popup = Popup::error("Error");
        assert_eq!(popup.kind, PopupKind::Error);
        assert_eq!(popup.message, "Error");
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup: Popup = Popup::success("Task completed!")
            .title("Some title")
            .close_on_any_key();

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Task completed!");
        assert_eq!(popup.title, "Some title");
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
    }

    #[test]
    fn should_create_area_for_popup() {
        let frame: Rect = create_helper_frame();
        let popup: Popup = Popup::info("Test");
        let area: Rect = popup.area(frame);

        let expected_x = (100 - POPUP_WIDTH) / 2;
        let expected_y = (100 - POPUP_HEIGHT) / 2;

        assert_eq!(area.x, expected_x as u16);
        assert_eq!(area.y, expected_y as u16);
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup: Popup = Popup::info("Test").close_on_any_key();

        assert_eq!(
            popup.handle_key(KeyCode::Char('q')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Enter),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Esc), Some(ModalResult::Cancelled));
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup: Popup = Popup::error("Test").close_on(KeyCode::Char('y'));

        assert_eq!(
            popup.handle_key(KeyCode::Char('y')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }
}
