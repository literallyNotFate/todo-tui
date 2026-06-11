use crate::{
    common::default_bool_is_true,
    core::{FocusArea, Selectable},
    state::{AdaptiveScroll, SidebarTab, UIState},
    ui::TextInput,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session (current UI state) to save to file/load from
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Session {
    pub last_selected_id: Option<Uuid>,
    pub last_focus: Selectable<FocusArea>,

    pub last_tab: Selectable<SidebarTab>,
    pub last_folder_id: Option<Uuid>,

    pub last_query: String,
    pub description_scroll_pos: u16,
    pub hotkeys_scroll_pos: u16,

    #[serde(default = "default_bool_is_true")]
    pub use_system_theme: bool,
}

impl Session {
    /// Creates session based on current state of the UI
    pub fn from_state(ui: &UIState, selected_id: Option<Uuid>) -> Self {
        Self {
            last_selected_id: selected_id,
            last_focus: ui.focused.clone(),
            last_tab: Selectable::new(ui.active_tab),
            last_folder_id: ui.active_folder,
            last_query: ui
                .search_input
                .as_ref()
                .map(|i| i.buffer.clone())
                .unwrap_or_default(),
            use_system_theme: ui.config.use_system_theme,
            description_scroll_pos: ui.desc_scroll.current.get(),
            hotkeys_scroll_pos: ui.hotkeys_scroll.current.get(),
        }
    }

    /// Modifies UI state using current session data
    pub fn apply_to(&self, ui: &mut UIState) {
        ui.active_tab = self.last_tab.value;
        ui.active_folder = self.last_folder_id;
        ui.focused = self.last_focus.clone();
        ui.desc_scroll = AdaptiveScroll::with_position(self.description_scroll_pos);
        ui.hotkeys_scroll = AdaptiveScroll::with_position(self.hotkeys_scroll_pos);
        ui.config.use_system_theme = self.use_system_theme;

        ui.refresh_theme();

        ui.search_input =
            (!self.last_query.is_empty()).then(|| TextInput::from(self.last_query.clone()));
    }
}

/// Unit-tests for session
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::FocusArea, state::SidebarTab, state::Task};

    fn setup_ui() -> UIState {
        UIState::default()
    }

    #[test]
    fn should_correctly_create_session_from_ui_state() {
        let mut ui = setup_ui();
        let expected_task_id = Uuid::new_v4();
        let expected_folder_id = Uuid::new_v4();

        ui.active_tab = SidebarTab::HighPriority;
        ui.active_folder = Some(expected_folder_id);
        ui.focused = Selectable::new(FocusArea::Sidebar);
        ui.config.use_system_theme = false;
        ui.search_input = Some(TextInput::from("rust match"));
        ui.desc_scroll = AdaptiveScroll::with_position(15);
        ui.hotkeys_scroll = AdaptiveScroll::with_position(30);

        let session = Session::from_state(&ui, Some(expected_task_id));

        assert_eq!(session.last_selected_id, Some(expected_task_id));
        assert_eq!(session.last_tab.value, SidebarTab::HighPriority);
        assert_eq!(session.last_folder_id, Some(expected_folder_id));
        assert_eq!(session.last_focus.value, FocusArea::Sidebar);
        assert_eq!(session.last_query, "rust match");
        assert!(!session.use_system_theme);
        assert_eq!(session.description_scroll_pos, 15);
        assert_eq!(session.hotkeys_scroll_pos, 30);
    }

    #[test]
    fn should_correctly_apply_session_to_ui_state() {
        let mut ui = setup_ui();
        let expected_folder_id = Uuid::new_v4();
        let expected_task_id = Uuid::new_v4();

        let mut task = Task::new("todo filter");
        task.id = expected_task_id;

        let session = Session {
            last_selected_id: Some(expected_task_id),
            last_focus: Selectable::new(FocusArea::Main),
            last_tab: Selectable::new(SidebarTab::Completed),
            last_folder_id: Some(expected_folder_id),
            last_query: "todo filter".to_string(),
            description_scroll_pos: 120,
            hotkeys_scroll_pos: 5,
            use_system_theme: true,
        };

        session.apply_to(&mut ui);

        assert_eq!(ui.active_tab, SidebarTab::Completed);
        assert_eq!(ui.active_folder, Some(expected_folder_id));
        assert_eq!(ui.focused.value, FocusArea::Main);
        assert!(ui.config.use_system_theme);
        assert_eq!(ui.desc_scroll.current.get(), 120);
        assert_eq!(ui.hotkeys_scroll.current.get(), 5);
        assert!(ui.search_input.is_some());
        assert_eq!(ui.search_input.unwrap().buffer, "todo filter");
    }

    #[test]
    fn should_handle_empty_query_with_none_search_input() {
        let mut ui = setup_ui();

        let session = Session {
            last_query: "".to_string(),
            last_tab: Selectable::new(SidebarTab::Inbox),
            last_focus: Selectable::new(FocusArea::Main),
            ..Session::default()
        };

        session.apply_to(&mut ui);
        assert!(ui.search_input.is_none());
    }
}
