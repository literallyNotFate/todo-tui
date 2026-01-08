use crate::app::{
    ui::dialogs::{confirm::confirm::Confirm, dialog::Dialog, popup::popup::Popup},
    utils::constants::{text::HELP_MESSAGE_TEXT, theme::*},
};

pub struct Components;

impl Components {
    // Help popup (controls)
    pub fn help_popup() -> Popup {
        use crate::app::ui::dialogs::popup::popup::PopupKind;
        use ratatui::crossterm::event::KeyCode;

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
}
