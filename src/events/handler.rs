use crate::{
    Application,
    app::ApplicationController,
    config::KeyMaps,
    core::{Action, ApplicationMode, Autosave, FocusArea, Selectable, Storage},
    models::{Filter, TaskDetails, TaskEditor},
    ui::{
        Popup, WidgetResponse, is_terminal_small,
        widgets::{
            input::Input,
            modal::{ModalAction, ModalResult},
        },
    },
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Handling all possible keys
pub struct EventHandler;

impl EventHandler {
    /// Main key event handling
    pub fn handle_key(app: &mut Application, event: KeyEvent) {
        if KeyMaps::is_kill_process(&event) {
            app.running = false;
            return;
        }

        if is_terminal_small(app.size.0, app.size.1) {
            if app.keymaps.is(&event, Action::Quit) {
                app.running = false;
            }
            app.ui.request_redraw();
            return;
        }

        let mut ctrl =
            ApplicationController::new(&mut app.data, &mut app.ui, &mut app.config, &app.keymaps);

        if ctrl.ui.modal.is_some() {
            Self::handle_modal(event, &mut ctrl, &mut app.storage, &mut app.running);
            return;
        }

        match app.mode {
            ApplicationMode::Search => {
                Self::handle_search_mode(event, &mut ctrl, &mut app.mode);
                return;
            }
            _ => {}
        }

        if let Some(action) = app.keymaps.action(&event) {
            Self::execute_action(
                action,
                &mut ctrl,
                &mut app.storage,
                &mut app.mode,
                &mut app.autosave,
                &mut app.running,
            );
        }
    }

    /// Handle modal keys (confirm/popup)
    fn handle_modal(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        storage: &mut Storage,
        running: &mut bool,
    ) {
        let action: Option<Action> = ctrl.keymaps.action(&event);
        let result: Option<ModalResult> = {
            let modal_wrapper = match ctrl.ui.modal.as_mut() {
                Some(m) => m,
                None => return,
            };
            modal_wrapper.modal.handle_action(action, &event)
        };

        if let Some(result) = result {
            let modal_action = ctrl.ui.modal.as_ref().unwrap().action.clone();

            match result {
                ModalResult::TaskSubmitted {
                    id,
                    title,
                    description,
                    priority,
                } => {
                    let editor = TaskEditor {
                        title,
                        description,
                        priority: Selectable::new(priority),
                    };

                    if let Some(task_id) = id {
                        ctrl.dispatch_update(task_id, editor);
                    } else {
                        ctrl.dispatch_append(
                            editor.title,
                            editor.description,
                            Some(*editor.priority),
                        );
                    }
                }

                ModalResult::Confirmed => {
                    log::debug!("Modal confirmed: action={:?}", modal_action);
                    match modal_action {
                        ModalAction::Remove => ctrl.dispatch_remove(),
                        ModalAction::Clear => ctrl.dispatch_clear(),
                        ModalAction::Save => {
                            ctrl.dispatch_save(storage);
                        }
                        ModalAction::UnsavedExit => {
                            if ctrl.dispatch_save(storage) {
                                *running = false;
                            }
                        }
                        _ => {}
                    }
                }

                ModalResult::Cancelled => {
                    log::debug!("Modal cancelled: action={:?}", modal_action);
                }
            }

            ctrl.ui.close_modal();
        }

        ctrl.ui.request_redraw();
    }

    /// Handles search keys
    fn handle_search_mode(
        event: KeyEvent,
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
        }
        ctrl.ui.request_redraw();
    }

    /// Helper function to execute pressed button action
    fn execute_action(
        action: Action,
        ctrl: &mut ApplicationController,
        storage: &mut Storage,
        mode: &mut ApplicationMode,
        autosave: &mut Autosave,
        running: &mut bool,
    ) {
        ctrl.ui.request_redraw();
        let focus: FocusArea = *ctrl.ui.focused;

        match action {
            Action::Quit => {
                if ctrl.state.any_unsaved_changes() {
                    ctrl.ui.unsaved_confirm();
                } else {
                    *running = false;
                }
            }
            Action::Save => {
                if ctrl.config.behavior.confirm_before_save {
                    ctrl.ui.save_confirm();
                } else {
                    ctrl.dispatch_save(storage);
                }
            }
            Action::ToggleAutosave => autosave.toggle_enabled(),
            Action::ShowHelp => {
                let help_lines = ctrl.keymaps.hotkeys_info(&ctrl.ui.theme.palette());
                ctrl.ui.show_modal(
                    Popup::help(help_lines).with_scroll(ctrl.ui.hotkeys_scroll.clone()),
                    ModalAction::None,
                );
            }
            Action::ToggleSidebar => ctrl.ui.toggle_sidebar(),
            Action::NextTheme => ctrl.ui.next_theme(),
            Action::PrevTheme => ctrl.ui.prev_theme(),
            Action::ToggleThemeMode => ctrl.ui.toggle_mode(),
            Action::Add => ctrl.ui.show_modal(Popup::new_task(), ModalAction::None),
            Action::Search => {
                ctrl.ui.show_search();
                *mode = ApplicationMode::Search;
            }

            Action::MoveLeft => {
                ctrl.ui.focused.set(FocusArea::Sidebar);
                *mode = ApplicationMode::Navigation;
            }
            Action::MoveRight => {
                ctrl.ui.focused.set(FocusArea::Main);
                *mode = ApplicationMode::List;
            }
            Action::MoveUp => match *ctrl.ui.focused {
                FocusArea::Sidebar => {
                    ctrl.ui.prev_tab_filter();
                    ctrl.stabilize(None);
                }
                FocusArea::Main => {
                    ctrl.dispatch_move_selection(-1);
                }
            },
            Action::MoveDown => match *ctrl.ui.focused {
                FocusArea::Sidebar => {
                    ctrl.ui.next_tab_filter();
                    ctrl.stabilize(None);
                }
                FocusArea::Main => {
                    ctrl.dispatch_move_selection(1);
                }
            },

            // For filters
            Action::FilterAll
            | Action::FilterActive
            | Action::FilterCompleted
            | Action::FilterHigh
            | Action::FilterToday
                if focus == FocusArea::Sidebar =>
            {
                match action {
                    Action::FilterAll => ctrl.ui.change_filter(Filter::All),
                    Action::FilterActive => ctrl.ui.change_filter(Filter::Active),
                    Action::FilterCompleted => ctrl.ui.change_filter(Filter::Completed),
                    Action::FilterHigh => ctrl.ui.change_filter(Filter::HighPriority),
                    Action::FilterToday => ctrl.ui.change_filter(Filter::Today),
                    _ => {}
                }
                ctrl.stabilize(None);
            }

            // For main content
            Action::Update
            | Action::Remove
            | Action::Complete
            | Action::Details
            | Action::Sort
            | Action::SortReverse
            | Action::ClearAll
                if focus == FocusArea::Main =>
            {
                match action {
                    Action::Update => {
                        if let Some(task) = ctrl
                            .ui
                            .selected_id(ctrl.state)
                            .and_then(|id| ctrl.state.find_by_id(id))
                        {
                            ctrl.ui
                                .show_modal(Popup::update_task(&task), ModalAction::None)
                        }
                    }
                    Action::Complete => ctrl.dispatch_toggle(),
                    Action::Remove => {
                        if ctrl.config.behavior.confirm_before_remove {
                            ctrl.ui.remove_confirm();
                        } else {
                            ctrl.dispatch_remove();
                        }
                    }
                    Action::Sort => {
                        ctrl.state.sort.parameter.next();
                        ctrl.dispatch_sorting();
                    }
                    Action::SortReverse => {
                        ctrl.state.sort.order.next();
                        ctrl.dispatch_sorting();
                    }
                    Action::Details => {
                        if let Some(id) = ctrl.ui.selected_id(ctrl.state) {
                            if let Some(task) = ctrl.state.find_by_id(id) {
                                log::debug!("Opening task details popup");
                                ctrl.ui.show_modal(
                                    Popup::details(
                                        " Details ",
                                        TaskDetails::from(task, &ctrl.config.ui),
                                    )
                                    .with_scroll(ctrl.ui.desc_scroll.clone())
                                    .close_on(KeyCode::Tab),
                                    ModalAction::None,
                                );
                            }
                        }
                    }
                    Action::ClearAll => ctrl.ui.clear_confirm(),
                    _ => {}
                }
            }
            Action::MoveTaskDown if focus == FocusArea::Main => ctrl.dispatch_move_tasks(1),
            Action::MoveTaskUp if focus == FocusArea::Main => ctrl.dispatch_move_tasks(-1),

            _ => log::trace!("Action {:?} ignored in focus {:?}", action, focus),
        }
    }
}

