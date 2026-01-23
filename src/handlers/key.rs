use super::{handle_modal_result, open_save_confirm, open_unsaved_exit_confirm};
use crate::{
    app::Application,
    enums::{ApplicationMode, FocusArea},
    handlers::{action::open_remove_confirm, open_clear_confirm},
    models::Filter,
    state::{ApplicationState, UIState},
    traits::ModalAction,
    ui::{Form, Notification, WidgetResponse, is_terminal_small},
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Main key event handler for Application
pub fn handle_key_event(app: &mut Application, event: KeyEvent, terminal_size: (u16, u16)) {
    // Dont block exit keys while fallback active
    if is_terminal_small(terminal_size.0, terminal_size.1) {
        if is_exit_key(&event) {
            app.running = false;
        }

        return;
    }

    // Exiting with Ctrl+C
    if is_kill_process_key(&event) {
        app.running = false;
        return;
    }

    // Dialog handling
    if app.ui.modal.is_some() {
        let result = {
            let dialog = app.ui.modal.as_mut().unwrap();
            dialog.modal.handle_key(event.code)
        };

        if let Some(result) = result {
            let intent: ModalAction = app.ui.modal.as_ref().unwrap().action.clone();
            app.ui.close_modal();

            handle_modal_result(
                &mut app.state,
                &mut app.ui,
                &mut app.running,
                &result,
                &intent,
            );

            app.sync_ui();
        }

        return;
    }

    handle_global_key(app, &event);
}

// Handle global keys
pub fn handle_global_key(app: &mut Application, event: &KeyEvent) {
    let code: KeyCode = event.code;
    let mode: KeyModifiers = event.modifiers;

    match code {
        KeyCode::Char('s') if mode == KeyModifiers::CONTROL => {
            open_save_confirm(&mut app.ui);
            return;
        }
        KeyCode::Esc => {
            if app.mode == ApplicationMode::Task {
                app.ui.task_form = None;
                app.mode = ApplicationMode::Browsing;
            } else {
                handle_close(&mut app.state, &mut app.ui, &mut app.running);
            }

            return;
        }
        _ => {}
    }

    match app.mode {
        ApplicationMode::Browsing => handle_browsing_keys(app, &event.code),
        ApplicationMode::Task => handle_form_keys(app, &event.code),
    }
}

// Handle browsing mode keys
pub fn handle_browsing_keys(app: &mut Application, code: &KeyCode) {
    match app.ui.focus_area {
        FocusArea::LeftPanel => match code {
            KeyCode::Char('j') | KeyCode::Down => app.ui.next_tab_filter(),
            KeyCode::Char('k') | KeyCode::Up => app.ui.prev_tab_filter(),
            KeyCode::Char('1') => app.ui.change_filter(Filter::All),
            KeyCode::Char('2') => app.ui.change_filter(Filter::Active),
            KeyCode::Char('3') => app.ui.change_filter(Filter::Completed),
            KeyCode::Char('4') => app.ui.change_filter(Filter::HighPriority),
            _ => {}
        },

        FocusArea::MainContent => match code {
            KeyCode::Char('j') | KeyCode::Down => app.state.next_task(),
            KeyCode::Char('k') | KeyCode::Up => app.state.prev_task(),
            KeyCode::Enter => app
                .state
                .toggle(&app.ui.current_filter, app.state.select_state.selected()),
            KeyCode::Char('d') => open_remove_confirm(&mut app.ui),
            KeyCode::Char('e') => {
                if let Some(ui_index) = app.state.select_state.selected() {
                    if let Some((_, task)) = app
                        .state
                        .filtered_stream(&app.ui.current_filter)
                        .nth(ui_index)
                    {
                        app.ui.task_form = Some(Form::from(task));
                        app.mode = ApplicationMode::Task;
                    }
                }
            }
            _ => {}
        },
    }

    match code {
        KeyCode::Char('q') => handle_close(&mut app.state, &mut app.ui, &mut app.running),
        KeyCode::Char('h') | KeyCode::Char('l') => app.ui.toggle_focus(),
        KeyCode::Char('a') => {
            app.ui.task_form = Some(Form::new());
            app.mode = ApplicationMode::Task;
        }
        KeyCode::Char('x') => open_clear_confirm(&mut app.ui),
        _ => {}
    }

    app.sync_ui();
}

// Handle task form keys
pub fn handle_form_keys(app: &mut Application, code: &KeyCode) {
    if let Some(form) = &mut app.ui.task_form {
        let response: WidgetResponse = form.handle_key(code);
        match response {
            WidgetResponse::Continue => return,
            WidgetResponse::Submit => {
                let result = form.apply(&mut app.state);
                let is_ok: bool = result.is_ok();

                match result {
                    Ok(msg) => app.state.notification = Some(Notification::success(msg)),
                    Err(e) => app.state.notification = Some(Notification::error(e.to_string())),
                }

                if is_ok {
                    app.ui.task_form = None;
                    app.mode = ApplicationMode::Browsing;
                } else {
                    return;
                }
            }
            WidgetResponse::Cancel => {
                app.ui.task_form = None;
                app.mode = ApplicationMode::Browsing;
            }
        }
    }
}

// Handle application closing (with unsaved changes)
pub fn handle_close(app_state: &mut ApplicationState, ui_state: &mut UIState, running: &mut bool) {
    if app_state.any_unsaved_changes() {
        open_unsaved_exit_confirm(ui_state);
    } else {
        *running = false;
    }
}

// If its kill process key
pub fn is_kill_process_key(key_event: &KeyEvent) -> bool {
    key_event.code == KeyCode::Char('c') && key_event.modifiers.contains(KeyModifiers::CONTROL)
}

pub fn is_exit_key(key_event: &KeyEvent) -> bool {
    matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc)
        || (key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL))
}

