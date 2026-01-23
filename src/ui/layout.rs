use ratatui::layout::{Constraint, Direction, Layout, Rect};

// Min possible terminal size
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

// Get main_layout for menu widget
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    let upper_area = chunks[0];
    let bottom_area = chunks[1];

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(upper_area);

    (body_chunks[0], body_chunks[1], bottom_area)
}

// Center a widget in a rect
pub fn center(percent_x: u16, percent_y: u16, rect: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(rect);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

// Check if terminal/frame area is small
pub fn is_terminal_small(width: u16, height: u16) -> bool {
    width < MIN_WIDTH || height < MIN_HEIGHT
}
