use crate::{
    Application,
    app::ApplicationController,
    config::KeyMaps,
    core::{Action, ApplicationMode, Autosave, FocusArea, Selectable},
    models::{Filter, TaskDetails, TaskEditor},
    ui::{
        Form, Popup, WidgetResponse, is_terminal_small,
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
        match app.mode {
            ApplicationMode::Form => {
                Self::handle_form_mode(event, &mut ctrl, &mut app.mode);
                return;
            }
            ApplicationMode::Search => {
                Self::handle_search_mode(event, &mut ctrl, &mut app.mode);
                return;
            }
            _ if ctrl.ui.modal.is_some() => {
                Self::handle_modal(event, &mut ctrl, &mut app.running);
                return;
            }
            _ => {}
        }

        if let Some(action) = app.keymaps.action(&event) {
            Self::execute_action(
                action,
                &mut ctrl,
                &mut app.mode,
                &mut app.autosave,
                &mut app.running,
            );
        }
    }

    /// Handle modal keys (confirm/popup)
    fn handle_modal(event: KeyEvent, ctrl: &mut ApplicationController, running: &mut bool) {
        let action: Option<Action> = ctrl.keymaps.action(&event);
        let result: Option<ModalResult> = {
            let modal_wrapper = match ctrl.ui.modal.as_mut() {
                Some(m) => m,
                None => return,
            };
            modal_wrapper.modal.handle_action(action, event.code)
        };

        if let Some(result) = result {
            let action = ctrl.ui.modal.as_ref().unwrap().action.clone();
            ctrl.ui.close_modal();

            if result == ModalResult::Confirmed {
                log::debug!("Modal confirmed: action={:?}", action);
                match action {
                    ModalAction::Remove => ctrl.dispatch_remove(),
                    ModalAction::Clear => ctrl.dispatch_clear(),
                    ModalAction::Save => {
                        ctrl.dispatch_save();
                    }
                    ModalAction::UnsavedExit => {
                        if ctrl.dispatch_save() {
                            *running = false;
                        }
                    }
                    _ => {}
                }
            } else {
                log::debug!("Modal cancelled: action={:?}", action);
            }
        }

        ctrl.ui.request_redraw();
    }

    /// Handles form keys
    fn handle_form_mode(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        mode: &mut ApplicationMode,
    ) {
        let form = match &mut ctrl.ui.task_form {
            Some(f) => f,
            None => return,
        };

        match form.handle_key(&event) {
            WidgetResponse::Submit => {
                let (id, title, desc, priority) = form.data();
                let editor: TaskEditor = TaskEditor {
                    title,
                    description: desc,
                    priority: Selectable::new(priority),
                };

                if let Some(task_id) = id {
                    ctrl.dispatch_update(task_id, editor);
                } else {
                    ctrl.dispatch_append(editor.title, editor.description, Some(*editor.priority));
                }

                ctrl.ui.task_form = None;
                *mode = ApplicationMode::List;
            }
            WidgetResponse::Cancel => {
                ctrl.ui.task_form = None;
                *mode = ApplicationMode::List;
            }
            _ => {}
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
                    ctrl.dispatch_save();
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
            Action::Add => {
                ctrl.ui.task_form = Some(Form::new());
                *mode = ApplicationMode::Form;
            }
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
                            ctrl.ui.task_form = Some(Form::from(&task));
                            *mode = ApplicationMode::Form;
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
                                        " Details ".into(),
                                        TaskDetails::from(task, &ctrl.config.ui),
                                    )
                                    .close_on(KeyCode::Tab)
                                    .with_scroll(ctrl.ui.desc_scroll.clone()),
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
        config::{Config, StorageConfig},
        core::{SortBy, SortOrder, Storage},
        models::Task,
        state::{ApplicationState, Session, UIState},
    };
    use ratatui::crossterm::event::KeyModifiers;
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    struct TestContext {
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
            Self {
                state: ApplicationState::default(),
                ui: UIState::default(),
                config: Config::default(),
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
            &mut ApplicationMode,
            &mut Autosave,
            &mut bool,
        ) {
            (
                &mut self.state,
                &mut self.ui,
                &mut self.config,
                &self.keymaps,
                &mut self.mode,
                &mut self.autosave,
                &mut self.running,
            )
        }
    }

    fn setup_application() -> Application {
        let mut app = Application::default();
        app.size = (100, 100);
        app
    }

    fn mock_unsaved_modal(
        event: KeyEvent,
        ctrl: &mut ApplicationController,
        running: &mut bool,
        path: &Path,
        config: &StorageConfig,
    ) {
        let action_intent = ctrl.keymaps.action(&event);
        let result = {
            let modal_wrapper = ctrl.ui.modal.as_mut().unwrap();
            modal_wrapper.modal.handle_action(action_intent, event.code)
        };

        if let Some(result) = result {
            let modal_action_type = ctrl.ui.modal.as_ref().unwrap().action.clone();
            ctrl.ui.close_modal();

            if result == ModalResult::Confirmed && modal_action_type == ModalAction::UnsavedExit {
                let current_id = ctrl.state.selected_id(
                    &ctrl.state.tasks,
                    &ctrl.ui.filter,
                    &ctrl.ui.search_query(),
                );
                let session = Session::from_state(ctrl.ui, current_id);

                match Storage::save(&ctrl.state.tasks, session, Some(path), config) {
                    Ok(string) => ctrl.ui.show_result_popup(Ok(string)),
                    Err(e) => ctrl.ui.show_result_popup(Err(e)),
                }

                *running = false;
            }
        }
    }

    #[test]
    fn should_block_input_when_terminal_is_too_small() {
        let mut app = setup_application();
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        EventHandler::handle_key(&mut app, event);

        assert_eq!(app.mode, ApplicationMode::Navigation);
        assert!(app.ui.task_form.is_none());
        assert!(app.running);
    }

    #[test]
    fn should_allow_exit_even_in_small_terminal() {
        let mut app = setup_application();
        app.size = (10, 5);

        let event = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        EventHandler::handle_key(&mut app, event);

        assert!(!app.running);
    }

    #[test]
    fn should_open_unsaved_confirm_on_exit_if_changes_exist() {
        let mut app = setup_application();
        app.data.tasks.push(Task::new("Changes", "", None));
        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('q')));

        assert!(app.running, "App should not close yet");
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_restore_mode_on_esc_from_form() {
        let mut app = setup_application();
        app.mode = ApplicationMode::Form;
        app.ui.task_form = Some(Form::new());

        EventHandler::handle_key(&mut app, key_event(KeyCode::Esc));

        assert_eq!(app.mode, ApplicationMode::List);
        assert!(app.ui.task_form.is_none());
        assert!(app.running);
    }

    #[test]
    fn should_prioritize_modal_over_global_keys() {
        let mut app = setup_application();
        app.ui.clear_confirm();

        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('a')));

        assert_eq!(app.mode, ApplicationMode::Navigation);
        assert!(app.ui.task_form.is_none());
        assert!(app.ui.modal.is_some());
    }

    #[test]
    fn should_trigger_save_on_ctrl_s() {
        let mut app = setup_application();
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        EventHandler::handle_key(&mut app, event);

        assert!(app.ui.modal.is_some(), "Save modal should appear");
    }

    #[test]
    fn should_handle_sort_keys() {
        let mut app = setup_application();
        app.ui.focused.value = FocusArea::Main;

        assert_eq!(app.data.sort.parameter, SortBy::Priority);
        assert_eq!(app.data.sort.order, SortOrder::Desc);

        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('s')));
        assert_eq!(app.data.sort.parameter, SortBy::Title);

        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('s')));
        assert_eq!(app.data.sort.parameter, SortBy::CreatedAt);

        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('r')));
        assert_eq!(app.data.sort.order, SortOrder::Asc);
    }

    #[test]
    fn should_toggle_autosave() {
        let mut app = setup_application();
        assert!(!app.autosave.enabled);

        let mut event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert!(app.autosave.enabled);

        event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT);
        EventHandler::handle_key(&mut app, event);
        assert!(!app.autosave.enabled);
    }

    #[test]
    fn should_toggle_focus_right_and_left() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::MoveRight, &mut ctrl, mode, autosave, running);

        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert_eq!(*mode, ApplicationMode::List);

        EventHandler::execute_action(Action::MoveLeft, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.focused, FocusArea::Sidebar);
        assert_eq!(*mode, ApplicationMode::Navigation);
    }

    #[test]
    fn should_test_delegation_to_left_panel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        ui.focused.set(FocusArea::Sidebar);
        ui.filter.set(Filter::All);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);

        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);
    }

    #[test]
    fn should_test_delegation_to_main_content() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("T1", "", None));
        state.tasks.push(Task::new("T2", "", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);
        *mode = ApplicationMode::List;

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_confirm_remove_task() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, running) = ctx.components();

        state.tasks.push(Task::new("To be deleted", "", None));
        state.select_state.select(Some(0));

        ui.remove_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Char('y')), &mut ctrl, running);

        assert_eq!(ctrl.state.tasks.len(), 0);
        assert!(ctrl.ui.modal.is_none());
        assert!(*running, "App should be running still");
    }

    #[test]
    fn should_cancel_clear_all_tasks() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, running) = ctx.components();
        state.tasks.push(Task::new("Keep me", "", None));

        ui.clear_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Esc), &mut ctrl, running);

        assert_eq!(ctrl.state.tasks.len(), 1);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_save_and_exit_on_unsaved_exit_confirm() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");

        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, running) = ctx.components();

        ui.unsaved_confirm();
        let cfg: StorageConfig = config.storage.clone();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        mock_unsaved_modal(
            key_event(KeyCode::Char('y')),
            &mut ctrl,
            running,
            &path,
            &cfg,
        );

        assert!(!(*running), "App should be closed");
    }

    #[test]
    fn should_not_exit_on_unsaved_exit_cancel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, running) = ctx.components();
        ui.unsaved_confirm();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Char('n')), &mut ctrl, running);

        assert!(*running, "App should not be closed if cancelled");
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_do_nothing_if_no_modal_exists() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, running) = ctx.components();
        ui.modal = None;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_modal(key_event(KeyCode::Enter), &mut ctrl, running);

        assert!(*running);
        assert!(ui.modal.is_none());
    }

    #[test]
    fn should_test_navigation_down_up() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("T1", "", None));
        state.tasks.push(Task::new("T2", "", None));
        state.tasks.push(Task::new("T3", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.state.select_state.selected(), Some(1));

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_toggle_task_on_enter() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        state.tasks.push(Task::new("T1", "", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        let initial_status = ctrl.state.tasks[0].completed;

        EventHandler::execute_action(Action::Complete, &mut ctrl, mode, autosave, running);
        assert_ne!(ctrl.state.tasks[0].completed, initial_status);
    }

    #[test]
    fn should_handle_update_mode_transition() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        state.tasks.push(Task::new("Edit Me", "Desc", None));
        state.select_state.select(Some(0));
        ui.focused.set(FocusArea::Main);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Update, &mut ctrl, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::Form);
        assert!(ctrl.ui.task_form.is_some());

        let form = ctrl.ui.task_form.as_ref().unwrap();
        assert_eq!(form.data().1, "Edit Me");
    }

    #[test]
    fn should_fail_on_open_edit_if_not_selected() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        state.select_state.select(None);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Update, &mut ctrl, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::List);
        assert!(ctrl.ui.task_form.is_none());
    }

    #[test]
    fn should_open_remove_confirm_dialog_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("To Delete", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Remove, &mut ctrl, mode, autosave, running);

        assert!(ctrl.ui.modal.is_some());
        assert_eq!(ctrl.ui.modal.as_ref().unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_move_task_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();

        ui.focused.set(FocusArea::Main);
        state.tasks.push(Task::new("Task 1", "", None));
        state.tasks.push(Task::new("Task 2", "", None));
        state.select_state.select(Some(0));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::MoveTaskDown, &mut ctrl, mode, autosave, running);

        assert_eq!(ctrl.state.tasks[1].title, "Task 1");
    }

    #[test]
    fn should_activate_search_on_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::Search, &mut ctrl, mode, autosave, running);

        assert_eq!(*mode, ApplicationMode::Search);
        assert!(ctrl.ui.search_input.is_some());
    }

    #[test]
    fn should_create_new_task_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        let mut mode = ApplicationMode::Form;

        let mut form = Form::new();
        form.set_value("title", "New Task");
        form.set_value("description", "Desc");
        form.focused = 3;
        ui.task_form = Some(form);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_form_mode(key_event(KeyCode::Enter), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.tasks.len(), 1);
        assert_eq!(ctrl.state.tasks[0].title, "New Task");
        assert!(ctrl.ui.task_form.is_none());
        assert_eq!(mode, ApplicationMode::List,);
    }

    #[test]
    fn should_update_existing_task_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        let mut mode = ApplicationMode::Form;

        let task = Task::new("Old Title", "", None);
        let task_id = task.id;
        state.tasks.push(task);

        let mut form = Form::from(&state.tasks[0]);
        form.set_value("title", "Updated Title");
        form.focused = 3;
        ui.task_form = Some(form);

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_form_mode(key_event(KeyCode::Enter), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.tasks.len(), 1);
        assert_eq!(ctrl.state.tasks[0].title, "Updated Title");
        assert_eq!(ctrl.state.tasks[0].id, task_id);
    }

    #[test]
    fn should_close_form_on_cancel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        let mut mode = ApplicationMode::Form;
        ui.task_form = Some(Form::new());

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_form_mode(key_event(KeyCode::Esc), &mut ctrl, &mut mode);

        assert_eq!(ctrl.state.tasks.len(), 0);
        assert!(ctrl.ui.task_form.is_none());
        assert_eq!(mode, ApplicationMode::List);
    }

    #[test]
    fn should_do_nothing_on_continue() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        let mut mode = ApplicationMode::Form;
        ui.task_form = Some(Form::new());

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_form_mode(key_event(KeyCode::Char('a')), &mut ctrl, &mut mode);

        assert!(ctrl.ui.task_form.is_some());
        assert_eq!(mode, ApplicationMode::Form);
    }

    #[test]
    fn should_exit_search_to_list_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Enter), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::List);
        assert_eq!(ctrl.ui.focused, FocusArea::Main);
        assert!(ctrl.ui.search_input.is_some(),);
    }

    #[test]
    fn should_cancel_search_and_return_to_left_panel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
        ui.show_search();

        let mut mode = ApplicationMode::Search;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::handle_search_mode(key_event(KeyCode::Esc), &mut ctrl, &mut mode);

        assert_eq!(mode, ApplicationMode::Navigation);
        assert_eq!(ctrl.ui.focused, FocusArea::Sidebar);
        assert!(ctrl.ui.search_input.is_none(),);
    }

    #[test]
    fn should_reset_selection_to_first_item_on_typing() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, _, _, _) = ctx.components();

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
        let (state, ui, config, keymaps, _, _, _) = ctx.components();
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
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);

        EventHandler::execute_action(Action::MoveDown, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(Action::MoveUp, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Active);
    }

    #[test]
    fn should_select_filter_on_numeric_key() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        EventHandler::execute_action(Action::FilterCompleted, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Completed);

        EventHandler::execute_action(Action::FilterToday, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::Today);

        EventHandler::execute_action(Action::FilterAll, &mut ctrl, mode, autosave, running);
        assert_eq!(ctrl.ui.filter, Filter::All);
    }

    #[test]
    fn should_test_focus_stabilization_on_filter_change() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, mode, autosave, running) = ctx.components();
        state.tasks.push(Task::new("T", "", None));
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        EventHandler::execute_action(Action::FilterCompleted, &mut ctrl, mode, autosave, running);

        assert!(
            ctrl.state.select_state.selected().unwrap_or(0) == 0,
            "Index should not be reset after filter change"
        );
    }

    #[test]
    fn should_ignore_unrelated_keys_in_left_panel() {
        let mut app = setup_application();
        app.ui.filter.set(Filter::All);
        EventHandler::handle_key(&mut app, key_event(KeyCode::Char('x')));
        assert_eq!(app.ui.filter, Filter::All);
    }
}
