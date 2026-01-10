use crate::app::{
    state::state::ApplicationState,
    ui::{
        components::components::Components, dialogs::dialog::DialogIntent, renderer::state::UIState,
    },
};

pub fn open_edit_current(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    use crate::app::ui::widgets::input::input::Input;

    let title: String = app_state
        .current_todo()
        .map(|t| t.title.clone())
        .unwrap_or_default();

    ui_state.show_input(Input::edit(title));
}

pub fn open_remove_confirm(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    let title: String = app_state
        .current_todo()
        .map(|t| t.title.clone())
        .unwrap_or_default();

    ui_state.show_dialog(Components::remove_todo_confirm(title), DialogIntent::Remove);
}

pub fn open_clear_confirm(app_state: &mut ApplicationState, ui_state: &mut UIState) {
    ui_state.show_dialog(
        Components::clear_todos_confirm(app_state.todos.len()),
        DialogIntent::Clear,
    );
}
