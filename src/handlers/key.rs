use super::{handle_modal_result, open_save_confirm, open_unsaved_exit_confirm};
use crate::{
    app::Application,
    enums::{ApplicationMode, FocusArea, WidgetResponse},
    handlers::{action::open_remove_confirm, open_clear_confirm},
    models::Filter,
    state::{ApplicationState, UIState},
    traits::{Input, ModalAction},
    ui::{Form, Notification, is_terminal_small},
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
            match app.mode {
                ApplicationMode::Search => app.ui.search_input = None,
                ApplicationMode::Form => app.ui.task_form = None,
                _ => {
                    handle_close(&mut app.state, &mut app.ui, &mut app.running);
                    return;
                }
            }

            app.restore_base_mode();
            return;
        }
        _ => {}
    }

    match app.mode {
        ApplicationMode::Browsing | ApplicationMode::List => {
            handle_browsing_and_list_keys(app, &code)
        }
        ApplicationMode::Form => handle_form_keys(app, event),
        ApplicationMode::Search => handle_search_keys(app, &code),
    }
}

// Handle browsing and list modes keys
pub fn handle_browsing_and_list_keys(app: &mut Application, code: &KeyCode) {
    match app.ui.focus_area {
        FocusArea::LeftPanel => match code {
            KeyCode::Char('j') | KeyCode::Down => app.ui.next_tab_filter(),
            KeyCode::Char('k') | KeyCode::Up => app.ui.prev_tab_filter(),
            KeyCode::Char('1') => app.ui.change_filter(Filter::All),
            KeyCode::Char('2') => app.ui.change_filter(Filter::Active),
            KeyCode::Char('3') => app.ui.change_filter(Filter::Completed),
            KeyCode::Char('4') => app.ui.change_filter(Filter::HighPriority),
            KeyCode::Char('5') => app.ui.change_filter(Filter::Today),
            _ => {}
        },

        FocusArea::MainContent => match code {
            KeyCode::Char('j') | KeyCode::Down => app.state.next_task(),
            KeyCode::Char('k') | KeyCode::Up => app.state.prev_task(),
            KeyCode::Char('[') => app.state.scroll.scroll_up(),
            KeyCode::Char(']') => app.state.scroll.scroll_down(),
            KeyCode::Enter => handle_toggle(&mut app.state, &app.ui),
            KeyCode::Char('J') => app.state.move_task_down(),
            KeyCode::Char('K') => app.state.move_task_up(),
            KeyCode::Char('d') => open_remove_confirm(&mut app.ui),
            KeyCode::Char('e') => handle_update(&app.state, &mut app.ui, &mut app.mode),
            KeyCode::Char('/') => handle_search(&mut app.ui, &mut app.mode),
            _ => {}
        },
    }

    match code {
        KeyCode::Char('q') => handle_close(&mut app.state, &mut app.ui, &mut app.running),
        KeyCode::Char('h') | KeyCode::Char('l') => handle_focus(&mut app.ui, &mut app.mode),
        KeyCode::Char('t') => app.ui.switch_theme(),
        KeyCode::Char('a') => handle_append(&mut app.ui, &mut app.mode),
        KeyCode::Char('x') => open_clear_confirm(&mut app.ui),
        _ => {}
    }

    app.sync_ui();
}

// Handle task form keys
pub fn handle_form_keys(app: &mut Application, event: &KeyEvent) {
    if let Some(form) = &mut app.ui.task_form {
        let response: WidgetResponse = form.handle_key(event);
        match response {
            WidgetResponse::Continue => (),
            WidgetResponse::Submit => {
                let result = form.apply(&mut app.state);
                let is_ok: bool = result.is_ok();

                match result {
                    Ok(msg) => app.state.notification = Some(Notification::success(msg)),
                    Err(e) => app.state.notification = Some(Notification::error(e.to_string())),
                }

                if is_ok {
                    app.ui.task_form = None;
                    app.restore_base_mode();
                }
            }
            WidgetResponse::Cancel => {
                app.ui.task_form = None;
                app.restore_base_mode();
            }
        }
    }
}

