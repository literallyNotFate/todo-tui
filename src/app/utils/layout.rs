use crate::app::ui::renderer::state::Anchor;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::Padding,
};

// Calculate area based on widget position (anchor)
pub fn anchored(frame: Rect, mut width: u16, mut height: u16, anchor: Anchor) -> Rect {
    width = width.min(frame.width.saturating_sub(2).max(1));
    height = height.min(frame.height.saturating_sub(2).max(1));

    match anchor {
        Anchor::Center => centered(frame, width, height),
        Anchor::TopRight => Rect {
            x: frame.x + frame.width - width - 2,
            y: frame.y + 1,
            width,
            height,
        },
        Anchor::TopLeft => Rect {
            x: frame.x + 2,
            y: frame.y + 1,
            width,
            height,
        },
        Anchor::BottomRight => Rect {
            x: frame.x + frame.width - width - 2,
            y: frame.y + frame.height - height - 1,
            width,
            height,
        },
        Anchor::BottomLeft => Rect {
            x: frame.x + 2,
            y: frame.y + frame.height - height - 1,
            width,
            height,
        },
    }
}

// Center a widget
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(height)]).flex(Flex::Center);
    let [vert_area] = vertical.areas(area);

    let horizontal = Layout::horizontal([Constraint::Max(width)]).flex(Flex::Center);
    let [final_area] = horizontal.areas(vert_area);

    final_area
}

// Calculate content size (width/height) for modals and other widgets
pub fn calculate_content_size(
    frame_area: Rect,
    content: &str,
    title_top_width: usize,
    title_bottom_width: usize,
    padding: Padding,
    max_percent: f32,
) -> (u16, u16) {
    use super::{
        math::{calculate_max_line_len, percentage_of},
        text::wrap_text,
    };

    let max_allowed_width: u16 = percentage_of(frame_area.width, max_percent);
    let raw_lines: Vec<&str> = content.lines().collect::<Vec<_>>();

    let content_max_line: usize = calculate_max_line_len(&raw_lines);
    let base_width: usize = content_max_line.min(max_allowed_width.into());

    let content_height: usize = wrap_text(content, base_width).len();
    let mut height = content_height + padding.top as usize + padding.bottom as usize + 2;

    let mut width = base_width + padding.left as usize + padding.right as usize + 6;

    width = width.max(title_top_width);
    width = width.max(title_bottom_width);

    width = width.min(frame_area.width as usize);
    height = height.min(frame_area.height as usize);

    (width as u16, height as u16)
}
