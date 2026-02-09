use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Min possible terminal size values
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Get layout for dashboard widget
pub fn main_layout(area: Rect) -> (Rect, Rect, Rect) {
    let [upper_area, bottom_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .areas(area);

    let [sidebar, main] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .areas(upper_area);

    (sidebar, main, bottom_area)
}

/// Center a widget in a rect
pub fn center(rect: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let widget_area = Layout::default()
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
        .split(widget_area[1])[1]
}

/// Check if terminal/frame area is small
pub fn is_terminal_small(width: u16, height: u16) -> bool {
    width < MIN_WIDTH || height < MIN_HEIGHT
}
