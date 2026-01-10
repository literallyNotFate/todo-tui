use crate::app::{
    state::state::ApplicationState,
    ui::{
        dialogs::dialog::{DialogIntent, DialogResult},
        renderer::state::UIState,
        widgets::input::input::InputMode,
    },
};

// Perform action based on dialog result enum
pub fn handle_dialog_result(
    app_state: &mut ApplicationState,
    ui_state: &mut UIState,
    result: &DialogResult,
    intent: &DialogIntent,
) {
    match result {
        DialogResult::Cancelled => {}
        DialogResult::Confirmed => match intent {
            DialogIntent::Remove => ui_state.notify(app_state.remove_todo()),
            DialogIntent::Clear => ui_state.notify(app_state.clear_todos()),
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
