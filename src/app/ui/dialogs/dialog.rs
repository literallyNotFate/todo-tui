use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect};

pub enum DialogAction {
    Remove,
    Append(String),
    Rename(String),
}

pub enum DialogResult {
    None,
    Confirmed,
    Cancelled,
}

pub trait Dialog {
    fn new() -> Self
    where
        Self: Sized;
    fn area(&self, frame_area: Rect) -> Rect;
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult>;
}
