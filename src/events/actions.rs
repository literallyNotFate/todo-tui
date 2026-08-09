use crate::{
    app::ApplicationController,
    core::{Action, ApplicationMode, Autosave, FocusArea, Storage},
    models::TaskDetails,
    state::SidebarTab,
    ui::{Popup, widgets::modal::ModalAction},
};

/// Handle actions
pub fn handle_action(
    action: Action,
    ctrl: &mut ApplicationController,
    storage: &mut Storage,
    mode: &mut ApplicationMode,
    autosave: &mut Autosave,
    running: &mut bool,
) {
    ctrl.ui.request_redraw();
    let focus: FocusArea = *ctrl.ui.focused;
    let folders: Vec<uuid::Uuid> = ctrl.state.get_folders();

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
        Action::SwitchTheme => ctrl.ui.show_modal(
            Popup::switch_theme(ctrl.ui.theme.theme_id()),
            ModalAction::None,
        ),
        Action::ToggleThemeMode => ctrl.ui.toggle_mode(),
        Action::AddTask => ctrl.ui.show_modal(Popup::append_task(), ModalAction::None),
        Action::AddFolder => ctrl
            .ui
            .show_modal(Popup::append_folder(), ModalAction::None),
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
                ctrl.ui.prev_tab_filter(&folders);
                ctrl.stabilize(None);
            }
            FocusArea::Main => {
                ctrl.dispatch_move_selection(-1);
            }
        },
        Action::MoveDown => match *ctrl.ui.focused {
            FocusArea::Sidebar => {
                ctrl.ui.next_tab_filter(&folders);
                ctrl.stabilize(None);
            }
            FocusArea::Main => {
                ctrl.dispatch_move_selection(1);
            }
        },

        // UI
        Action::ToggleSidebar => ctrl.ui.toggle_sidebar(),
        Action::ToggleFooter => ctrl.ui.toggle_footer(),
        Action::IncreaseSidebar => ctrl.ui.increase_sidebar(),
        Action::DecreaseSidebar => ctrl.ui.decrease_sidebar(),
        Action::ResetUI => ctrl.ui.reset_ui(),

        // For filters
        Action::FilterInbox
        | Action::FilterActive
        | Action::FilterCompleted
        | Action::FilterHigh
        | Action::FilterToday => {
            match action {
                Action::FilterInbox => ctrl.ui.change_filter(SidebarTab::Inbox, None),
                Action::FilterActive => ctrl.ui.change_filter(SidebarTab::Active, None),
                Action::FilterCompleted => ctrl.ui.change_filter(SidebarTab::Completed, None),
                Action::FilterHigh => ctrl.ui.change_filter(SidebarTab::HighPriority, None),
                Action::FilterToday => ctrl.ui.change_filter(SidebarTab::Today, None),
                _ => {}
            }

            ctrl.stabilize(None);
        }

        // For sidebar
        Action::RemoveFolder | Action::UpdateFolder if focus == FocusArea::Sidebar => {
            match action {
                Action::RemoveFolder => {
                    if let Some(folder_id) = ctrl.ui.active_folder {
                        if ctrl.config.behavior.confirm_before_remove {
                            ctrl.ui.remove_folder_confirm(folder_id);
                        } else {
                            ctrl.dispatch_remove_folder(folder_id);
                        }
                    }
                }
                Action::UpdateFolder => {
                    if let Some(folder_id) = ctrl.ui.active_folder {
                        if let Some(folder) = ctrl.state.find_folder_by_id(folder_id) {
                            ctrl.ui
                                .show_modal(Popup::update_folder(&folder), ModalAction::None);
                        }
                    }
                }
                _ => {}
            }
        }

        // For main content
        Action::UpdateTask
        | Action::RemoveTask
        | Action::Complete
        | Action::Pin
        | Action::ShowDetails
        | Action::Sort
        | Action::SortReverse
        | Action::Clear
            if focus == FocusArea::Main =>
        {
            match action {
                Action::UpdateTask => {
                    if let Some(task) = ctrl
                        .ui
                        .selected_id(ctrl.state, &ctrl.config.behavior)
                        .and_then(|id| ctrl.state.find_task_by_id(id))
                    {
                        ctrl.ui
                            .show_modal(Popup::update_task(&task), ModalAction::None)
                    }
                }
                Action::Complete => ctrl.dispatch_completed(),
                Action::Pin => ctrl.dispatch_pinned(),
                Action::RemoveTask => {
                    if ctrl.config.behavior.confirm_before_remove {
                        ctrl.ui.remove_task_confirm();
                    } else {
                        ctrl.dispatch_remove_task();
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
                Action::ShowDetails => {
                    if let Some(id) = ctrl.ui.selected_id(ctrl.state, &ctrl.config.behavior) {
                        if let Some(task) = ctrl.state.find_task_by_id(id) {
                            log::debug!("Opening task details popup");
                            ctrl.ui.show_modal(
                                Popup::details(
                                    " Details ",
                                    TaskDetails::from(task, &ctrl.config.ui),
                                )
                                .with_scroll(ctrl.ui.desc_scroll.clone())
                                .close_on(ratatui::crossterm::event::KeyCode::Tab),
                                ModalAction::None,
                            );
                        }
                    }
                }
                Action::Clear => ctrl.ui.clear_confirm(),
                _ => {}
            }
        }
        Action::MoveTaskDown if focus == FocusArea::Main => ctrl.dispatch_move_tasks(1),
        Action::MoveTaskUp if focus == FocusArea::Main => ctrl.dispatch_move_tasks(-1),

        _ => log::trace!("Action {:?} ignored in focus {:?}", action, focus),
    }
}

