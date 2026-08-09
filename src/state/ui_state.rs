use crate::{
    config::{BehaviorConfig, UIConfig},
    core::{FocusArea, Selectable},
    models::TaskFilter,
    state::{ActiveModal, AdaptiveScroll, ApplicationResult, ApplicationState, Session},
    theme::{BuiltinTheme, Theme, ThemeId},
    ui::{
        Confirm, Notification, Popup, TextInput,
        widgets::{
            input::Input,
            modal::{Modal, ModalAction, ModalSize},
        },
    },
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Main application UI state (only for rendering purposes)
pub struct UIState {
    pub active_tab: SidebarTab,
    pub active_folder: Option<Uuid>,

    pub focused: Selectable<FocusArea>,
    pub modal: Option<ActiveModal>,
    pub search_input: Option<TextInput>,

    pub desc_scroll: AdaptiveScroll,
    pub hotkeys_scroll: AdaptiveScroll,

    pub theme: Theme,
    pub config: UIConfig,

    should_redraw: bool,
}

/// Application sidebar tab (main filters)
#[derive(
    Default,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "title_case")]
pub enum SidebarTab {
    #[default]
    Inbox,
    Active,
    Completed,
    HighPriority,
    Today,
}

impl UIState {
    pub const MIN_SIDEBAR: u16 = 10;
    pub const MAX_SIDEBAR: u16 = 100;
    pub const DEFAULT_SIDEBAR: u16 = 30;

