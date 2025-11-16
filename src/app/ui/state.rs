use super::widgets::{inputbox::input::InputBox, popup_widget::popup::Popup};

#[derive(Default)]
pub struct UIState {
    pub popup: Option<Popup>,
    pub inputbox: Option<InputBox>,
}

impl UIState {
    // Popup
    pub fn show_popup(&mut self, popup: Popup) {
        self.popup = Some(popup)
    }

    pub fn close_popup(&mut self) {
        self.popup = None;
    }

    // Input
    pub fn show_input(&mut self, input: InputBox) {
        self.inputbox = Some(input);
    }

    pub fn close_input(&mut self) {
        self.inputbox = None;
    }
}
