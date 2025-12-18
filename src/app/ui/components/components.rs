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

    // Append todo confirm dialog
    pub fn append_confirm() -> Confirm {
        Confirm::new()
            .with_message("Append this todo?")
            .with_border_color(Color::Rgb(249, 214, 109))
    }

    // Rename todo confirm dialog
    pub fn rename_confirm() -> Confirm {
        Confirm::new()
            .with_message("Rename this todo?")
            .with_border_color(Color::Rgb(109, 172, 249))
    }

    // Remove todo confirm dialog
    pub fn remove_confirm() -> Confirm {
        Confirm::new()
            .with_message("Remove this todo?")
            .with_border_color(Color::Rgb(249, 109, 109))
    }
}
