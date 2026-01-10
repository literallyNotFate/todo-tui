use crate::app::{
    application::Application,
    ui::{
        dialogs::dialog::DialogIntent,
        widgets::input::input::{Input, InputResult},
    },
};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

// Main key event handler for Application
pub fn handle_key_event(app: &mut Application, event: KeyEvent) {
    use crate::app::handlers::action::*;
    use ratatui::crossterm::event::KeyModifiers;

    // Exiting with Ctrl+C
    if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
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
            handle_dialog_result(&mut app.state, &mut app.ui, &result, &intent);
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

    handle_global_key(app, event.code);
}

// Handle global keys
pub fn handle_global_key(app: &mut Application, key: KeyCode) {
    use crate::app::{
        handlers::ui::{open_clear_confirm, open_edit_current, open_remove_confirm},
        ui::{components::components::Components, dialogs::dialog::DialogIntent},
    };

    match key {
        KeyCode::Char('q') | KeyCode::Esc => app.running = false,
        KeyCode::Char('k') | KeyCode::Up => app.state.select_state.select_previous(),
        KeyCode::Char('j') | KeyCode::Down => app.state.select_state.select_next(),
        KeyCode::Enter => app.state.toggle_current(),
        KeyCode::Char('a') => app.ui.show_input(Input::insert()),
        KeyCode::Char('r') => open_edit_current(&mut app.state, &mut app.ui),
        KeyCode::Char('d') => open_remove_confirm(&mut app.state, &mut app.ui),
        KeyCode::Char('x') => open_clear_confirm(&mut app.state, &mut app.ui),
        KeyCode::Char('?') => app
            .ui
            .show_dialog(Components::help_popup(), DialogIntent::None),
        _ => {}
    }
}
