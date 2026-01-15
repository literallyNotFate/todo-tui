use super::{
    handle_dialog_result, handle_input_submit, open_clear_confirm, open_edit_current,
    open_remove_confirm, open_save_confirm, open_unsaved_exit_confirm,
};
use crate::{
    app::Application,
    state::{ApplicationState, UIState},
    ui::{DialogIntent, Input, InputResult, help_popup},
    utils::constants::terminal::is_terminal_small,
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
    if app.ui.dialog.is_some() {
        let result = {
            let dialog = app.ui.dialog.as_mut().unwrap();
            dialog.modal.handle_key(event.code)
        };

        if let Some(result) = result {
            let intent: DialogIntent = app.ui.dialog.as_ref().unwrap().intent.clone();
            handle_dialog_result(
                &mut app.state,
                &mut app.ui,
                &mut app.running,
                &result,
                &intent,
            );
            app.ui.close_dialog();
            return;
        }
    }

    // Input handling
    if app.ui.input.is_some() {
        let result = {
            let input = app.ui.input.as_mut().unwrap();
            input.handle_key(event.code)
        };

        match result {
            InputResult::Continue => return,
            InputResult::Cancel => {
                app.ui.close_input();
                return;
            }
            InputResult::Submit(text) => {
                let mode = app.ui.input.as_ref().unwrap().mode;
                handle_input_submit(&mut app.state, &mut app.ui, mode, text);
                app.ui.close_input();
                return;
            }
        }
    }

    handle_global_key(app, &event);
}

