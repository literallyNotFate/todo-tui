use ratatui::style::Color;

use crate::app::ui::dialogs::{confirm::confirm::Confirm, dialog::Dialog, popup::popup::Popup};

pub struct Components;

impl Components {
    // Help popup (controls)
    pub fn help_popup() -> Popup {
        use crate::app::ui::dialogs::popup::popup::PopupKind;
        use ratatui::crossterm::event::KeyCode;

        let help_message: Vec<&str> = vec![
            " a -> append a todo",
            " r -> rename a todo",
            " d -> delete a todo",
            " Enter -> mark as completed",
            " k/Up -> go up",
            " j/Down -> go down",
            " q/Esc -> quit",
            " ? -> toggle help",
        ];

        Popup::new()
            .with_message(help_message.join("\n"))
            .kind(PopupKind::Help)
            .title("Controls")
            .close_on(KeyCode::Char('?'))
    }

    // Remove todo confirm dialog
    pub fn remove_confirm(data: String) -> Confirm {
        Confirm::new()
            .with_message(format!("Are you sure to remove selected task ({})?", data))
            .with_border_color(Color::Rgb(249, 109, 109))
    }
}
