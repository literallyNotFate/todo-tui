use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect};

#[derive(Debug, PartialEq)]
pub enum DialogResult {
    Confirmed,
    Cancelled,
}

// Dialog actions (remove todo, clear, save, none - for popup)
#[derive(Debug, Clone, PartialEq)]
pub enum DialogIntent {
    None,
    Remove,
    Clear,
    Save,
}

pub trait Dialog {
    fn new() -> Self
    where
        Self: Sized;
    fn area(&self, frame_area: Rect) -> Rect;
    fn titles_len(&self) -> (usize, usize);
    fn render(&self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult>;
}
