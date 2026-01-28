use crate::{
    enums::FocusArea,
    models::Filter,
    state::ApplicationResult,
    theme::{Theme, ThemeColors},
    traits::{InteractableEnum, Modal, ModalAction},
    ui::{Form, Notification, Popup},
};
use ratatui::style::Style;

pub struct ActiveModal {
    pub modal: Box<dyn Modal>,
    pub action: ModalAction,
}

#[derive(Default)]
pub struct UIState<'a> {
    pub current_filter: Filter,
    pub focus_area: FocusArea,

    pub modal: Option<ActiveModal>,
    pub task_form: Option<Form<'a>>,

    pub theme: Theme,
}

impl<'a> UIState<'a> {
    // Next tab filter
    pub fn next_tab_filter(&mut self) {
        self.current_filter = self.current_filter.next();
    }

    // Prev tab filter
    pub fn prev_tab_filter(&mut self) {
        self.current_filter = self.current_filter.prev();
    }

    // Change to specific filter
    pub fn change_filter(&mut self, filter: Filter) {
        self.current_filter = filter;
    }

    // Toggle main menu focus (filters/tasks + form)
    pub fn toggle_focus(&mut self) {
        self.focus_area = match self.focus_area {
            FocusArea::LeftPanel => FocusArea::MainContent,
            FocusArea::MainContent => FocusArea::LeftPanel,
        };
    }

    // Return styles if focused
    pub fn styles_on_focus(&self) -> Style {
        let colors: ThemeColors = self.theme.data();
        if self.focus_area == FocusArea::MainContent {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.border)
        }
    }

    // Switch theme
    pub fn switch_theme(&mut self) {
        self.theme = self.theme.next()
    }

    // Modal
    pub fn show_modal<M: Modal + 'static>(&mut self, modal: M, action: ModalAction) {
        self.modal = Some(ActiveModal {
            modal: Box::new(modal),
            action,
        });
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    // Handle save result with popup
    pub fn handle_save_with_popup(&mut self, result: ApplicationResult<String>) {
        match result {
            Ok(msg) => {
                let popup: Popup = Popup::success(msg).close_on_any_key();
                self.show_modal(popup, ModalAction::None);
            }
            Err(e) => {
                let popup: Popup = Popup::error(e.to_string()).close_on_any_key();
                self.show_modal(popup, ModalAction::None);
            }
        }
    }

    // Expire notification (close after duration)
    pub fn expire_notification(&self, notification: &mut Option<Notification>) {
        if let Some(n) = notification
            && n.is_expired()
        {
            *notification = None;
        }
    }
}

// Unit-tests for UIState
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state::StorageError, ui::Popup};
    use std::time::{Duration, Instant};

    #[test]
    fn should_navigate_through_filters() {
        let mut ui = UIState::default();
        ui.current_filter = Filter::All;

        ui.next_tab_filter();
        assert_eq!(ui.current_filter, Filter::Active);

        ui.next_tab_filter();
        assert_eq!(ui.current_filter, Filter::Completed);

        ui.prev_tab_filter();
        assert_eq!(ui.current_filter, Filter::Active);

        ui.change_filter(Filter::HighPriority);
        assert_eq!(ui.current_filter, Filter::HighPriority);
    }

    #[test]
    fn should_toggle_focus_properly() {
        let mut ui = UIState::default();
        ui.focus_area = FocusArea::LeftPanel;

        ui.toggle_focus();
        assert_eq!(ui.focus_area, FocusArea::MainContent);

        ui.toggle_focus();
        assert_eq!(ui.focus_area, FocusArea::LeftPanel);
    }

    #[test]
    fn should_show_close_dialog_with_ui_state() {
        let mut ui = UIState::default();

        ui.show_modal(Popup::info("Test"), ModalAction::Remove);

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.as_ref().unwrap().action, ModalAction::Remove);

        ui.close_modal();
        assert!(ui.modal.is_none());
    }

    #[test]
    fn should_handle_save_result_with_popup() {
        let mut ui = UIState::default();

        ui.handle_save_with_popup(Ok("Saved!".to_string()));
        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.as_ref().unwrap().action, ModalAction::None);

        ui.close_modal();

        ui.handle_save_with_popup(Err(StorageError::JSONError.into()));
        assert!(ui.modal.is_some());
    }

    #[test]
    fn test_notification_expiration() {
        let ui = UIState::default();

        let mut expired_notification = Some(Notification {
            created_at: Instant::now() - Duration::from_secs(10),
            ..Notification::success("Test")
        });

        ui.expire_notification(&mut expired_notification);
        assert!(
            expired_notification.is_none(),
            "Expired notification must be removed from UIState"
        );

        let mut fresh_notification = Some(Notification::success("Hello"));
        ui.expire_notification(&mut fresh_notification);
        assert!(
            fresh_notification.is_some(),
            "Fresh notification must remain active"
        );
    }
}
