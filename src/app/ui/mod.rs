pub mod components;
pub mod renderer;

use components::popup::Popup;

#[derive(Default)]
pub struct UIState {
    pub popup: Option<Popup>,
}

impl UIState {
    // Popup
    pub fn show_popup(&mut self, popup: Popup) {
        self.popup = Some(popup)
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }
}
