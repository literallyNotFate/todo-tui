use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(height)]).flex(Flex::Center);
    let [vert_area] = vertical.areas(area);

    let horizontal = Layout::horizontal([Constraint::Max(width)]).flex(Flex::Center);
    let [final_area] = horizontal.areas(vert_area);

    final_area
}
