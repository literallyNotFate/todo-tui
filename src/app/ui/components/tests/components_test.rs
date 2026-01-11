// Unit-tests for components
#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::KeyCode;

    use crate::app::{
        ui::{
            components::components::Components,
            dialogs::{
                confirm::confirm::{Confirm, ConfirmOption},
                popup::popup::{Popup, PopupCloseBehavior, PopupKind},
            },
            renderer::state::Anchor,
        },
        utils::constants::{
            text::HELP_MESSAGE_TEXT,
            theme::{COLOR_ORANGE, COLOR_RED},
        },
    };

    #[test]
    fn should_return_help_popup() {
        let help_popup: Popup = Components::help_popup();

        assert_eq!(help_popup.message, HELP_MESSAGE_TEXT);
        assert_eq!(help_popup.kind, PopupKind::Help);
        assert_eq!(help_popup.title, Some(String::from("Controls")));
        assert_eq!(
            help_popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Char('?')),
        );
        assert_eq!(help_popup.anchor, Anchor::Center);
    }

    #[test]
    fn should_return_remove_confirm() {
        let task: String = String::from("Test");
        let remove_confirm: Confirm = Components::remove_todo_confirm(task.clone());

        assert_eq!(
            remove_confirm.message,
            format!("Are you sure to remove selected task ({})?", task),
        );
        assert_eq!(remove_confirm.select, ConfirmOption::Cancel);
        assert_eq!(remove_confirm.anchor, Anchor::Center);
        assert_eq!(remove_confirm.styles.border_color, COLOR_RED);

        let remove_confirm: Confirm = Components::remove_todo_confirm("".to_string());
        assert_eq!(
            remove_confirm.message,
            "Are you sure to remove selected task?"
        );
    }

    #[test]
    fn should_return_clear_confirm() {
        let count: usize = 3;
        let clear_confirm: Confirm = Components::clear_todos_confirm(count);

        assert_eq!(
            clear_confirm.message,
            format!("Are you sure to clear all tasks (todos count: {})?", count)
        );
        assert_eq!(clear_confirm.select, ConfirmOption::Cancel);
        assert_eq!(clear_confirm.anchor, Anchor::Center);
        assert_eq!(clear_confirm.styles.border_color, COLOR_ORANGE);
    }
}
