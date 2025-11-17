use super::widgets::{
    confirm_widget::confirm::Confirm, inputbox::input::InputBox, popup_widget::popup::Popup,
};

#[derive(Default)]
pub struct UIState {
    pub popup: Option<Popup>,
    pub inputbox: Option<InputBox>,
    pub confirm: Option<Confirm>,
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

    // Confirm
    pub fn show_confirm(&mut self, confirm: Confirm) {
        self.confirm = Some(confirm);
    }

    pub fn close_confirm(&mut self) {
        self.confirm = None;
    }
}
