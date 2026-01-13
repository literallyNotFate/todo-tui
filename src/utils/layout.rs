use crate::state::Anchor;
use ratatui::{layout::Rect, widgets::Padding};

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
    use ratatui::layout::{Constraint, Flex, Layout};

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

// Unit-tests for layout functions
#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create rect
    fn create_helper_frame(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn should_create_rect_in_center() {
        let frame: Rect = create_helper_frame(0, 0, 100, 50);
        let widget: Rect = centered(frame, 40, 10);

        assert_eq!(widget.x, (100 - 40) / 2);
        assert_eq!(widget.y, (50 - 10) / 2);
        assert_eq!(widget.width, 40);
        assert_eq!(widget.height, 10);
    }

    #[test]
    fn should_create_area_based_on_center_position() {
        let frame: Rect = create_helper_frame(0, 0, 80, 40);
        let area: Rect = anchored(frame, 30, 10, Anchor::Center);
        let expected: Rect = centered(frame, 30, 10);
        assert_eq!(area, expected);
    }

    #[test]
    fn should_create_area_based_on_top_right_position() {
        let frame: Rect = create_helper_frame(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::TopRight);

        assert_eq!(area.x, 10 + 100 - 30 - 2);
        assert_eq!(area.y, 10 + 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_top_left_position() {
        let frame: Rect = create_helper_frame(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::TopLeft);

        assert_eq!(area.x, 10 + 2);
        assert_eq!(area.y, 10 + 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_bottom_right_position() {
        let frame: Rect = create_helper_frame(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::BottomRight);

        assert_eq!(area.x, 10 + 100 - 30 - 2);
        assert_eq!(area.y, 10 + 50 - 10 - 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_bottom_left_position() {
        let frame = create_helper_frame(10, 10, 100, 50);
        let area = anchored(frame, 30, 10, Anchor::BottomLeft);

        assert_eq!(area.x, 10 + 2);
        assert_eq!(area.y, 10 + 50 - 10 - 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_calculate_content_size_with_small_content() {
        let frame: Rect = create_helper_frame(0, 0, 100, 50);
        let content: &str = "Short message";
        let (width, height): (u16, u16) =
            calculate_content_size(frame, content, 0, 0, Padding::uniform(1), 80.0);

        assert_eq!(width, 21);
        assert!((5..=7).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_with_long_content() {
        let frame: Rect = create_helper_frame(0, 0, 100, 50);
        let content: &str = "This is a very long message that should be wrapped into multiple lines when calculating size";
        let (width, height): (u16, u16) =
            calculate_content_size(frame, content, 20, 30, Padding::uniform(1), 80.0);

        assert!((80..=88).contains(&width));
        assert!((5..=8).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_empty_content() {
        let frame: Rect = create_helper_frame(0, 0, 100, 50);
        let (width, height): (u16, u16) =
            calculate_content_size(frame, "", 20, 30, Padding::uniform(1), 80.0);

        assert!((20..=30).contains(&width));
        assert!((4..=6).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_with_big_titles() {
        let frame: Rect = create_helper_frame(0, 0, 100, 50);
        let content: &str = "Short";
        let (width, _): (u16, u16) =
            calculate_content_size(frame, content, 60, 40, Padding::uniform(1), 80.0);

        assert!(width >= 60);
    }
}
