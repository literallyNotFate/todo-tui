use crate::ui::{Popup, PopupComponent, RenderContext, widgets::modal::popup::PopupKind};
use ratatui::layout::Rect;

/// Component to render message popup (success/error/info)
pub struct MessageComponent(pub String);

impl PopupComponent for MessageComponent {
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            widgets::{Paragraph, Wrap},
        };

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
            .split(area);

        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ])
            .split(vertical_chunks[1]);

        let message_area = horizontal_chunks[1];
        let message = Paragraph::new(self.0.as_str())
            .centered()
            .wrap(Wrap { trim: true });

        ctx.render_widget(message, message_area);
    }
}

impl Popup {
    pub fn success(msg: impl Into<String>) -> Self {
        Self::new(
            " Success ",
            Box::new(MessageComponent(msg.into())),
            PopupKind::Success,
        )
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(
            " Error ",
            Box::new(MessageComponent(msg.into())),
            PopupKind::Error,
        )
    }
    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(
            " Info ",
            Box::new(MessageComponent(msg.into())),
            PopupKind::Info,
        )
    }
}

/// Unit-tests for message components
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{
        WidgetResponse,
        widgets::modal::{ModalResult, popup::PopupCloseBehavior},
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn should_create_success_popup_with_message_component() {
        let popup = Popup::success("Operation completed successfully!");

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.title, " Success ");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );
        assert_eq!(popup.content.to_modal_result(), ModalResult::Cancelled);
    }

    #[test]
    fn should_create_error_popup_with_message_component() {
        let popup = Popup::error("Something went wrong!");

        assert_eq!(popup.kind, PopupKind::Error);
        assert_eq!(popup.title, " Error ");
        assert_eq!(popup.content.to_modal_result(), ModalResult::Cancelled);
    }

    #[test]
    fn should_create_info_popup_with_message_component() {
        let popup = Popup::info("Please note this information.");

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Info ");
        assert_eq!(popup.content.to_modal_result(), ModalResult::Cancelled);
    }

    #[test]
    fn should_handle_key_in_message_component_as_continue() {
        let mut component = MessageComponent("Test message".to_string());
        let dummy_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let response = component.handle_key(&dummy_key);

        assert!(matches!(response, WidgetResponse::Continue));
    }
}
