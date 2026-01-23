use crate::{
    state::{ApplicationState, UIState},
    traits::{ModalAction, ModalResult},
    ui::Confirm,
};

// Perform action based on modal result enum
pub fn handle_modal_result(
    app_state: &mut ApplicationState,
    ui_state: &mut UIState,
    running: &mut bool,
    result: &ModalResult,
    action: &ModalAction,
) {
    match result {
        ModalResult::Cancelled => {}
        ModalResult::Confirmed => match action {
            ModalAction::Remove => {
                let result =
                    app_state.remove(&ui_state.current_filter, app_state.select_state.selected());
                app_state.notify(result);
            }
            ModalAction::Clear => {
                let result = app_state.clear(&ui_state.current_filter);
                app_state.notify(result);
            }
            ModalAction::Save => ui_state.handle_save_with_popup(app_state.save()),
            ModalAction::UnsavedExit => handle_unsaved_exit(result, app_state, ui_state, running),
            ModalAction::None => {}
        },
    }
}

// Handle unsaved changes (confirm and save)
pub fn handle_unsaved_exit(
    result: &ModalResult,
    app_state: &mut ApplicationState,
    ui_state: &mut UIState,
    running: &mut bool,
) {
    match result {
        ModalResult::Confirmed => {
            ui_state.handle_save_with_popup(app_state.save());
            *running = false;
        }
        ModalResult::Cancelled => {
            *running = false;
        }
    }
}

pub fn open_remove_confirm(ui_state: &mut UIState) {
    ui_state.show_modal(
        Confirm::new("Are you sure to remove selected task?"),
        ModalAction::Remove,
    );
}

pub fn open_clear_confirm(ui_state: &mut UIState) {
    ui_state.show_modal(
        Confirm::new("Are you sure to clear all tasks?"),
        ModalAction::Clear,
    );
}

pub fn open_save_confirm(ui_state: &mut UIState) {
    ui_state.show_modal(
        Confirm::new("Do you want to save tasks?"),
        ModalAction::Save,
    );
}

pub fn open_unsaved_exit_confirm(ui_state: &mut UIState) {
    ui_state.show_modal(
        Confirm::new("You have unsaved changes. Save before exit?"),
        ModalAction::UnsavedExit,
    );
}

// Unit-tests for action handler
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_modal_result_cancelled() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_modal_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &ModalResult::Cancelled,
            &ModalAction::Remove,
        );

        assert!(app_state.notification.is_none());
    }

    #[test]
    fn should_handle_modal_result_confirmed_remove() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_modal_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &ModalResult::Confirmed,
            &ModalAction::Remove,
        );

        assert!(app_state.notification.is_some());
    }

    #[test]
    fn should_handle_modal_result_confirmed_clear() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_modal_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &ModalResult::Confirmed,
            &ModalAction::Clear,
        );

        assert!(app_state.notification.is_some());
    }

    #[test]
    fn should_handle_modal_result_confirmed_save() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_modal_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &ModalResult::Confirmed,
            &ModalAction::Save,
        );

        assert!(ui_state.modal.is_some());
    }

    #[test]
    fn should_handle_modal_result_none() {
        let mut app_state = ApplicationState::default();
        let mut ui_state = UIState::default();
        let mut running = true;

        handle_modal_result(
            &mut app_state,
            &mut ui_state,
            &mut running,
            &ModalResult::Confirmed,
            &ModalAction::None,
        );

        assert!(ui_state.modal.is_none());
    }

    #[test]
    fn should_open_remove_confirm() {
        let mut ui_state = UIState::default();

        open_remove_confirm(&mut ui_state);

        assert!(ui_state.modal.is_some());
        assert_eq!(ui_state.modal.unwrap().action, ModalAction::Remove);
    }

    #[test]
    fn should_open_clear_confirm() {
        let mut ui_state = UIState::default();

        open_clear_confirm(&mut ui_state);

        assert!(ui_state.modal.is_some());
        assert_eq!(ui_state.modal.unwrap().action, ModalAction::Clear);
    }

    #[test]
    fn should_open_save_confirm() {
        let mut ui_state = UIState::default();

        open_save_confirm(&mut ui_state);

        assert!(ui_state.modal.is_some());
        assert_eq!(ui_state.modal.unwrap().action, ModalAction::Save);
    }

    #[test]
    fn should_open_unsaved_confirm_exit() {
        let mut ui_state = UIState::default();

        open_unsaved_exit_confirm(&mut ui_state);

        assert!(ui_state.modal.is_some());
        assert_eq!(ui_state.modal.unwrap().action, ModalAction::UnsavedExit);
    }
}
