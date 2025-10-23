pub mod popup;

use popup::Popup;
use ratatui::Frame;

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

    // Render
    pub fn render(&self, frame: &mut Frame) {
        if let Some(popup) = &self.popup {
            popup.render(frame);
        }
    }
}
