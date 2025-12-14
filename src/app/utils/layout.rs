use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Padding,
};

use super::{
    math::{calculate_max_line_len, percentage_of},
    text::wrap_text,
};

// Center a widget
pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(height)]).flex(Flex::Center);
    let [vert_area] = vertical.areas(area);

    let horizontal = Layout::horizontal([Constraint::Max(width)]).flex(Flex::Center);
    let [final_area] = horizontal.areas(vert_area);

    final_area
}

// Calculate modal area for popups/confirm
pub fn calculate_modal_area(
    frame_area: Rect,
    content: &str,
    title_top_width: usize,
    title_bottom_width: usize,
    padding: Padding,
    max_percent: f32,
) -> Rect {
    let max_allowed_width: usize = percentage_of(frame_area.width, max_percent);
    let raw_lines: Vec<&str> = content.lines().collect::<Vec<_>>();

    let content_max_line: usize = calculate_max_line_len(&raw_lines);
    let base_width: usize = content_max_line.min(max_allowed_width);

    let content_height: usize = wrap_text(content, base_width).len();
    let mut height = content_height + padding.top as usize + padding.bottom as usize + 2;

    let mut width = base_width + padding.left as usize + padding.right as usize + 6;

    width = width.max(title_top_width);
    width = width.max(title_bottom_width);

    width = width.min(frame_area.width as usize);
    height = height.min(frame_area.height as usize);

    center(frame_area, width as u16, height as u16)
}