    pub fn new(mut config: UIConfig) -> Self {
        config
            .last_dark
            .get_or_insert(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        config
            .last_light
            .get_or_insert(ThemeId::Builtin(BuiltinTheme::GruvboxLight));

        let theme: Theme = Theme::new(config.theme.clone());

        Self {
            active_tab: SidebarTab::Inbox,
            active_folder: None,
            focused: Selectable::default(),
            modal: None,
            search_input: None,
            desc_scroll: AdaptiveScroll::default(),
            hotkeys_scroll: AdaptiveScroll::default(),
            theme,
            config,
            should_redraw: true,
        }
    }

    /// Apply UI state from loaded session
    pub fn apply_session(&mut self, session: Session) {
        session.apply_to(self);
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

    /// Return id of a task based on TableState selection
    pub fn selected_id(&self, state: &ApplicationState, config: &BehaviorConfig) -> Option<Uuid> {
        let index: usize = state.select_state.selected()?;
        let filter = TaskFilter::new(self.active_tab, self.active_folder, self.search_query());
        filter
            .apply(&state.tasks, config)
            .get(index)
            .map(|task| task.id)
    }

    /// Toggle dark/light theme mode
    pub fn toggle_mode(&mut self) {
        self.config.use_system_theme = false;
        let current_id: ThemeId = self.theme.theme_id().clone();

        if self.theme.is_dark() {
            self.config.last_dark = Some(current_id);
        } else {
            self.config.last_light = Some(current_id);
        }

        let target = if self.theme.is_dark() {
            self.config
                .last_light
                .clone()
                .unwrap_or(ThemeId::Builtin(BuiltinTheme::GruvboxLight))
        } else {
            self.config
                .last_dark
                .clone()
                .unwrap_or(ThemeId::Builtin(BuiltinTheme::GruvboxDark))
        };

        log::info!("Manual mode toggle. Target: {}", target);
        self.apply_theme_id(target);
    }

    /// Helper function to refresh theme
    pub fn refresh_theme(&mut self) {
        if self.config.use_system_theme {
            self.apply_system_theme();
        } else {
            if self.theme.theme_id() != &self.config.theme {
                self.apply_theme_id(self.config.theme.clone());
            }
        }
    }

    /// Applies theme based on system preferences
    pub fn apply_system_theme(&mut self) -> bool {
        let is_dark = dark_light::detect()
            .map(|m| m == dark_light::Mode::Dark)
            .unwrap_or(true);

        let target = if is_dark {
            self.config.last_dark.clone().unwrap()
        } else {
            self.config.last_light.clone().unwrap()
        };

        if self.config.theme == target {
            return false;
        }

        log::info!("System theme change detected: {}", target);
        self.apply_theme_id(target);
        true
    }

    /// Function to change theme by ID
    pub fn apply_theme_id(&mut self, id: ThemeId) {
        if let Some(index) = self.theme.registry.all_ids.iter().position(|x| x == &id) {
            self.theme.registry.current_index = index;
        } else {
            self.theme = Theme::new(id.clone());
        }

        self.config.theme = id;
        self.request_redraw();
    }

    /// Next filter tab (including dynamic folders)
    pub fn next_tab_filter(&mut self, folder_ids: &[Uuid]) {
        self.move_sidebar(1, folder_ids);
    }

    /// Previous filter tab (including dynamic folders)
    pub fn prev_tab_filter(&mut self, folder_ids: &[Uuid]) {
        self.move_sidebar(-1, folder_ids);
    }

    /// Centralized method to sidebar navigation
    pub fn move_sidebar(&mut self, direction: i32, folder_ids: &[Uuid]) {
        let static_count = 5;
        let has_divider: bool = !folder_ids.is_empty();
        let divider_count = if has_divider { 1 } else { 0 };
        let total_count = static_count + divider_count + folder_ids.len();

        let mut current_idx = match self.active_folder {
            Some(id) => {
                let folder_pos = folder_ids.iter().position(|&x| x == id).unwrap_or(0);
                static_count + divider_count + folder_pos
            }
            None => self.active_tab as usize,
        };

        if direction > 0 {
            current_idx = (current_idx + 1) % total_count;
            if has_divider && current_idx == static_count {
                current_idx += 1;
            }
        } else {
            if current_idx == 0 {
                current_idx = total_count - 1;
            } else {
                current_idx -= 1;
                if has_divider && current_idx == static_count {
                    current_idx -= 1;
                }
            }
        }

        if current_idx < static_count {
            self.active_tab = match current_idx {
                0 => SidebarTab::Inbox,
                1 => SidebarTab::Active,
                2 => SidebarTab::Completed,
                3 => SidebarTab::HighPriority,
                _ => SidebarTab::Today,
            };
            self.active_folder = None;
        } else {
            let folder_pos = current_idx - static_count - divider_count;
            self.active_folder = Some(folder_ids[folder_pos]);
        }

        log::trace!(
            "Sidebar position changed to: Tab={:?}, Folder={:?}",
            self.active_tab,
            self.active_folder
        );
    }

    /// Changes to specific filter
    pub fn change_filter(&mut self, tab: SidebarTab, folder_id: Option<Uuid>) {
        self.active_tab = tab;
        self.active_folder = folder_id;
        log::trace!(
            "Sidebar manually set to: Tab={:?}, Folder={:?}",
            self.active_tab,
            self.active_folder
        );
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

    /// Toggle footer
    pub fn toggle_footer(&mut self) {
        self.config.show_footer = !self.config.show_footer;
    }

    /// Make sidebar bigger
    pub fn increase_sidebar(&mut self) {
        self.config.sidebar_width = (self.config.sidebar_width + 2).min(Self::MAX_SIDEBAR);
        self.should_redraw = true;
    }

    /// Make sidebar smaller
    pub fn decrease_sidebar(&mut self) {
        self.config.sidebar_width = self
            .config
            .sidebar_width
            .saturating_sub(2)
            .max(Self::MIN_SIDEBAR);
        self.should_redraw = true;
    }

    /// Fully reset UI state with theme
    pub fn reset_ui(&mut self) {
        self.config = UIConfig::default();
        self.theme = Theme::default();
        self.should_redraw = true;
    }

    /// Closes existing modal widget
    pub fn close_modal(&mut self) {
        if let Some(m) = &self.modal {
            log::debug!("Closing modal: action={:?}", m.action);
        }

        self.modal = None;
    }

    /// Opens remove confirm widget for a specific task
    pub fn remove_task_confirm(&mut self) {
        self.show_modal(
            Confirm::new("Are you sure to remove selected task?"),
            ModalAction::Remove,
        );
    }

    /// Opens remove confirm widget for a specific folder
    pub fn remove_folder_confirm(&mut self, folder_id: Uuid) {
        self.show_modal(
            Confirm::new("Are you sure remove folder with all its tasks?"),
            ModalAction::RemoveFolder(folder_id),
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

impl Default for UIState {
    fn default() -> Self {
        Self::new(UIConfig::default())
    }
}

/// Unit-tests for UIState
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{StorageError, TaskError},
        models::{Priority, Task},
        ui::Popup,
    };
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };

    #[test]
    fn should_navigate_through_filters_and_dynamic_folders() {
        let mut ui = UIState::default();
        ui.active_tab = SidebarTab::Inbox;
        ui.active_folder = None;

        let folder_a = Uuid::new_v4();
        let folder_b = Uuid::new_v4();
        let folders_ids = vec![folder_a, folder_b];

        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_tab, SidebarTab::Active);

        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_tab, SidebarTab::Completed);

        ui.prev_tab_filter(&folders_ids);
        assert_eq!(ui.active_tab, SidebarTab::Active);

        ui.next_tab_filter(&folders_ids);
        ui.next_tab_filter(&folders_ids);
        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_tab, SidebarTab::Today);

        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_folder, Some(folder_a));

        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_folder, Some(folder_b));

        ui.next_tab_filter(&folders_ids);
        assert_eq!(ui.active_tab, SidebarTab::Inbox);
        assert_eq!(ui.active_folder, None);

        ui.prev_tab_filter(&folders_ids);
        assert_eq!(ui.active_folder, Some(folder_b));

        ui.change_filter(SidebarTab::HighPriority, None);
        assert_eq!(ui.active_tab, SidebarTab::HighPriority);
        assert_eq!(ui.active_folder, None);
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
        let error: StorageError = StorageError::Database {
            path: PathBuf::default(),
            src: "Some error".to_string(),
        };

        ui.show_result_popup(Err(error.into()));
        assert!(ui.modal.is_some());
    }

    #[test]
    fn should_handle_notification_pushing() {
        let ui = UIState::default();
        let mut state = ApplicationState::default();

        ui.push_notification(&mut state, Ok("Success message".to_string()));
        assert!(state.notification.is_some());

        let error_res: ApplicationResult<String> = Err(TaskError::EmptyTitle.into());
        ui.push_notification(&mut state, error_res);
        assert!(state.notification.is_some());
    }

    #[test]
    fn should_return_id_of_selected_task() {
        let mut ui = UIState::default();
        let mut state = ApplicationState::default();
        let config = BehaviorConfig::default();

        let tasks: Vec<Task> = vec![
            Task::new("Task 1"),
            Task::new("Task 2").with_priority(Priority::Medium),
        ];
        let last_id: Uuid = tasks[1].id;
        state.tasks = tasks;
        state.select_state.select(Some(1));

        ui.active_tab = SidebarTab::Inbox;

        let expected_id: Uuid = ui.selected_id(&state, &config).unwrap();
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
    fn should_apply_system_theme() {
        let mut config = UIConfig::default();
        config.use_system_theme = true;
        let dark = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        let light = ThemeId::Builtin(BuiltinTheme::GruvboxLight);

        config.last_dark = Some(dark.clone());
        config.last_light = Some(light.clone());

        let mut ui = UIState::new(config);
        ui.apply_system_theme();

        let current = ui.theme.theme_id();
        assert!(current == &dark || current == &light);
    }

    #[test]
    fn should_toggle_sidebar_and_footer_showing() {
        let mut ui = UIState::default();
        assert!(ui.config.show_sidebar);
        assert!(ui.config.show_footer);

        ui.toggle_sidebar();
        assert!(!ui.config.show_sidebar);
        ui.toggle_footer();
        assert!(!ui.config.show_footer);

        ui.toggle_sidebar();
        assert!(ui.config.show_sidebar);
        ui.toggle_footer();
        assert!(ui.config.show_footer);
    }

    #[test]
    fn should_handle_sidebar_increase_limit() {
        let mut ui = UIState::default();
        ui.config.sidebar_width = 98;

        ui.increase_sidebar();
        assert_eq!(ui.config.sidebar_width, 100);

        ui.increase_sidebar();
        assert_eq!(ui.config.sidebar_width, 100);
    }

    #[test]
    fn should_handle_sidebar_decrease_limit() {
        let mut ui = UIState::default();
        ui.config.sidebar_width = 12;

        ui.decrease_sidebar();
        assert_eq!(ui.config.sidebar_width, 10);

        ui.decrease_sidebar();
        assert_eq!(ui.config.sidebar_width, 10);
    }

    #[test]
    fn should_handle_reset_ui() {
        let mut ui = UIState::default();
        ui.config.sidebar_width = 15;
        ui.config.show_sidebar = false;

        ui.reset_ui();

        assert_eq!(ui.config.sidebar_width, UIConfig::default().sidebar_width);
        assert!(ui.config.show_sidebar);
    }

    #[test]
    fn show_test_redraw_lifecycle() {
        let mut ui = UIState::default();
        assert!(
            ui.needs_redraw(),
            "Initial state should require redraw for first frame"
        );

        ui.clear_redraw_flag();
        assert!(
            !ui.needs_redraw(),
            "Flag should be false after clear_redraw_flag"
        );

        ui.request_redraw();
        assert!(
            ui.needs_redraw(),
            "Flag should be true after request_redraw"
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
    fn should_open_remove_task_confirm() {
        let mut ui = UIState::default();
        ui.remove_task_confirm();

        assert!(ui.modal.is_some());
        assert_eq!(ui.modal.unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_open_remove_folder_confirm() {
        let mut ui = UIState::default();
        let folder_id = Uuid::new_v4();
        ui.remove_folder_confirm(folder_id);

        assert!(ui.modal.is_some());
        assert_eq!(
            ui.modal.unwrap().action,
            ModalAction::RemoveFolder(folder_id)
        );
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

    #[test]
    fn should_handle_infolder_filter_switching() {
        let mut ui = UIState::default();
        let folder_id = Uuid::new_v4();

        ui.change_filter(SidebarTab::Inbox, Some(folder_id));
        assert_eq!(ui.active_tab, SidebarTab::Inbox);
        assert_eq!(ui.active_folder, Some(folder_id));

        ui.remove_folder_confirm(folder_id);
        assert!(ui.modal.is_some());
        assert_eq!(
            ui.modal.unwrap().action,
            ModalAction::RemoveFolder(folder_id)
        );
    }
}
