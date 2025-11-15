use crate::app::ui::widgets::popup_widget::popup::{Popup, PopupKind};
use ratatui::crossterm::event::KeyCode;

// Help popup (controls)
pub fn help_popup() -> Popup {
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