// Handle global keys
pub fn handle_global_key(app: &mut Application, key_event: &KeyEvent) {
    let code: KeyCode = key_event.code;
    let mode: KeyModifiers = key_event.modifiers;

    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            handle_close(&mut app.state, &mut app.ui, &mut app.running)
        }
        KeyCode::Char('k') | KeyCode::Up => app.state.select_state.select_previous(),
        KeyCode::Char('j') | KeyCode::Down => app.state.select_state.select_next(),
        KeyCode::Enter => app.state.toggle_current(),
        KeyCode::Char('a') => app.ui.show_input(Input::insert()),
        KeyCode::Char('r') => open_edit_current(&mut app.state, &mut app.ui),
        KeyCode::Char('d') => open_remove_confirm(&mut app.state, &mut app.ui),
        KeyCode::Char('x') => open_clear_confirm(&mut app.state, &mut app.ui),
        KeyCode::Char('?') => app.ui.show_dialog(help_popup(), DialogIntent::None),
        KeyCode::Char('s') if mode.contains(KeyModifiers::CONTROL) => {
            open_save_confirm(&mut app.state, &mut app.ui)
        }
        _ => {}
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
        state::{ActiveDialog, ApplicationResult, ApplicationState, UIState},
        ui::{Dialog, Popup},
    };
    use std::{
        fs::{self, File},
        hash::{DefaultHasher, Hash, Hasher},
        io::BufWriter,
        path::{Path, PathBuf},
    };
    use tempdir::TempDir;

    // Mock application structure
    struct MockApplication {
        running: bool,
        state: ApplicationState,
        ui: UIState,
    }

    impl MockApplication {
        pub fn new() -> Self {
            Self {
                running: true,
                state: ApplicationState::default(),
                ui: UIState::default(),
            }
        }

        pub fn set_n_todos(&mut self, n: u8) {
            for i in 0..=n {
                self.state.append_todo(format!("Task {}", i + 1)).unwrap();
            }
        }

        pub fn save(&mut self, path: &Path) {
            fs::create_dir_all(path.parent().unwrap()).unwrap();

            let file: File = File::create(path).unwrap();
            let writer: BufWriter<File> = BufWriter::new(file);

            serde_json::to_writer_pretty(writer, &self.state.todos).unwrap();
            let mut hasher = DefaultHasher::new();
            self.state.todos.hash(&mut hasher);
            self.state.saved_todos_hash = hasher.finish();
        }
    }

    // Macro to test handle_global_key() function
    macro_rules! mock_handle_global_key {
        (&mut $app:expr, $key_event:expr) => {
            match $key_event.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    handle_close(&mut $app.state, &mut $app.ui, &mut $app.running)
                }
                KeyCode::Char('k') | KeyCode::Up => $app.state.select_state.select_previous(),
                KeyCode::Char('j') | KeyCode::Down => $app.state.select_state.select_next(),
                KeyCode::Enter => $app.state.toggle_current(),
                KeyCode::Char('a') => $app.ui.show_input(Input::insert()),
                KeyCode::Char('r') => open_edit_current(&mut $app.state, &mut $app.ui),
                KeyCode::Char('d') => open_remove_confirm(&mut $app.state, &mut $app.ui),
                KeyCode::Char('x') => open_clear_confirm(&mut $app.state, &mut $app.ui),
                KeyCode::Char('?') => $app.ui.show_dialog(help_popup(), DialogIntent::None),
                KeyCode::Char('s') if $key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    open_save_confirm(&mut $app.state, &mut $app.ui)
                }
                _ => {}
            }
        };
    }

    // Macro to test handle_key_event() function
    macro_rules! mock_handle_key_event {
        (&mut $app:expr, $event:expr, $terminal_size:expr) => {{
            let is_small = is_terminal_small($terminal_size.0, $terminal_size.1);

            if is_small {
                if is_exit_key(&$event) {
                    $app.running = false;
                }
            } else {
                if is_kill_process_key(&$event) {
                    $app.running = false;
                } else if $app.ui.dialog.is_some() {
                    let result = {
                        let dialog = $app.ui.dialog.as_mut().unwrap();
                        dialog.modal.handle_key($event.code)
                    };
                    if let Some(result) = result {
                        let intent = $app.ui.dialog.as_ref().unwrap().intent.clone();
                        handle_dialog_result(
                            &mut $app.state,
                            &mut $app.ui,
                            &mut $app.running,
                            &result,
                            &intent,
                        );
                        $app.ui.close_dialog();
                    }
                } else if $app.ui.input.is_some() {
                    let result = {
                        let input = $app.ui.input.as_mut().unwrap();
                        input.handle_key($event.code)
                    };
                    match result {
                        InputResult::Continue => {}
                        InputResult::Cancel => {
                            $app.ui.close_input();
                        }
                        InputResult::Submit(text) => {
                            let mode = $app.ui.input.as_ref().unwrap().mode;
                            handle_input_submit(&mut $app.state, &mut $app.ui, mode, text);
                            $app.ui.close_input();
                        }
                    }
                } else {
                    mock_handle_global_key!(&mut $app, $event);
                }
            }
        }};
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
        let mut app = MockApplication::new();
        app.state.append_todo("Test task").unwrap();

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(app.running, "Running should NOT be false yet");
        assert!(
            app.ui.dialog.is_some(),
            "Unsaved exit confirm dialog should be opened"
        );
        assert_eq!(
            app.ui.dialog.as_ref().unwrap().intent,
            DialogIntent::UnsavedExit,
            "Intent should be UnsavedExit"
        );
    }

    #[test]
    fn should_handle_close_when_no_changes_made() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = MockApplication::new();
        app.save(&path);
        assert!(!app.state.any_unsaved_changes());

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(!app.running, "Running should be set to false");
        assert!(app.ui.dialog.is_none(), "No dialog should be opened");
    }

    #[test]
    fn should_handle_close_when_save_made() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = MockApplication::new();
        app.state.append_todo("Test task").unwrap();
        assert!(app.state.any_unsaved_changes());

        app.save(&path);

        handle_close(&mut app.state, &mut app.ui, &mut app.running);

        assert!(!app.running, "Running should be false");
        assert!(app.ui.dialog.is_none(), "No dialog after save");
    }

    #[test]
    fn should_handle_global_key_navigation() {
        let mut app = MockApplication::new();
        app.set_n_todos(3);

        assert_eq!(app.state.select_state.selected(), Some(3));

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)
        );
        assert_eq!(app.state.select_state.selected(), Some(2));

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)
        );
        assert_eq!(app.state.select_state.selected(), Some(3));
    }

    #[test]
    fn should_handle_global_key_actions() {
        let mut app = MockApplication::new();
        app.set_n_todos(3);

        mock_handle_global_key!(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(app.state.todos[3].done, "Should be toggled");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)
        );
        assert!(app.ui.input.is_some(), "Should show input on 'a'");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)
        );
        assert!(app.ui.input.is_some(), "Should open edit on 'r'");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE)
        );
        assert!(app.ui.dialog.is_some(), "Should open remove confirm on 'd'");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)
        );
        assert!(app.ui.dialog.is_some(), "Should open clear confirm on 'x'");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE)
        );
        assert!(app.ui.dialog.is_some(), "Should show help on '?'");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)
        );
        assert!(app.ui.dialog.is_some(), "Should open save confirm");

        mock_handle_global_key!(
            &mut app,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)
        );
        assert!(
            app.ui.dialog.is_some(),
            "Should open unsaved changes confirm"
        );
    }

    #[test]
    fn should_handle_key_event_small_terminal_exit() {
        let mut app = MockApplication::new();
        let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let terminal_size = (30, 10);

        mock_handle_key_event!(&mut app, key_q, terminal_size);
        assert!(!app.running, "Should exit on 'q' even in small terminal");
    }

    #[test]
    fn should_handle_key_event_small_terminal_ignore_other() {
        let mut app = MockApplication::new();
        let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let terminal_size = (30, 10);

        mock_handle_key_event!(&mut app, key_enter, terminal_size);
        assert!(app.running, "Should ignore non-exit keys in small terminal");
    }

    #[test]
    fn should_handle_key_event_dialog_priority() {
        let mut app = MockApplication::new();
        app.ui.dialog = Some(ActiveDialog {
            modal: Box::new(Popup::new()),
            intent: DialogIntent::None,
        });

        let key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let terminal_size = (80, 24);

        mock_handle_key_event!(&mut app, key, terminal_size);
        assert!(
            app.ui.dialog.is_none(),
            "Dialog should be closed after handling"
        );
    }

    #[test]
    fn should_handle_key_event_input_priority() {
        let mut app = MockApplication::new();
        app.ui.input = Some(Input::insert());

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let terminal_size = (80, 24);

        mock_handle_key_event!(&mut app, key, terminal_size);
        assert!(
            app.ui.input.is_none(),
            "Input should be closed after submit"
        );
    }

    #[test]
    fn should_handle_key_event_global_fallback() {
        let mut app = MockApplication::new();

        let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let terminal_size = (80, 24);

        mock_handle_key_event!(&mut app, key_a, terminal_size);
        assert!(app.ui.input.is_some(), "Should handle global 'a' key");
    }
}
