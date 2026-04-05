use crate::{
    config::UIConfig,
    core::{FocusArea, Selectable},
    models::Filter,
    state::{ActiveModal, AdaptiveScroll, ApplicationResult, ApplicationState, Session},
    theme::{Theme, ThemeName},
    ui::{
        Confirm, Form, Notification, Popup, TextInput,
        widgets::{
            input::Input,
            modal::{Modal, ModalAction, ModalSize},
        },
    },
};
use uuid::Uuid;

/// Main application UI state (only for rendering purposes)
#[derive(Default)]
pub struct UIState {
    pub filter: Selectable<Filter>,
    pub focused: Selectable<FocusArea>,

    pub modal: Option<ActiveModal>,
    pub task_form: Option<Form>,
    pub search_input: Option<TextInput>,

    pub desc_scroll: AdaptiveScroll,
    pub hotkeys_scroll: AdaptiveScroll,

    pub theme: Theme,
    pub config: UIConfig,

    should_redraw: bool,
}

impl UIState {
    pub fn new(mut config: UIConfig) -> Self {
        config.last_dark.get_or_insert(ThemeName::GruvboxDark);
        config.last_light.get_or_insert(ThemeName::GruvboxLight);

        let mut ui = Self {
            theme: Theme::new(config.theme),
            config,
            ..Self::default()
        };

        ui.refresh_theme();
        ui
    }

    /// Apply UI state from loaded session
    pub fn apply_session(&mut self, session: Session) {
        self.filter = session.last_filter;
        self.focused = session.last_focus;
        self.desc_scroll = AdaptiveScroll::with_position(session.description_scroll_pos);
        self.hotkeys_scroll = AdaptiveScroll::with_position(session.hotkeys_scroll_pos);
        self.config.use_system_theme = session.use_system_theme;

        self.search_input =
            (!session.last_query.is_empty()).then(|| TextInput::from(session.last_query));
        self.refresh_theme();
    }

    /// Request UI to be redrawed in the application draw method
    pub fn request_redraw(&mut self) {
        self.should_redraw = true;
    }

    /// Check whether terminal should redraw the frame
    pub fn needs_redraw(&self) -> bool {
        self.should_redraw
    }

    /// Tells terminal to stop redrawing UI
    pub fn clear_redraw_flag(&mut self) {
        self.should_redraw = false;
    }