/// Unit-tests for actions handler
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Application,
        config::{Config, KeyMaps},
        core::{SortBy, SortOrder, Storage},
        events::EventHandler,
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
            let temp_dir = TempDir::new("task_event_test").unwrap();
            let db_path = temp_dir.path().join("test_tasks.db");
            let config = Config::default();
            let storage = Storage::init(Some(&db_path), &config.storage).unwrap();

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

        let temp_dir = TempDir::new("app_struct_test").unwrap();
        let db_path = temp_dir.path().join("app.db");
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
        app.data.tasks.push(Task::new("Changes"));
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
        app.ui.focused.set(FocusArea::Main);

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
        handle_action(
            Action::MoveRight,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert_eq!(*mode, ApplicationMode::List);

        handle_action(
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
        ui.active_tab = SidebarTab::Inbox;

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Active);

        handle_action(
            Action::MoveDown,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Completed);

        self::handle_action(Action::MoveUp, &mut ctrl, storage, mode, autosave, running);
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Active);
    }

    #[test]
    fn should_test_delegation_to_main_content() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("T1"));
        state.tasks.push(Task::new("T2"));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);
        *mode = ApplicationMode::List;

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
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
    fn should_handle_update_mode_transition() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("Edit Me"));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
            Action::UpdateTask,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert_eq!(*mode, ApplicationMode::List);
        assert!(ctrl.ui.modal.is_some());
    }

    #[test]
    fn should_fail_on_open_edit_if_not_selected() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        state.select_state.select(None);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
            Action::UpdateTask,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert_eq!(*mode, ApplicationMode::List);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_open_remove_confirm_dialog_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("To Delete"));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
            Action::RemoveTask,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );

        assert!(ctrl.ui.modal.is_some());
        assert_eq!(ctrl.ui.modal.as_ref().unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_move_task_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("Task 1"));
        state.tasks.push(Task::new("Task 2"));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
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
        handle_action(Action::Search, &mut ctrl, storage, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::Search);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_select_filter_on_numeric_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        handle_action(
            Action::FilterCompleted,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Completed);

        handle_action(
            Action::FilterToday,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Today);

        handle_action(
            Action::FilterInbox,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert_eq!(ctrl.ui.active_tab, SidebarTab::Inbox);
    }

    #[test]
    fn should_test_focus_stabilization_on_filter_change() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, mode, autosave, running) = ctx.components();
        state.tasks.push(Task::new("T"));
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_action(
            Action::FilterCompleted,
            &mut ctrl,
            storage,
            mode,
            autosave,
            running,
        );
        assert!(ctrl.state.select_state.selected().unwrap_or(0) == 0);
    }

    #[test]
    fn should_ignore_unrelated_keys_in_left_panel() {
        let mut wrapper = setup_application();
        let app = &mut wrapper.app;
        app.ui.focused.set(FocusArea::Sidebar);
        app.ui.active_tab = SidebarTab::Inbox;

        EventHandler::handle_key(app, key_event(KeyCode::Char('x')));
        assert_eq!(app.ui.active_tab, SidebarTab::Inbox);
    }
}