// Unit-tests for key event handlers
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::Todo,
        state::{ApplicationResult, ApplicationState},
        ui::{FieldType, Popup},
    };
    use std::{
        fs::{self, File},
        hash::{DefaultHasher, Hash, Hasher},
        io::BufWriter,
        path::{Path, PathBuf},
    };
    use tempdir::TempDir;

    // Setup application
    fn setup_app() -> Application {
        Application::test()
    }

    // Setup some todos
    fn setup_with_n_todos(n: usize) -> ApplicationState {
        let mut state: ApplicationState = ApplicationState::default();
        for i in 1..=n {
            let todo: Todo = Todo::new(format!("Task {}", i), "Description", None);
            let _: ApplicationResult<String> = state.append(todo);
        }

        state
    }

    // Mock save method to save in temp directory
    fn save(app_state: &mut ApplicationState, path: &Path) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let file: File = File::create(path).unwrap();
        let writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &app_state.todos).unwrap();
        let mut hasher = DefaultHasher::new();
        app_state.todos.hash(&mut hasher);
        app_state.saved_todos_hash = hasher.finish();
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

    #[test]
    fn should_handle_close_open_confirm_when_unsaved_changes() {
        let mut app = setup_app();
        app.state.append(Todo::new("Test", "Test", None)).unwrap();

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(app.running, "Running should NOT be false yet");
        assert!(
            app.ui.modal.is_some(),
            "Unsaved exit confirm dialog should be opened"
        );
        assert_eq!(
            app.ui.modal.as_ref().unwrap().action,
            ModalAction::UnsavedExit,
            "Action should be UnsavedExit"
        );
    }

    #[test]
    fn should_handle_close_when_no_changes_made() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = setup_app();
        save(&mut app.state, &path);
        assert!(!app.state.any_unsaved_changes());

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(!app.running, "Running should be set to false");
        assert!(app.ui.modal.is_none(), "No modal should be opened");
    }

    #[test]
    fn should_handle_close_when_save_made() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = setup_app();
        app.state.append(Todo::new("Test", "Test", None)).unwrap();
        assert!(app.state.any_unsaved_changes());

        save(&mut app.state, &path);

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(!app.running, "Running should be false");
        assert!(app.ui.modal.is_none(), "No modal shown after save");
    }

    #[test]
    fn should_toggle_focus_area() {
        let mut app = setup_app();
        app.ui.focus_area = FocusArea::LeftPanel;

        let key = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
        handle_global_key(&mut app, &key);
        assert_eq!(app.ui.focus_area, FocusArea::MainContent);

        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        handle_global_key(&mut app, &key);
        assert_eq!(app.ui.focus_area, FocusArea::LeftPanel);
    }

    #[test]
    fn should_test_esc_logic_in_different_modes() {
        let mut app = setup_app();

        app.mode = ApplicationMode::Task;
        app.ui.task_form = Some(crate::ui::Form::new());
        let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        handle_global_key(&mut app, &esc);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(app.ui.task_form.is_none());

        app.mode = ApplicationMode::Browsing;
        handle_global_key(&mut app, &esc);
        assert!(app.ui.modal.is_some(), "Unsaved dialog must be active");
    }

    #[test]
    fn should_handle_global_key_actions() {
        let mut app = setup_app();
        app.state = setup_with_n_todos(5);

        assert_eq!(app.ui.current_filter, Filter::All);
        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert_eq!(app.ui.focus_area, FocusArea::LeftPanel);

        handle_global_key(
            &mut app,
            &KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        );
        assert_eq!(app.ui.current_filter, Filter::Active);

        handle_global_key(&mut app, &KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.ui.current_filter, Filter::Completed);
        assert!(
            app.state
                .filtered_stream(&Filter::Completed)
                .next()
                .is_none(),
            "Filter 'Completed' must contain no todos"
        );

        handle_global_key(&mut app, &KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.ui.current_filter, Filter::HighPriority);
        assert!(
            app.state
                .filtered_stream(&Filter::Completed)
                .next()
                .is_none(),
            "Filter 'HighPriority' must contain no todos"
        );

        handle_global_key(&mut app, &KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.ui.current_filter, Filter::All);

        handle_global_key(
            &mut app,
            &KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
        );
        assert_eq!(app.ui.focus_area, FocusArea::MainContent);
        assert_eq!(app.state.select_state.selected(), Some(0));

        handle_global_key(
            &mut app,
            &KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        );
        assert_eq!(app.state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_test_edit_and_create_task_keys() {
        let mut app = setup_app();

        app.state.append(Todo::new("Task 1", "Desc", None)).unwrap();
        app.state.append(Todo::new("Task 2", "Desc", None)).unwrap();
        app.state.append(Todo::new("Task 3", "Desc", None)).unwrap();

        app.ui.focus_area = FocusArea::MainContent;
        app.ui.current_filter = Filter::All;
        app.state.select_state.select(Some(1));

        handle_global_key(
            &mut app,
            &KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
        );
        assert_eq!(app.mode, ApplicationMode::Task);
        assert!(app.ui.task_form.is_some(), "Form must be create");

        if let Some(form) = &app.ui.task_form {
            assert_eq!(form.fields[0].name, "title", "First field must be title");

            if let FieldType::Text { input } = &form.fields[0].field_type {
                assert_eq!(
                    input.buffer, "Task 2",
                    "First field buffer must contain title value"
                );
            };
        }

        app.mode = ApplicationMode::Browsing;
        app.ui.task_form = None;

        handle_global_key(
            &mut app,
            &KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
        );

        assert_eq!(app.mode, ApplicationMode::Task);
        assert!(app.ui.task_form.is_some());

        if let Some(form) = &app.ui.task_form {
            assert_eq!(form.fields[0].name, "title", "First field must be title");

            if let FieldType::Text { input } = &form.fields[0].field_type {
                assert_eq!(input.buffer, "", "First field must be empty (new form)");
            };
        }
    }

    #[test]
    fn should_handle_key_event_small_terminal_exit() {
        let mut app = setup_app();
        let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let terminal_size = (30, 10);

        handle_key_event(&mut app, key_q, terminal_size);
        assert!(!app.running, "Should exit on 'q' even in small terminal");
    }

    #[test]
    fn should_handle_key_event_small_terminal_ignore_other() {
        let mut app = setup_app();
        let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let terminal_size = (30, 10);

        handle_key_event(&mut app, key_enter, terminal_size);
        assert!(app.running, "Should ignore non-exit keys in small terminal");
    }

    #[test]
    fn should_handle_key_event_dialog_priority() {
        let mut app = setup_app();
        app.ui.show_modal(Popup::info("test"), ModalAction::None);

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let terminal_size = (80, 24);

        handle_key_event(&mut app, key, terminal_size);
        assert!(
            app.ui.modal.is_none(),
            "Modal should be closed after handling"
        );
    }
}
