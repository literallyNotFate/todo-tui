// Unit-tests for layout functions
#[cfg(test)]
mod tests {
    use crate::app::{
        ui::renderer::state::Anchor,
        utils::layout::{anchored, calculate_content_size, center},
    };
    use ratatui::{layout::Rect, widgets::Padding};

    // Helper function to create rect
    fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn should_create_rect_in_center() {
        let frame: Rect = rect(0, 0, 100, 50);
        let widget: Rect = center(frame, 40, 10);

        assert_eq!(widget.x, (100 - 40) / 2);
        assert_eq!(widget.y, (50 - 10) / 2);
        assert_eq!(widget.width, 40);
        assert_eq!(widget.height, 10);
    }

    #[test]
    fn should_create_area_based_on_center_position() {
        let frame: Rect = rect(0, 0, 80, 40);
        let area: Rect = anchored(frame, 30, 10, Anchor::Center);
        let expected: Rect = center(frame, 30, 10);
        assert_eq!(area, expected);
    }

    #[test]
    fn should_create_area_based_on_top_right_position() {
        let frame: Rect = rect(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::TopRight);

        assert_eq!(area.x, 10 + 100 - 30 - 2);
        assert_eq!(area.y, 10 + 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_top_left_position() {
        let frame: Rect = rect(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::TopLeft);

        assert_eq!(area.x, 10 + 2);
        assert_eq!(area.y, 10 + 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_bottom_right_position() {
        let frame: Rect = rect(10, 10, 100, 50);
        let area: Rect = anchored(frame, 30, 10, Anchor::BottomRight);

        assert_eq!(area.x, 10 + 100 - 30 - 2);
        assert_eq!(area.y, 10 + 50 - 10 - 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_create_area_based_on_bottom_left_position() {
        let frame = rect(10, 10, 100, 50);
        let area = anchored(frame, 30, 10, Anchor::BottomLeft);

        assert_eq!(area.x, 10 + 2);
        assert_eq!(area.y, 10 + 50 - 10 - 1);
        assert_eq!(area.width, 30);
        assert_eq!(area.height, 10);
    }

    #[test]
    fn should_calculate_content_size_with_small_content() {
        let frame: Rect = rect(0, 0, 100, 50);
        let content: &str = "Short message";
        let (width, height): (u16, u16) =
            calculate_content_size(frame, content, 0, 0, Padding::uniform(1), 80.0);

        assert_eq!(width, 21);
        assert!((5..=7).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_with_long_content() {
        let frame: Rect = rect(0, 0, 100, 50);
        let content: &str = "This is a very long message that should be wrapped into multiple lines when calculating size";
        let (width, height): (u16, u16) =
            calculate_content_size(frame, content, 20, 30, Padding::uniform(1), 80.0);

        assert!((80..=88).contains(&width));
        assert!((5..=8).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_empty_content() {
        let frame: Rect = rect(0, 0, 100, 50);
        let (width, height): (u16, u16) =
            calculate_content_size(frame, "", 20, 30, Padding::uniform(1), 80.0);

        assert!((20..=30).contains(&width));
        assert!((4..=6).contains(&height));
    }

    #[test]
    fn should_calculate_content_size_with_big_titles() {
        let frame: Rect = rect(0, 0, 100, 50);
        let content: &str = "Short";
        let (width, _): (u16, u16) =
            calculate_content_size(frame, content, 60, 40, Padding::uniform(1), 80.0);

        assert!(width >= 60);
    }
}
