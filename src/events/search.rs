use crate::{
    app::ApplicationController,
    core::{ApplicationMode, FocusArea},
    ui::{WidgetResponse, widgets::input::Input},
};

/// Handle keys for task search mode
pub fn handle_search(
    event: ratatui::crossterm::event::KeyEvent,
    ctrl: &mut ApplicationController,
    mode: &mut ApplicationMode,
) {
    let input = match &mut ctrl.ui.search_input {
        Some(i) => i,
        None => return,
    };

    match input.handle_key(&event.code) {
        WidgetResponse::Submit => {
            *mode = ApplicationMode::List;
            ctrl.ui.focused.set(FocusArea::Main);
        }
        WidgetResponse::Cancel => {
            ctrl.ui.search_input = None;
            *mode = ApplicationMode::Navigation;
            ctrl.ui.focused.set(FocusArea::Sidebar);
        }
        WidgetResponse::Continue => {
            ctrl.state.select_state.select(Some(0));
        }
        _ => {}
    }

    ctrl.ui.request_redraw();
}

/// Unit-tests for search mode key handler
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{Config, KeyMaps},
        core::Storage,
        models::Task,
        state::{ApplicationState, UIState},
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tempdir::TempDir;

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    struct TestContext {
        _temp_dir: TempDir,
        _storage: Storage,
        state: ApplicationState,
        ui: UIState,
        config: Config,
        keymaps: KeyMaps,
    }

    impl TestContext {
        fn new() -> Self {
            let temp_dir = TempDir::new("search_event_test").unwrap();
            let db_path = temp_dir.path().join("test_search.db");
            let config = Config::default();
            let storage = Storage::init(Some(&db_path), &config.storage).unwrap();

            Self {
                _temp_dir: temp_dir,
                _storage: storage,
                state: ApplicationState::default(),
                ui: UIState::default(),
                config,
                keymaps: KeyMaps::default(),
            }
        }

        pub fn components(
            &mut self,
        ) -> (&mut ApplicationState, &mut UIState, &mut Config, &KeyMaps) {
            (
                &mut self.state,
                &mut self.ui,
                &mut self.config,
                &self.keymaps,
            )
        }
    }

    #[test]
    fn should_exit_search_to_list_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_search(key_event(KeyCode::Enter), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::List);
        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_cancel_search_and_return_to_left_panel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_search(key_event(KeyCode::Esc), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Navigation);
        assert_eq!(ctrl.ui.focused, FocusArea::Sidebar);
        assert!(ctrl.ui.search_input.is_none());
    }

    #[test]
    fn should_reset_selection_to_first_item_on_typing() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps) = ctx.components();

        state.tasks.push(Task::new("Apple"));
        state.tasks.push(Task::new("Banana"));
        state.select_state.select(Some(1));
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_search(key_event(KeyCode::Char('a')), &mut ctrl, &mut mode);
        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_handle_empty_input_gracefully() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        state.tasks.clear();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_search(key_event(KeyCode::Char('x')), &mut ctrl, &mut mode);
        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }
}
