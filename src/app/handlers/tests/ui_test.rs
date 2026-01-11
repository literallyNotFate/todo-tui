// Unit-tests for ui handler
#[cfg(test)]
mod tests {
    use crate::app::{
        handlers::ui::{open_clear_confirm, open_edit_current, open_remove_confirm},
        state::state::ApplicationState,
        ui::{
            dialogs::dialog::DialogIntent, renderer::state::UIState,
            widgets::input::input::InputMode,
        },
    };

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

        assert!(ui_state.dialog.is_some(),);
        assert_eq!(ui_state.dialog.unwrap().intent, DialogIntent::Clear);
    }
}
