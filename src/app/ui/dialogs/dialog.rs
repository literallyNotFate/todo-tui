use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect};

// Dialog result (confirmed or cancelled for confirm)
pub enum DialogResult {
    Confirmed,
    Cancelled,
}

// Basic dialog trait for popup/confirm
pub trait Dialog {
    fn new() -> Self
    where
        Self: Sized;
    fn area(&self, frame_area: Rect) -> Rect;
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult>;
}