    /// Shows modal widget (confirm/popup)
    pub fn show_modal<M: Modal + 'static>(&mut self, modal: M, action: ModalAction) {
        log::info!("Opening modal: action={:?}", action);
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
            Ok(msg) => {
                log::debug!("Result popup (Success): {}", msg);
                self.show_modal(
                    Popup::success(msg)
                        .with_size(ModalSize::Small)
                        .close_on_any_key(),
                    ModalAction::None,
                )
            }
            Err(e) => {
                log::error!("Result popup (Error): {}", e);
                self.show_modal(
                    Popup::error(e.to_string())
                        .with_size(ModalSize::Small)
                        .close_on_any_key(),
                    ModalAction::None,
                )
            }
        }
    }

    /// Expire notification function (close after duration)
    pub fn expire_notification(&self, notification: &mut Option<Notification>) -> bool {
        if let Some(n) = notification {
            if n.is_expired() {
                *notification = None;
                return true;
            }

            if n.tick() {
                return true;
            }
        }

        false
    }

    /// Toggles search input field
    pub fn show_search(&mut self) {
        self.search_input = Some(TextInput::new().title(" Search "));
    }

    /// Returns search query string
    pub fn search_query(&self) -> &str {
        self.search_input
            .as_ref()
            .map(|input| input.buffer.as_str())
            .unwrap_or("")
    }

    /// Return id of a todo based on TableState selection
    pub fn selected_id(&self, state: &ApplicationState) -> Option<Uuid> {
        let index: usize = state.select_state.selected()?;
        state
            .filter(&self.filter)
            .nth(index)
            .map(|(_, task)| task.id)
    }

    /// Toggle dark/light theme mode
    pub fn toggle_mode(&mut self) {
        self.config.use_system_theme = false;

        let current_name: ThemeName = self.theme.name();
        if self.theme.is_dark() {
            self.config.last_dark = Some(current_name);
        } else {
            self.config.last_light = Some(current_name);
        }

        let target = if self.theme.is_dark() {
            self.config.last_light.unwrap_or(ThemeName::GruvboxLight)
        } else {
            self.config.last_dark.unwrap_or(ThemeName::GruvboxDark)
        };

        log::info!("Manual theme mode toggle. New theme: {}", target);
        self.theme.name.set(target);
        self.config.theme = target;
    }

    /// Next theme wrapper
    pub fn next_theme(&mut self) {
        self.cycle_theme(true);
    }

    /// Previous theme wrapper
    pub fn prev_theme(&mut self) {
        self.cycle_theme(false);
    }

    /// Helper function to refresh theme
    pub fn refresh_theme(&mut self) {
        if self.config.use_system_theme {
            self.apply_system_theme();
        } else {
            self.theme = Theme::new(self.config.theme);
        }
    }

    /// Base method to cycle through themes in both directions
    fn cycle_theme(&mut self, forward: bool) {
        self.config.use_system_theme = false;

        if forward {
            self.theme.next();
        } else {
            self.theme.prev();
        }

        let new_name: ThemeName = self.theme.name();
        self.config.theme = new_name;
        log::info!("Theme changed to: {}", new_name);

        if self.theme.is_dark() {
            self.config.last_dark = Some(new_name);
        } else {
            self.config.last_light = Some(new_name);
        }
    }

    /// Applies theme based on system preferences
    pub fn apply_system_theme(&mut self) -> bool {
        let is_dark = dark_light::detect()
            .map(|m| m == dark_light::Mode::Dark)
            .unwrap_or(true);

        let target_theme = if is_dark {
            self.config.last_dark.unwrap_or(ThemeName::GruvboxDark)
        } else {
            self.config.last_light.unwrap_or(ThemeName::GruvboxLight)
        };

        if self.config.theme == target_theme {
            return false;
        }

        log::info!(
            "System theme change detected! Switching to {:?}",
            target_theme
        );

        self.theme = Theme::new(target_theme);
        self.config.theme = target_theme;

        true
    }

    /// Next filter tab (sidebar)
    pub fn next_tab_filter(&mut self) {
        self.filter.next();
        log::trace!("Filter changed to: {:?}", self.filter);
    }

    /// Previous filter tab (sidebar)
    pub fn prev_tab_filter(&mut self) {
        self.filter.prev();
        log::trace!("Filter changed to: {:?}", self.filter);
    }

    /// Changes to specific filter
    pub fn change_filter(&mut self, filter: Filter) {
        self.filter.set(filter);
        log::trace!("Filter changed to: {:?}", self.filter);
    }

    /// Toggle main menu focus (filters/tasks + form)
    pub fn toggle_focus(&mut self) {
        self.focused.next();
        log::trace!("Focus toggled to: {:?}", self.focused);
        self.hotkeys_scroll.reset();
    }

    /// Toggle sidebar
    pub fn toggle_sidebar(&mut self) {
        self.config.show_sidebar = !self.config.show_sidebar;
    }

    /// Closes existing modal widget
    pub fn close_modal(&mut self) {
        if let Some(m) = &self.modal {
            log::debug!("Closing modal: action={:?}", m.action);
        }

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

/// Unit-tests for UIState
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
        ui.filter.set(Filter::All);

        ui.next_tab_filter();
        assert_eq!(ui.filter, Filter::Active);

        ui.next_tab_filter();
        assert_eq!(ui.filter, Filter::Completed);

        ui.prev_tab_filter();
        assert_eq!(ui.filter, Filter::Active);

        ui.change_filter(Filter::HighPriority);
        assert_eq!(ui.filter, Filter::HighPriority);
    }

    #[test]
    fn should_toggle_focus_properly() {
        let mut ui = UIState::default();
        ui.focused.set(FocusArea::Sidebar);

        ui.toggle_focus();
        assert_eq!(ui.focused, FocusArea::Main);

        ui.toggle_focus();
        assert_eq!(ui.focused, FocusArea::Sidebar);
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

        ui.show_result_popup(Err(StorageError::JSON("Some error".to_string()).into()));
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
    fn should_handle_theme_mode_switching_with_memory() {
        let mut config = UIConfig::default();
        config.last_dark = Some(ThemeName::KanagawaWave);
        config.last_light = Some(ThemeName::KanagawaLotus);

        let mut ui = UIState::new(config);
        ui.theme = Theme::new(ThemeName::KanagawaWave);

        ui.toggle_mode();
        assert_eq!(ui.theme.name, ThemeName::KanagawaLotus);
        assert!(!ui.config.use_system_theme);

        ui.next_theme();
        let current_light = *ui.theme.name;
        assert_eq!(ui.config.last_light, Some(current_light));

        ui.toggle_mode();
        assert_eq!(ui.theme.name, ThemeName::KanagawaWave);

        ui.toggle_mode();
        assert_eq!(ui.theme.name, current_light);
    }

    #[test]
    fn should_apply_system_theme() {
        let mut config = UIConfig::default();
        config.use_system_theme = true;
        config.last_dark = Some(ThemeName::GruvboxDark);
        config.last_light = Some(ThemeName::GruvboxLight);

        let mut ui = UIState::new(config);
        ui.apply_system_theme();

        let is_one_of_ours =
            ui.theme.name == ThemeName::GruvboxDark || ui.theme.name == ThemeName::GruvboxLight;
        assert!(is_one_of_ours);
    }

    #[test]
    fn should_toggle_sidebar_showing() {
        let mut ui = UIState::default();
        assert!(ui.config.show_sidebar);

        ui.toggle_sidebar();
        assert!(!ui.config.show_sidebar);

        ui.toggle_sidebar();
        assert!(ui.config.show_sidebar);
    }

    #[test]
    fn show_test_redraw_lifecycle() {
        let mut ui = UIState::default();

        assert!(
            !ui.needs_redraw(),
            "Initial state should not require redraw"
        );

        ui.request_redraw();
        assert!(
            ui.needs_redraw(),
            "Flag should be true after request_redraw"
        );

        ui.clear_redraw_flag();
        assert!(
            !ui.needs_redraw(),
            "Flag should be false after clear_redraw_flag"
        );
    }

    #[test]
    fn should_send_signal_to_redraw_only_once() {
        let mut ui = UIState::default();

        ui.request_redraw();
        ui.request_redraw();

        assert!(ui.needs_redraw());

        ui.clear_redraw_flag();
        assert!(!ui.needs_redraw());
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
