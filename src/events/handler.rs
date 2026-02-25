use crate::{
    Application,
    app::ApplicationController,
    core::ApplicationMode,
    enums::{FocusArea, WidgetResponse},
    models::{Filter, Todo},
    traits::{Input, ModalAction, ModalResult},
    ui::{Form, is_terminal_small},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct EventHandler;

/// Handling all possible keys
impl EventHandler {
    /// Main key event handling
    pub fn handle_key(app: &mut Application, event: KeyEvent) {
        if is_terminal_small(app.size.0, app.size.1) {
            if is_exit_key(&event) {
                app.running = false;
            }

            return;
        }

        if is_kill_process_key(&event) {
            app.running = false;
            return;
        }

        match app.mode {
            ApplicationMode::Form | ApplicationMode::Search => {
                let mut ctrl =
                    ApplicationController::new(&mut app.data, &mut app.ui, &mut app.config);

                if event.code == KeyCode::Esc {
                    ctrl.ui.task_form = None;
                    ctrl.ui.search_input = None;
                    drop(ctrl);

                    app.restore_base_mode();
                } else if app.mode == ApplicationMode::Form {
                    Self::handle_form_mode(event, &mut ctrl, &mut app.mode);
                } else {
                    Self::handle_search_mode(event, &mut ctrl, &mut app.mode);
                }

                return;
            }
            _ if app.ui.modal.is_some() => {
                let mut ctrl =
                    ApplicationController::new(&mut app.data, &mut app.ui, &mut app.config);
                Self::handle_modal(event, &mut ctrl, &mut app.running);
                return;
            }
            _ => {}
        }

        let mut ctrl = ApplicationController::new(&mut app.data, &mut app.ui, &mut app.config);
        match (event.code, event.modifiers) {
            (KeyCode::Char('q') | KeyCode::Esc, KeyModifiers::NONE) => {
                if ctrl.state.any_unsaved_changes() {
                    ctrl.ui.unsaved_confirm();
                } else {
                    drop(ctrl);

                    app.config.update_from_ui(&app.ui);
                    let _ = app.config.save(None);
                    app.running = false;
                }
            }
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                ctrl.ui.task_form = Some(Form::new());
                app.mode = ApplicationMode::Form;
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if ctrl.config.behavior.confirm_before_save {
                    ctrl.ui.save_confirm();
                } else {
                    let _ = ctrl.dispatch_save();
                }
            }
            (KeyCode::Char('t'), KeyModifiers::NONE) => ctrl.ui.next_theme(),
            (KeyCode::Char('t'), KeyModifiers::CONTROL) => ctrl.ui.prev_theme(),
            (KeyCode::Char('b'), KeyModifiers::NONE) => ctrl.ui.toggle_sidebar(),
            (KeyCode::Char('m'), KeyModifiers::NONE) => ctrl.ui.toggle_mode(),
            (KeyCode::Char('x'), KeyModifiers::NONE) => ctrl.ui.clear_confirm(),
            (KeyCode::Char('j'), KeyModifiers::ALT) => ctrl.ui.sidebar_scroll.scroll_down(),
            (KeyCode::Char('k'), KeyModifiers::ALT) => ctrl.ui.sidebar_scroll.scroll_up(),
            (KeyCode::Char('a'), KeyModifiers::ALT) => app.autosave.toggle_enabled(),
            (KeyCode::Char('s'), KeyModifiers::NONE) => {
                ctrl.state.sort.parameter = ctrl.state.sort.parameter.next();
                ctrl.dispatch_sorting();
            }
            (KeyCode::Char('r'), KeyModifiers::NONE) => {
                ctrl.state.sort.order = ctrl.state.sort.order.next();
                ctrl.dispatch_sorting();
            }
            _ => Self::handle_main_mode(event, &mut ctrl, &mut app.mode, &mut app.running),
        }
    }

    /// Handle modal keys (confirm/popup)
    fn handle_modal(event: KeyEvent, ctrl: &mut ApplicationController, running: &mut bool) {
        let result = {
            let modal_wrapper = ctrl.ui.modal.as_mut().unwrap();
            modal_wrapper.modal.handle_key(event.code)
        };

        if let Some(result) = result {
            let action: ModalAction = ctrl.ui.modal.as_ref().unwrap().action.clone();
            ctrl.ui.close_modal();

            if result == ModalResult::Confirmed {
                match action {
                    ModalAction::Remove => ctrl.dispatch_remove(),
                    ModalAction::Clear => ctrl.dispatch_clear(),
                    ModalAction::Save => {
                        let _ = ctrl.dispatch_save();
                    }
                    ModalAction::UnsavedExit => {
                        if ctrl.dispatch_save() {
                            *running = false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Handles main mode keys
    fn handle_main_mode(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        mode: &mut ApplicationMode,
        _running: &mut bool,
    ) {
        match event.code {
            KeyCode::Char('h') | KeyCode::Char('l') => {
                ctrl.ui.toggle_focus();
                *mode = match ctrl.ui.focus_area {
                    FocusArea::LeftPanel => ApplicationMode::Browsing,
                    FocusArea::MainContent => ApplicationMode::List,
                };
            }
            _ => match ctrl.ui.focus_area {
                FocusArea::LeftPanel => Self::handle_left_panel(event.code, ctrl),
                FocusArea::MainContent => Self::handle_main_content(event.code, ctrl, mode),
            },
        }
    }

    /// Handles main content keys
    fn handle_main_content(
        code: KeyCode,
        ctrl: &mut ApplicationController,
        mode: &mut ApplicationMode,
    ) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => ctrl.dispatch_move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => ctrl.dispatch_move_selection(-1),
            KeyCode::Enter => ctrl.dispatch_toggle(),
            KeyCode::Char('e') => {
                if let Some(id) = ctrl.ui.selected_id(ctrl.state) {
                    if let Some(task) = ctrl.state.todos.iter().find(|t| t.id == id) {
                        ctrl.ui.task_form = Some(Form::from(&task));
                        *mode = ApplicationMode::Form;
                    }
                }
            }
            KeyCode::Char('d') => {
                if ctrl.config.behavior.confirm_before_remove {
                    ctrl.ui.remove_confirm();
                } else {
                    ctrl.dispatch_remove()
                }
            }
            KeyCode::Char('J') => ctrl.dispatch_move_tasks(1),
            KeyCode::Char('K') => ctrl.dispatch_move_tasks(-1),
            KeyCode::Char('[') => ctrl.ui.desc_scroll.scroll_up(),
            KeyCode::Char(']') => ctrl.ui.desc_scroll.scroll_down(),
            KeyCode::Char('/') => {
                ctrl.ui.show_search();
                *mode = ApplicationMode::Search;
            }
            _ => {}
        }
    }

    /// Handles left panel keys
    fn handle_left_panel(code: KeyCode, ctrl: &mut ApplicationController) {
        match code {
            KeyCode::Char('j') | KeyCode::Down => ctrl.ui.next_tab_filter(),
            KeyCode::Char('k') | KeyCode::Up => ctrl.ui.prev_tab_filter(),
            KeyCode::Char('1') => ctrl.ui.change_filter(Filter::All),
            KeyCode::Char('2') => ctrl.ui.change_filter(Filter::Active),
            KeyCode::Char('3') => ctrl.ui.change_filter(Filter::Completed),
            KeyCode::Char('4') => ctrl.ui.change_filter(Filter::HighPriority),
            KeyCode::Char('5') => ctrl.ui.change_filter(Filter::Today),
            _ => return,
        }

        ctrl.stabilize_ui_focus(None);
    }

    /// Handles form keys
    fn handle_form_mode(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        mode: &mut ApplicationMode,
    ) {
        if let Some(form) = &mut ctrl.ui.task_form {
            match form.handle_key(&event) {
                WidgetResponse::Submit => {
                    let (id, title, desc, priority) = form.data();
                    if let Some(task_id) = id {
                        let updated: Todo = Todo::from_id(task_id, title, desc, Some(priority));
                        ctrl.dispatch_update(task_id, updated);
                    } else {
                        ctrl.dispatch_append(title, desc, Some(priority));
                    }

                    ctrl.ui.task_form = None;
                    *mode = ApplicationMode::List;
                }
                WidgetResponse::Cancel => {
                    ctrl.ui.task_form = None;
                    *mode = ApplicationMode::List;
                }
                WidgetResponse::Continue => {}
            }
        }
    }

    /// Handles search keys
    fn handle_search_mode(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        mode: &mut ApplicationMode,
    ) {
        if let Some(input) = &mut ctrl.ui.search_input {
            match input.handle_key(&event.code) {
                WidgetResponse::Submit => {
                    *mode = ApplicationMode::List;
                    ctrl.ui.focus_area = FocusArea::MainContent;
                }
                WidgetResponse::Cancel => {
                    ctrl.ui.search_input = None;
                    *mode = ApplicationMode::Browsing;
                    ctrl.ui.focus_area = FocusArea::LeftPanel;
                }
                WidgetResponse::Continue => ctrl.state.select_state.select(Some(0)),
            }
        }
    }
}

/// Check if its kill process key
pub fn is_kill_process_key(key_event: &KeyEvent) -> bool {
    key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL)
}

/// Check if its exit key
pub fn is_exit_key(key_event: &KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc)
        || (key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL))
}

/// Unit-tests for event handler
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        models::{SortBy, SortOrder},
        state::{ApplicationState, UIState},
    };
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    fn setup_app() -> Application {
        let mut app = Application::test();
        app.size = (100, 100);
        app
    }

    fn setup_ctx() -> (ApplicationState, UIState, ApplicationMode, Config, bool) {
        let state = ApplicationState::default();
        let ui = UIState::default();
        let mode = ApplicationMode::List;
        let config = Config::default();
        let running = true;
        (state, ui, mode, config, running)
    }

    fn mock_unsaved_modal(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        running: &mut bool,
        path: &Path,
    ) {
        let result = {
            let modal_wrapper = ctrl.ui.modal.as_mut().unwrap();
            modal_wrapper.modal.handle_key(event.code)
        };

        if let Some(result) = result {
            let action: ModalAction = ctrl.ui.modal.as_ref().unwrap().action.clone();
            ctrl.ui.close_modal();

            if result == ModalResult::Confirmed && action == ModalAction::UnsavedExit {
                match ctrl.state.save(Some(path)) {
                    Ok(string) => ctrl.ui.show_result_popup(Ok(string)),
                    Err(e) => ctrl.ui.show_result_popup(Err(e)),
                }

                *running = false;
            }
        }
    }

    #[test]
    fn should_block_input_when_terminal_is_too_small() {
        let mut app = setup_app();
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(app.ui.task_form.is_none());
        assert!(app.running);
    }

    #[test]
    fn should_allow_exit_even_in_small_terminal() {
        let mut app = setup_app();
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        EventHandler::handle_key(&mut app, event);

        assert!(!app.running);
    }

    #[test]
    fn should_open_unsaved_confirm_on_exit_if_changes_exist() {
        let mut app = setup_app();
        app.data.todos.push(Todo::new("Changes", "", None));

        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);

        assert!(app.running, "App should not close yet");
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_restore_mode_on_esc_from_form() {
        let mut app = setup_app();
        app.mode = ApplicationMode::Form;
        app.ui.task_form = Some(Form::new());

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(app.ui.task_form.is_none());
        assert!(app.running);
    }

    #[test]
    fn should_prioritize_modal_over_global_keys() {
        let mut app = setup_app();
        app.ui.clear_confirm();

        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(app.ui.task_form.is_none());
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_trigger_save_on_ctrl_s() {
        let mut app = setup_app();
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);

        EventHandler::handle_key(&mut app, event);

        assert!(app.ui.modal.is_some(), "Save modal should appear");
    }

    #[test]
    fn should_handle_sort_keys() {
        let mut app = setup_app();

        assert_eq!(app.data.sort.parameter, SortBy::Priority);
        assert_eq!(app.data.sort.order, SortOrder::Desc);

        let mut event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.data.sort.parameter, SortBy::Title);

        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.data.sort.parameter, SortBy::CreatedAt);

        event = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.data.sort.order, SortOrder::Asc);
    }

    #[test]
    fn should_scroll_through_sidebar() {
        let mut app = setup_app();

        app.ui.sidebar_scroll.current.set(10);
        app.ui.sidebar_scroll.max_scroll.set(20);

        let mut event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.ui.sidebar_scroll.current.get(), 11);

        event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.ui.sidebar_scroll.current.get(), 12);

        event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert_eq!(app.ui.sidebar_scroll.current.get(), 11);
    }

    #[test]
    fn should_toggle_autosave() {
        let mut app = setup_app();
        assert!(!app.autosave.enabled);

        let mut event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert!(app.autosave.enabled);

        event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert!(!app.autosave.enabled);
    }

    #[test]
    fn should_toggle_focus_right_with_l() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        ui.focus_area = FocusArea::LeftPanel;
        let mut mode = ApplicationMode::Browsing;

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);

        EventHandler::handle_main_mode(event, &mut ctrl, &mut mode, &mut running);

        assert_eq!(ctrl.ui.focus_area, FocusArea::MainContent);
        assert_eq!(mode, ApplicationMode::List);
    }

    #[test]
    fn should_toggle_focus_left_with_h() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        ui.focus_area = FocusArea::MainContent;
        let mut mode = ApplicationMode::List;

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);

        EventHandler::handle_main_mode(event, &mut ctrl, &mut mode, &mut running);

        assert_eq!(ctrl.ui.focus_area, FocusArea::LeftPanel);
        assert_eq!(mode, ApplicationMode::Browsing);
    }

    #[test]
    fn should_test_delegation_to_left_panel() {
        let (mut state, mut ui, mut mode, mut config, mut running) = setup_ctx();
        ui.focus_area = FocusArea::LeftPanel;
        ui.current_filter = Filter::All;

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        EventHandler::handle_main_mode(event, &mut ctrl, &mut mode, &mut running);

        assert_eq!(ctrl.ui.current_filter, Filter::Active);
    }

    #[test]
    fn should_test_delegation_to_main_content() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        state.todos.push(Todo::new("T1", "", None));
        state.todos.push(Todo::new("T2", "", None));
        state.select_state.select(Some(0));

        ui.focus_area = FocusArea::MainContent;
        let mut mode = ApplicationMode::List;

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        EventHandler::handle_main_mode(event, &mut ctrl, &mut mode, &mut running);

        assert_eq!(ctrl.state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_confirm_remove_task() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        state.todos.push(Todo::new("To be deleted", "", None));
        state.select_state.select(Some(0));

        ui.remove_confirm();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        EventHandler::handle_modal(event, &mut ctrl, &mut running);

        assert_eq!(ctrl.state.todos.len(), 0);
        assert!(ctrl.ui.modal.is_none());
        assert!(running, "App should be running still");
    }

    #[test]
    fn should_cancel_clear_all_tasks() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        state.todos.push(Todo::new("Keep me", "", None));

        ui.clear_confirm();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        EventHandler::handle_modal(event, &mut ctrl, &mut running);

        assert_eq!(ctrl.state.todos.len(), 1);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_save_and_exit_on_unsaved_exit_confirm() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        ui.unsaved_confirm();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        mock_unsaved_modal(event, &mut ctrl, &mut running, &path);

        assert!(!running, "App should be closed");
    }

    #[test]
    fn should_not_exit_on_unsaved_exit_cancel() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        ui.unsaved_confirm();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
        EventHandler::handle_modal(event, &mut ctrl, &mut running);

        assert!(running, "App should not be closed if cancelled");
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    #[should_panic]
    fn should_panic_if_no_modal_exists() {
        let (mut state, mut ui, _, mut config, mut running) = setup_ctx();
        ui.modal = None;
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        EventHandler::handle_modal(event, &mut ctrl, &mut running);
    }

    #[test]
    fn should_test_navigation_down_up() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("T1", "", None));
        state.todos.push(Todo::new("T2", "", None));
        state.todos.push(Todo::new("T3", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_main_content(KeyCode::Char('j'), &mut ctrl, &mut mode);
        assert_eq!(ctrl.state.select_state.selected(), Some(1));

        EventHandler::handle_main_content(KeyCode::Char('k'), &mut ctrl, &mut mode);
        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_toggle_task_on_enter() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("T1", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let initial_status = ctrl.state.todos[0].completed;

        EventHandler::handle_main_content(KeyCode::Enter, &mut ctrl, &mut mode);
        assert_ne!(ctrl.state.todos[0].completed, initial_status);
    }

    #[test]
    fn should_handle_edit_mode_transition() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("Edit Me", "Desc", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        EventHandler::handle_main_content(KeyCode::Char('e'), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Form);
        assert!(ctrl.ui.task_form.is_some());

        let form = ctrl.ui.task_form.as_ref().unwrap();
        assert_eq!(form.data().1, "Edit Me");
    }

    #[test]
    fn should_open_remove_confirm_dialog_on_key() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("To Delete", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        EventHandler::handle_main_content(KeyCode::Char('d'), &mut ctrl, &mut mode);

        assert!(ctrl.ui.modal.is_some());
        assert_eq!(ctrl.ui.modal.as_ref().unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_move_task_on_key() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("Task 1", "", None));
        state.todos.push(Todo::new("Task 2", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        EventHandler::handle_main_content(KeyCode::Char('J'), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.todos[1].title, "Task 1");
    }

    #[test]
    fn should_activate_search_on_key() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_main_content(KeyCode::Char('/'), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Search);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_scroll_through_description() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();

        ui.desc_scroll.current.set(10);
        ui.desc_scroll.max_scroll.set(20);
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_main_content(KeyCode::Char('['), &mut ctrl, &mut mode);
        assert_eq!(ctrl.ui.desc_scroll.current.get(), 9);

        EventHandler::handle_main_content(KeyCode::Char('['), &mut ctrl, &mut mode);
        assert_eq!(ctrl.ui.desc_scroll.current.get(), 8);

        EventHandler::handle_main_content(KeyCode::Char(']'), &mut ctrl, &mut mode);
        assert_eq!(ctrl.ui.desc_scroll.current.get(), 9);
    }

    #[test]
    fn should_fail_on_open_edit_if_not_selected() {
        let (mut state, mut ui, mut mode, mut config, _) = setup_ctx();
        state.select_state.select(None);

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        EventHandler::handle_main_content(KeyCode::Char('e'), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::List);
        assert!(ctrl.ui.task_form.is_none());
    }

    #[test]
    fn should_create_new_task_on_submit() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut mode = ApplicationMode::Form;

        let mut form = Form::new();
        form.set_value("title", "New Task");
        form.set_value("description", "Desc");
        form.focused = 3;
        ui.task_form = Some(form);

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        EventHandler::handle_form_mode(event, &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.todos.len(), 1);
        assert_eq!(ctrl.state.todos[0].title, "New Task");
        assert!(ctrl.ui.task_form.is_none());
        assert_eq!(mode, ApplicationMode::List,);
    }

    #[test]
    fn should_update_existing_task_on_submit() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut mode = ApplicationMode::Form;

        let task = Todo::new("Old Title", "", None);
        let task_id = task.id;
        state.todos.push(task);

        let mut form = Form::from(&state.todos[0]);
        form.set_value("title", "Updated Title");
        form.focused = 3;
        ui.task_form = Some(form);

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        EventHandler::handle_form_mode(event, &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.todos.len(), 1);
        assert_eq!(ctrl.state.todos[0].title, "Updated Title");
        assert_eq!(ctrl.state.todos[0].id, task_id);
    }

    #[test]
    fn should_close_form_on_cancel() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut mode = ApplicationMode::Form;
        ui.task_form = Some(Form::new());

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        EventHandler::handle_form_mode(event, &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.todos.len(), 0);
        assert!(ctrl.ui.task_form.is_none());
        assert_eq!(mode, ApplicationMode::List);
    }

    #[test]
    fn should_do_nothing_on_continue() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut mode = ApplicationMode::Form;
        ui.task_form = Some(Form::new());

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        EventHandler::handle_form_mode(event, &mut ctrl, &mut mode);

        assert!(ctrl.ui.task_form.is_some());
        assert_eq!(mode, ApplicationMode::Form);
    }

    #[test]
    fn should_exit_search_to_list_on_submit() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        EventHandler::handle_search_mode(event, &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::List);
        assert_eq!(ctrl.ui.focus_area, FocusArea::MainContent);
        assert!(ctrl.ui.search_input.is_some(),);
    }

    #[test]
    fn should_cancel_search_and_return_to_left_panel() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);

        EventHandler::handle_search_mode(event, &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Browsing);
        assert_eq!(ctrl.ui.focus_area, FocusArea::LeftPanel);
        assert!(ctrl.ui.search_input.is_none(),);
    }

    #[test]
    fn should_reset_selection_to_first_item_on_typing() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("Apple", "", None));
        state.todos.push(Todo::new("Banana", "", None));
        state.select_state.select(Some(1));

        ui.show_search();
        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        EventHandler::handle_search_mode(event, &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_handle_empty_input_gracefully() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        state.todos.clear();

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let event = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);

        EventHandler::handle_search_mode(event, &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_cycle_filter_down_up_on_key() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_left_panel(KeyCode::Char('j'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::Active);

        EventHandler::handle_left_panel(KeyCode::Down, &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::Completed);

        EventHandler::handle_left_panel(KeyCode::Char('k'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::Active);
    }

    #[test]
    fn should_select_filter_on_numeric_key() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_left_panel(KeyCode::Char('3'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::Completed);

        EventHandler::handle_left_panel(KeyCode::Char('5'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::Today);

        EventHandler::handle_left_panel(KeyCode::Char('1'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::All);
    }

    #[test]
    fn should_test_focus_stabilization_on_filter_change() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        state.todos.push(Todo::new("T", "", None));
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        EventHandler::handle_left_panel(KeyCode::Char('3'), &mut ctrl);

        assert!(
            ctrl.state.select_state.selected().unwrap_or(0) == 0,
            "Index should not be reset after filter change"
        );
    }

    #[test]
    fn should_ignore_unrelated_keys_in_left_panel() {
        let (mut state, mut ui, _, mut config, _) = setup_ctx();
        ui.current_filter = Filter::All;
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        EventHandler::handle_left_panel(KeyCode::Char('x'), &mut ctrl);
        assert_eq!(ctrl.ui.current_filter, Filter::All);
    }

    #[test]
    fn should_check_if_its_kill_process_key() {
        let key_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_kill_process_key(&key_ctrl_c));

        let key_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(!is_kill_process_key(&key_c));

        let key_other = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!is_kill_process_key(&key_other));
    }

    #[test]
    fn should_check_if_its_is_exit_key() {
        let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(is_exit_key(&key_q));

        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(is_exit_key(&key_esc));

        let key_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_exit_key(&key_ctrl_c));

        let key_other = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!is_exit_key(&key_other));
    }
}
