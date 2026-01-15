use crate::{
    state::{ApplicationState, UIState},
    ui::{DialogIntent, DialogResult, InputMode},
};

// Perform action based on dialog result enum
pub fn handle_dialog_result(
    app_state: &mut ApplicationState,
    ui_state: &mut UIState,
    running: &mut bool,
    result: &DialogResult,
    intent: &DialogIntent,
) {
    match result {
        DialogResult::Cancelled => {}
        DialogResult::Confirmed => match intent {
            DialogIntent::Remove => ui_state.notify(app_state.remove_todo()),
            DialogIntent::Clear => ui_state.notify(app_state.clear_todos()),
            DialogIntent::Save => ui_state.notify(app_state.save()),
            DialogIntent::UnsavedExit => handle_unsaved_exit(app_state, running),
            DialogIntent::None => {}
        },
    }
}

// Perform action after input submit (either edit or append)
pub fn handle_input_submit(
    app_state: &mut ApplicationState,
    ui_state: &mut UIState,
    mode: InputMode,
    text: String,
) {
    match mode {
        InputMode::Insert => ui_state.notify(app_state.append_todo(text)),
        InputMode::Edit => ui_state.notify(app_state.rename_todo(text)),
    }
}

// Handle unsaved changes (confirm and save)
pub fn handle_unsaved_exit(app_state: &mut ApplicationState, running: &mut bool) {
    let _ = app_state.save().unwrap_or_default();
    *running = false;
}

pub fn open_edit_current(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    use crate::ui::Input;

    let title: String = app_state
        .current_todo()
        .map(|t| t.title.clone())
        .unwrap_or_default();

    ui_state.show_input(Input::edit(title));
}

pub fn open_remove_confirm(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    use crate::ui::remove_todo_confirm;

    let title: String = app_state
        .current_todo()
        .map(|t| t.title.clone())
        .unwrap_or_default();

    ui_state.show_dialog(remove_todo_confirm(title), DialogIntent::Remove);
}

pub fn open_clear_confirm(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    use crate::ui::clear_todos_confirm;
    ui_state.show_dialog(
        clear_todos_confirm(app_state.todos.len()),
        DialogIntent::Clear,
    );
}

pub fn open_save_confirm(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    use crate::ui::save_todos_confirm;
    ui_state.show_dialog(
        save_todos_confirm(app_state.todos.len()),
        DialogIntent::Save,
    );
}

pub fn open_unsaved_exit_confirm(ui_state: &mut UIState) {
    use crate::ui::unsaved_exit_confirm;
    ui_state.show_dialog(unsaved_exit_confirm(), DialogIntent::UnsavedExit);
}

// Unit-tests for action handler
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_dialog_result_cancelled() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &DialogResult::Cancelled,
            &DialogIntent::Remove,
        );

        assert!(ui_state.notification.is_none());
    }

    #[test]
    fn should_handle_dialog_result_confirmed_remove() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &DialogResult::Confirmed,
            &DialogIntent::Remove,
        );

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_handle_dialog_result_confirmed_clear() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &DialogResult::Confirmed,
            &DialogIntent::Clear,
        );

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_handle_dialog_result_confirmed_save() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &DialogResult::Confirmed,
            &DialogIntent::Save,
        );

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_handle_dialog_result_none() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &DialogResult::Confirmed,
            &DialogIntent::None,
        );

        assert!(ui_state.notification.is_none());
    }

    #[test]
    fn should_handle_input_submit_insert() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let text = "New task".to_string();

        handle_input_submit(&mut app_state, &mut ui_state, InputMode::Insert, text);

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_handle_input_submit_edit() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let text: String = "Edited task".to_string();

        handle_input_submit(&mut app_state, &mut ui_state, InputMode::Edit, text);

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_open_edit_current() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        open_edit_current(&mut app_state, &mut ui_state);

        assert!(ui_state.input.is_some());
        assert_eq!(ui_state.input.unwrap().mode, InputMode::Edit);
    }

    #[test]
    fn should_open_remove_confirm() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        open_remove_confirm(&mut app_state, &mut ui_state);

        assert!(ui_state.dialog.is_some());
        assert_eq!(ui_state.dialog.unwrap().intent, DialogIntent::Remove);
    }

    #[test]
    fn should_open_clear_confirm() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        open_clear_confirm(&mut app_state, &mut ui_state);

        assert!(ui_state.dialog.is_some());
        assert_eq!(ui_state.dialog.unwrap().intent, DialogIntent::Clear);
    }

    #[test]
    fn should_open_save_confirm() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        open_save_confirm(&mut app_state, &mut ui_state);

        assert!(ui_state.dialog.is_some());
        assert_eq!(ui_state.dialog.unwrap().intent, DialogIntent::Save);
    }

    #[test]
    fn should_open_unsaved_confirm_exit() {
        let mut ui_state = UIState::default();

        open_unsaved_exit_confirm(&mut ui_state);

        assert!(ui_state.dialog.is_some());
        assert_eq!(ui_state.dialog.unwrap().intent, DialogIntent::UnsavedExit);
    }
}