// Handle search keys
pub fn handle_search_keys(app: &mut Application, code: &KeyCode) {
    if let Some(input) = app.ui.search_input.as_mut() {
        match input.handle_key(&code) {
            WidgetResponse::Submit => app.mode = ApplicationMode::Browsing,
            WidgetResponse::Cancel => {
                app.ui.search_input = None;
                app.restore_base_mode();
            }
            WidgetResponse::Continue => {}
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

// Handle creating new form (on append)
pub fn handle_append(ui_state: &mut UIState, mode: &mut ApplicationMode) {
    ui_state.task_form = Some(Form::new());
    *mode = ApplicationMode::Form;
}

// Handle search tasks by title
pub fn handle_search(ui_state: &mut UIState, mode: &mut ApplicationMode) {
    ui_state.show_search();
    *mode = ApplicationMode::Search;
}

// Handle toggle completed
pub fn handle_toggle(app_state: &mut ApplicationState, ui_state: &UIState) {
    app_state.toggle(&ui_state.current_filter, app_state.select_state.selected())
}

// Handle focus
pub fn handle_focus(ui_state: &mut UIState, mode: &mut ApplicationMode) {
    ui_state.toggle_focus();
    *mode = match ui_state.focus_area {
        FocusArea::LeftPanel => ApplicationMode::Browsing,
        FocusArea::MainContent => ApplicationMode::List,
    };
}

// Handle updating existing form
pub fn handle_update(
    app_state: &ApplicationState,
    ui_state: &mut UIState,
    mode: &mut ApplicationMode,
) {
    if let Some(ui_index) = app_state.select_state.selected() {
        if let Some((_, task)) = app_state
            .filtered_stream(&ui_state.current_filter)
            .nth(ui_index)
        {
            ui_state.task_form = Some(Form::from(task));
            *mode = ApplicationMode::Form;
        }
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
        state::{AdaptiveScroll, ApplicationResult, ApplicationState},
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
    fn setup_app() -> Application<'static> {
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

        app.mode = ApplicationMode::Form;
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
        assert_eq!(app.ui.current_filter, Filter::Today);

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
    fn should_enter_search_mode_on_slash() {
        let mut app = setup_app();
        app.ui.focus_area = FocusArea::MainContent;

        let key = KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE);
        handle_global_key(&mut app, &key);

        assert_eq!(app.mode, ApplicationMode::Search);
        assert!(
            app.ui.search_input.is_some(),
            "Search input should be initialized"
        );
    }

    #[test]
    fn should_update_search_query_and_filter_sidebar_counts() {
        let mut app = setup_app();
        app.state
            .append(Todo::new("Buy Milk", "Desc", None))
            .unwrap();
        app.state
            .append(Todo::new("Code Rust", "Desc", None))
            .unwrap();

        handle_search(&mut app.ui, &mut app.mode);

        let keys = vec!['m', 'i', 'l', 'k'];
        for c in keys {
            let key_event = KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
            handle_search_keys(&mut app, &key_event.code);
        }

        let query = app.ui.search_input.as_ref().unwrap().buffer.as_str();
        assert_eq!(query, "milk");

        let filtered_count = Filter::All.count(&app.state.todos, query);
        assert_eq!(filtered_count, 1, "Only one task should match 'milk'");
    }

    #[test]
    fn should_exit_search_mode_on_submit_but_keep_query() {
        let mut app = setup_app();
        handle_search(&mut app.ui, &mut app.mode);

        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        handle_search_keys(&mut app, &key_a.code);

        let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        handle_search_keys(&mut app, &key_enter.code);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(
            app.ui.search_input.is_some(),
            "Search query should persist after Submit"
        );
        assert_eq!(app.ui.search_input.as_ref().unwrap().buffer, "a");
    }

    #[test]
    fn should_clear_search_and_exit_on_cancel() {
        let mut app = setup_app();
        handle_search(&mut app.ui, &mut app.mode);

        let key_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        handle_search_keys(&mut app, &key_x.code);

        let key_esc = KeyCode::Esc;
        handle_search_keys(&mut app, &key_esc);

        assert_eq!(app.mode, ApplicationMode::Browsing);
        assert!(
            app.ui.search_input.is_none(),
            "Search input should be cleared on Cancel"
        );
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
        assert_eq!(app.mode, ApplicationMode::Form);
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

        assert_eq!(app.mode, ApplicationMode::Form);
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

    #[test]
    fn should_handle_description_scroll_keys() {
        let mut scroll: AdaptiveScroll = AdaptiveScroll {
            current: 0,
            max_scroll: 5,
        };

        let keys = vec![KeyCode::Char(']'), KeyCode::Char(']')];

        for key in keys {
            match key {
                KeyCode::Char('[') => scroll.scroll_up(),
                KeyCode::Char(']') => scroll.scroll_down(),
                _ => {}
            }
        }

        assert_eq!(scroll.current, 2);
    }
}
