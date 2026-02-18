use crate::{
    enums::FocusArea,
    models::Filter,
    state::{AdaptiveScroll, ApplicationResult, ApplicationState},
    theme::Theme,
    traits::{Input, InteractableEnum, Modal, ModalAction},
    ui::{Confirm, Form, Notification, Popup, TextInput},
};
use uuid::Uuid;

/// Active modal widget with modal itself and its action like save etc.
pub struct ActiveModal {
    pub modal: Box<dyn Modal>,
    pub action: ModalAction,
}

/// Main application UI state (only for rendering purposes)
#[derive(Default)]
pub struct UIState {
    pub current_filter: Filter,
    pub focus_area: FocusArea,

    pub modal: Option<ActiveModal>,
    pub task_form: Option<Form>,
    pub search_input: Option<TextInput>,

    pub desc_scroll: AdaptiveScroll,
    pub sidebar_scroll: AdaptiveScroll,

    pub theme: Theme,
}

impl UIState {
    /// Shows modal widget (confirm/popup)
    pub fn show_modal<M: Modal + 'static>(&mut self, modal: M, action: ModalAction) {
        self.modal = Some(ActiveModal {
            modal: Box::new(modal),
            action,
        });
    }

    /// Push notification to the state on error/success operation
    pub fn push_notification(
        &self,
        state: &mut ApplicationState,
        result: ApplicationResult<String>,
    ) {
        match result {
            Ok(msg) => state.notification = Some(Notification::success(msg)),
            Err(e) => state.notification = Some(Notification::error(e.to_string())),
        }
    }

    /// Show corresponding popup depending on ApplicationResult
    pub fn show_result_popup(&mut self, result: ApplicationResult<String>) {
        match result {
            Ok(msg) => self.show_modal(Popup::success(msg).close_on_any_key(), ModalAction::None),
            Err(e) => self.show_modal(
                Popup::error(e.to_string()).close_on_any_key(),
                ModalAction::None,
            ),
        }
    }

    /// Expire notification function (close after duration)
    pub fn expire_notification(&self, notification: &mut Option<Notification>) {
        if let Some(n) = notification
            && n.is_expired()
        {
            *notification = None;
        }
    }

    /// Toggles search input field
    pub fn show_search(&mut self) {
        self.search_input = Some(TextInput::new().title(" Search "));
    }

    /// Returns search query string
    pub fn search_query(&self) -> String {
        self.search_input
            .as_ref()
            .map(|i| i.buffer.to_lowercase())
            .unwrap_or_default()
    }

    /// Return id of a todo based on TableState selection
    pub fn selected_id(&self, state: &ApplicationState) -> Option<Uuid> {
        let index: usize = state.select_state.selected()?;
        state
            .filter(&self.current_filter)
            .nth(index)
            .map(|(_, task)| task.id)
    }

    /// Switch application theme
    pub fn switch_theme(&mut self) {
        self.theme = self.theme.next()
    }

    /// Next filter tab (sidebar)
    pub fn next_tab_filter(&mut self) {
        self.current_filter = self.current_filter.next();
    }

    /// Previous filter tab (sidebar)
    pub fn prev_tab_filter(&mut self) {
        self.current_filter = self.current_filter.prev();
    }

    /// Changes to specific filter
    pub fn change_filter(&mut self, filter: Filter) {
        self.current_filter = filter;
    }

    /// Toggle main menu focus (filters/tasks + form)
    pub fn toggle_focus(&mut self) {
        self.focus_area = match self.focus_area {
            FocusArea::LeftPanel => FocusArea::MainContent,
            FocusArea::MainContent => FocusArea::LeftPanel,
        };
        self.sidebar_scroll.reset();
    }

    /// Closes existing modal widget
    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    /// Opens remove confirm widget
    pub fn remove_confirm(&mut self) {
        self.show_modal(
            Confirm::new("Are you sure to remove selected task?"),
            ModalAction::Remove,
        );
    }

    /// Opens clear confirm widget
    pub fn clear_confirm(&mut self) {
        self.show_modal(
            Confirm::new("Are you sure to clear all tasks?"),
            ModalAction::Clear,
        );
    }

    /// Opens save confirm widget
    pub fn save_confirm(&mut self) {
        self.show_modal(
            Confirm::new("Do you want to save tasks?"),
            ModalAction::Save,
        );
    }

    /// Opens confirm widget on unsaved exit
    pub fn unsaved_confirm(&mut self) {
        self.show_modal(
            Confirm::new("You have unsaved changes. Save before exit?"),
            ModalAction::UnsavedExit,
        );
    }
}

// Unit-tests for UIState
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{StorageError, TodoError},
        models::{Priority, Todo},
        ui::Popup,
    };
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
    fn should_show_search_input() {
        let mut ui = UIState::default();
        ui.show_search();

        assert!(ui.search_input.is_some());
        assert_eq!(ui.search_input.as_ref().unwrap().buffer, "");

        ui.search_input = None;
        assert!(ui.search_input.is_none());
    }

    #[test]
    fn should_handle_save_result_with_popup() {
        let mut ui = UIState::default();
        ui.show_result_popup(Ok("Saved!".to_string()));

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.as_ref().unwrap().action, ModalAction::None);

        ui.close_modal();

        ui.show_result_popup(Err(StorageError::JSONError.into()));
        assert!(ui.modal.is_some());
    }

    #[test]
    fn should_handle_notification_pushing() {
        let ui = UIState::default();
        let mut state = ApplicationState::default();

        ui.push_notification(&mut state, Ok("Success message".to_string()));
        assert!(state.notification.is_some());

        let error_res: ApplicationResult<String> = Err(TodoError::EmptyTitle.into());
        ui.push_notification(&mut state, error_res);
        assert!(state.notification.is_some());
    }

    #[test]
    fn should_return_id_of_selected_task() {
        let ui = UIState::default();
        let mut state = ApplicationState::default();

        let todos: Vec<Todo> = vec![
            Todo::new("Task 1", "", None),
            Todo::new("Task 2", "", Some(Priority::Medium)),
        ];
        let last_id: Uuid = todos[1].id;
        state.todos = todos;
        state.select_state.select(Some(1));

        let expected_id: Uuid = ui.selected_id(&state).unwrap();
        assert_eq!(last_id, expected_id);
    }

    #[test]
    fn should_test_notification_expiration() {
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

    #[test]
    fn should_open_remove_confirm() {
        let mut ui = UIState::default();
        ui.remove_confirm();

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_open_clear_confirm() {
        let mut ui = UIState::default();
        ui.clear_confirm();

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.unwrap().action, ModalAction::Clear);
    }

    #[test]
    fn should_open_save_confirm() {
        let mut ui = UIState::default();
        ui.save_confirm();

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.unwrap().action, ModalAction::Save);
    }

    #[test]
    fn should_open_unsaved_confirm_exit() {
        let mut ui = UIState::default();
        ui.unsaved_confirm();

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.unwrap().action, ModalAction::UnsavedExit);
    }
}