/// Unit-tests for event handler
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::Config,
        core::{SortBy, SortOrder, Storage},
        models::Task,
        state::{ApplicationState, UIState},
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;
    use tempdir::TempDir;

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    struct TestContext {
        _temp_dir: TempDir,
        storage: Storage,
        state: ApplicationState,
        ui: UIState,
        config: Config,
        keymaps: KeyMaps,
        mode: ApplicationMode,
        autosave: Autosave,
        running: bool,
    }

    impl TestContext {
        fn new() -> Self {
            let temp_dir: TempDir = TempDir::new("task_event_test").unwrap();
            let db_path: PathBuf = temp_dir.path().join("test_tasks.db");
            let config: Config = Config::default();
            let storage: Storage = Storage::init(Some(&db_path), &config.storage).unwrap();

            Self {
                _temp_dir: temp_dir,
                storage,
                state: ApplicationState::default(),
                ui: UIState::default(),
                config,
                keymaps: KeyMaps::default(),
                mode: ApplicationMode::List,
                autosave: Autosave::new(false),
                running: true,
            }
        }

        pub fn components(
            &mut self,
        ) -> (
            &mut ApplicationState,
            &mut UIState,
            &mut Config,
            &KeyMaps,
            &mut Storage,
            &mut ApplicationMode,
            &mut Autosave,
            &mut bool,
        ) {
            (
                &mut self.state,
                &mut self.ui,
                &mut self.config,
                &self.keymaps,
                &mut self.storage,
                &mut self.mode,
                &mut self.autosave,
                &mut self.running,
            )
        }
    }

    struct AppTestWrapper {
        _temp_dir: TempDir,
        pub app: Application,
    }

    fn setup_application() -> AppTestWrapper {
        let mut app = Application::default();
        app.size = (100, 100);

        let temp_dir: TempDir = TempDir::new("app_struct_test").unwrap();
        let db_path: PathBuf = temp_dir.path().join("app.db");
        app.storage = Storage::init(Some(&db_path), &app.config.storage).unwrap();

        AppTestWrapper {
            _temp_dir: temp_dir,
            app,
        }
    }

    #[test]
    fn should_block_input_when_terminal_is_too_small() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        EventHandler::handle_key(app, event);

        assert_eq!(app.mode, ApplicationMode::Navigation);
        assert!(app.ui.modal.is_none());
        assert!(app.running);
    }

    #[test]
    fn should_allow_exit_even_in_small_terminal() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        EventHandler::handle_key(app, event);

        assert!(!app.running);
    }

    #[test]
    fn should_open_unsaved_confirm_on_exit_if_changes_exist() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.data.tasks.push(Task::new("Changes", "", None));
        EventHandler::handle_key(app, key_event(KeyCode::Char('q')));

        assert!(app.running, "App should not close yet");
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_prioritize_modal_over_global_keys() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.ui.clear_confirm();

        EventHandler::handle_key(app, key_event(KeyCode::Char('a')));

        assert_eq!(app.mode, ApplicationMode::Navigation);
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_trigger_save_on_ctrl_s() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        EventHandler::handle_key(app, event);

        assert!(app.ui.modal.is_some(), "Save modal should appear");
    }

    #[test]
    fn should_handle_sort_keys() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.ui.focused.value = FocusArea::Main;

        assert_eq!(app.data.sort.parameter, SortBy::Priority);
        assert_eq!(app.data.sort.order, SortOrder::Desc);

        EventHandler::handle_key(app, key_event(KeyCode::Char('s')));
        assert_eq!(app.data.sort.parameter, SortBy::Title);

        EventHandler::handle_key(app, key_event(KeyCode::Char('s')));
        assert_eq!(app.data.sort.parameter, SortBy::CreatedAt);

        EventHandler::handle_key(app, key_event(KeyCode::Char('r')));
        assert_eq!(app.data.sort.order, SortOrder::Asc);
    }

    #[test]
    fn should_toggle_autosave() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        assert!(!app.autosave.enabled);

        let mut event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(app, event);
        assert!(app.autosave.enabled);

        event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(app, event);
        assert!(!app.autosave.enabled);
    }

    #[test]
    fn should_toggle_focus_right_and_left() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::MoveRight,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert_eq!(*mode, ApplicationMode::List);

        EventHandler::execute_action(
            Action::MoveLeft,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.focused, FocusArea::Sidebar);
        assert_eq!(*mode, ApplicationMode::Navigation);
    }

    #[test]
    fn should_test_delegation_to_left_panel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        ui.focused.set(FocusArea::Sidebar);
        ui.filter.set(Filter::All);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Active);

        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, storage, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);
    }

    #[test]
    fn should_test_delegation_to_main_content() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("T1", "", None));
        state.tasks.push(Task::new("T2", "", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);
        *mode = ApplicationMode::List;

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_confirm_remove_task() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();

        state.tasks.push(Task::new("To be deleted", "", None));
        state.select_state.select(Some(0));

        ui.remove_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Char('y')), &mut ctrl, storage, running);

        assert_eq!(ctrl.state.tasks.len(), 0);
        assert!(ctrl.ui.modal.is_none());
        assert!(*running, "App should be running still");
    }

    #[test]
    fn should_cancel_clear_all_tasks() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();
        state.tasks.push(Task::new("Keep me", "", None));

        ui.clear_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Esc), &mut ctrl, storage, running);

        assert_eq!(ctrl.state.tasks.len(), 1);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_save_and_exit_on_unsaved_exit_confirm() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();

        state.tasks.push(Task::new("Task to DB", "", None));
        ui.unsaved_confirm();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Char('y')), &mut ctrl, storage, running);

        assert!(!(*running), "App should be closed");
    }

    #[test]
    fn should_not_exit_on_unsaved_exit_cancel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();
        ui.unsaved_confirm();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Char('n')), &mut ctrl, storage, running);

        assert!(*running, "App should not be closed if cancelled");
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_do_nothing_if_no_modal_exists() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();
        ui.modal = None;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Enter), &mut ctrl, storage, running);

        assert!(*running);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_test_navigation_down_up() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("T1", "", None));
        state.tasks.push(Task::new("T2", "", None));
        state.tasks.push(Task::new("T3", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.state.select_state.selected(), Some(1));

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, storage, mode, autosave, running);
        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_toggle_task_on_enter() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        state.tasks.push(Task::new("T1", "", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        let initial_status = ctrl.state.tasks[0].completed;

        EventHandler::execute_action(
            Action::Complete,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_ne!(ctrl.state.tasks[0].completed, initial_status);
    }

    #[test]
    fn should_handle_update_mode_transition() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("Edit Me", "Desc", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Update, &mut ctrl, storage, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::List);
        assert!(ctrl.ui.modal.is_some());
    }

    #[test]
    fn should_fail_on_open_edit_if_not_selected() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        state.select_state.select(None);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Update, &mut ctrl, storage, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::List);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_open_remove_confirm_dialog_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("To Delete", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Remove, &mut ctrl, storage, mode, autosave, running);

        assert!(ctrl.ui.modal.is_some());
        assert_eq!(ctrl.ui.modal.as_ref().unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_move_task_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("Task 1", "", None));
        state.tasks.push(Task::new("Task 2", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::MoveTaskDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert_eq!(ctrl.state.tasks[1].title, "Task 1");
    }

    #[test]
    fn should_activate_search_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Search, &mut ctrl, storage, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::Search);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_create_new_task_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, _, _, running) = ctx.components();

        ui.show_modal(Popup::append_task(), ModalAction::None);
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        EventHandler::handle_modal(key_event(KeyCode::Char('G')), &mut ctrl, storage, running);
        EventHandler::handle_modal(key_event(KeyCode::Char('o')), &mut ctrl, storage, running);

        EventHandler::handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);
        EventHandler::handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);

        EventHandler::handle_modal(key_event(KeyCode::Char('T')), &mut ctrl, storage, running);
        EventHandler::handle_modal(key_event(KeyCode::Char('U')), &mut ctrl, storage, running);
        EventHandler::handle_modal(key_event(KeyCode::Char('I')), &mut ctrl, storage, running);

        EventHandler::handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);
        EventHandler::handle_modal(key_event(KeyCode::Enter), &mut ctrl, storage, running);

        assert!(ctrl.ui.modal.is_none());
        assert_eq!(ctrl.state.tasks.len(), 1);
        assert_eq!(ctrl.state.tasks[0].title, "Go");
        assert_eq!(ctrl.state.tasks[0].description, "TUI");
    }

    #[test]
    fn should_exit_search_to_list_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _, _) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Enter), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::List);
        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_cancel_search_and_return_to_left_panel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _, _) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Esc), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Navigation);
        assert_eq!(ctrl.ui.focused, FocusArea::Sidebar);
        assert!(ctrl.ui.search_input.is_none());
    }

    #[test]
    fn should_reset_selection_to_first_item_on_typing() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _, _) = ctx.components();

        state.tasks.push(Task::new("Apple", "", None));
        state.tasks.push(Task::new("Banana", "", None));
        state.select_state.select(Some(1));
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Char('a')), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_handle_empty_input_gracefully() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _, _) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        state.tasks.clear();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Char('x')), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_cycle_filter_down_up_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Active);

        EventHandler::execute_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, storage, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);
    }

    #[test]
    fn should_select_filter_on_numeric_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        EventHandler::execute_action(
            Action::FilterCompleted,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(
            Action::FilterToday,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::Today);

        EventHandler::execute_action(
            Action::FilterAll,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.filter, Filter::All);
    }

    #[test]
    fn should_test_focus_stabilization_on_filter_change() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        state.tasks.push(Task::new("T", "", None));
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(
            Action::FilterCompleted,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert!(
            ctrl.state.select_state.selected().unwrap_or(0) == 0,
            "Index should not be reset after filter change"
        );
    }

    #[test]
    fn should_ignore_unrelated_keys_in_left_panel() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.ui.filter.set(Filter::All);
        EventHandler::handle_key(app, key_event(KeyCode::Char('x')));
        assert_eq!(app.ui.filter, Filter::All);
    }
}
