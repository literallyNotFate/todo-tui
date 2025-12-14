use ratatui::style::Color;

use crate::app::ui::widgets::{
    confirm::confirm::{Confirm, ConfirmAction},
    popup::popup::Popup,
};

pub struct Components;

impl Components {
    // Help popup (controls)
    pub fn help_popup() -> Popup {
        use crate::app::ui::widgets::popup::popup::PopupKind;
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

        Popup::new(help_message.join("\n"))
            .kind(PopupKind::Help)
            .title("Controls")
            .close_on(KeyCode::Char('?'))
    }

    // Append todo confirm dialog
    pub fn append_confirm(text: String) -> Confirm {
        Confirm::new()
            .with_message("Append this todo?")
            .with_border_color(Color::Rgb(249, 214, 109))
            .action(ConfirmAction::Append(text))
    }

    // Rename todo confirm dialog
    pub fn rename_confirm(text: String) -> Confirm {
        Confirm::new()
            .with_message("Rename this todo?")
            .with_border_color(Color::Rgb(109, 172, 249))
            .action(ConfirmAction::Rename(text))
    }

    // Remove todo confirm dialog
    pub fn remove_confirm() -> Confirm {
        Confirm::new()
            .with_message("Remove this todo?")
            .with_border_color(Color::Rgb(249, 109, 109))
            .action(ConfirmAction::Remove)
    }
}
