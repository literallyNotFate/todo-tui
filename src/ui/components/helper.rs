use crate::{
    ui::{Confirm, Dialog, Popup, PopupKind},
    utils::constants::{
        text::HELP_MESSAGE_TEXT,
        theme::{COLOR_BLUE, COLOR_ORANGE, COLOR_RED},
    },
};
use ratatui::crossterm::event::KeyCode;

pub fn help_popup() -> Popup {
    Popup::new()
        .with_message(HELP_MESSAGE_TEXT)
        .kind(PopupKind::Help)
        .title("Controls")
        .close_on(KeyCode::Char('?'))
}

// Remove todo confirm dialog
pub fn remove_todo_confirm(data: String) -> Confirm {
    let message: String = if !data.is_empty() {
        format!("Are you sure to remove selected task ({})?", data)
    } else {
        "Are you sure to remove selected task?".to_string()
    };

    Confirm::new()
        .with_message(message)
        .with_border_color(COLOR_RED)
}

// Clear all todos in the list
pub fn clear_todos_confirm(data: usize) -> Confirm {
    Confirm::new()
        .with_message(format!(
            "Are you sure to clear all tasks (todos count: {})?",
            data
        ))
        .with_border_color(COLOR_ORANGE)
}

// Save all todos to json file
pub fn save_todos_confirm(data: usize) -> Confirm {
    Confirm::new()
        .with_message(format!(
            "Do you want to save tasks (todos count: {})?",
            data
        ))
        .with_border_color(COLOR_BLUE)
}

// Unit-tests for components
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state::Anchor,
        ui::{ConfirmOption, PopupCloseBehavior},
    };

    #[test]
    fn should_return_help_popup() {
        let help_popup: Popup = help_popup();

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
        let remove_confirm: Confirm = remove_todo_confirm(task.clone());

        assert_eq!(
            remove_confirm.message,
            format!("Are you sure to remove selected task ({})?", task),
        );
        assert_eq!(remove_confirm.select, ConfirmOption::Cancel);
        assert_eq!(remove_confirm.anchor, Anchor::Center);
        assert_eq!(remove_confirm.styles.border_color, COLOR_RED);

        let remove_confirm: Confirm = remove_todo_confirm("".to_string());
        assert_eq!(
            remove_confirm.message,
            "Are you sure to remove selected task?"
        );
    }

    #[test]
    fn should_return_clear_confirm() {
        let count: usize = 3;
        let clear_confirm: Confirm = clear_todos_confirm(count);

        assert_eq!(
            clear_confirm.message,
            format!("Are you sure to clear all tasks (todos count: {})?", count)
        );
        assert_eq!(clear_confirm.select, ConfirmOption::Cancel);
        assert_eq!(clear_confirm.anchor, Anchor::Center);
        assert_eq!(clear_confirm.styles.border_color, COLOR_ORANGE);
    }

    #[test]
    fn should_return_save_confirm() {
        let count: usize = 3;
        let save_confirm: Confirm = save_todos_confirm(count);

        assert_eq!(
            save_confirm.message,
            format!("Do you want to save tasks (todos count: {})?", count)
        );
        assert_eq!(save_confirm.select, ConfirmOption::Cancel);
        assert_eq!(save_confirm.anchor, Anchor::Center);
        assert_eq!(save_confirm.styles.border_color, COLOR_BLUE);
    }
}
