// Unit-tests for action handler
#[cfg(test)]
mod tests {
    use crate::app::{
        handlers::action::{handle_dialog_result, handle_input_submit},
        state::state::ApplicationState,
        ui::{
            dialogs::dialog::{DialogIntent, DialogResult},
            renderer::state::UIState,
            widgets::input::input::InputMode,
        },
    };

    #[test]
    fn should_handle_dialog_result_cancelled() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &DialogResult::Cancelled,
            &DialogIntent::Remove,
        );

        assert!(ui_state.notification.is_none());
    }

    #[test]
    fn should_handle_dialog_result_confirmed_remove() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &DialogResult::Confirmed,
            &DialogIntent::Remove,
        );

        assert!(ui_state.notification.is_some(),);
    }

    #[test]
    fn should_handle_dialog_result_confirmed_clear() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
            &DialogResult::Confirmed,
            &DialogIntent::Clear,
        );

        assert!(ui_state.notification.is_some());
    }

    #[test]
    fn should_handle_dialog_result_none() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();

        handle_dialog_result(
            &mut app_state,
            &mut ui_state,
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

        assert!(ui_state.notification.is_some(),);
    }
}
