// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use crate::app::{
        ui::{
            dialogs::{
                dialog::{Dialog, DialogResult},
                popup::{
                    popup::{Popup, PopupCloseBehavior, PopupKind},
                    utils::{color_based_on_popup_kind, lines_based_on_popup},
                },
            },
            renderer::state::WidgetPosition,
        },
        utils::colors::theme::*,
    };
    use ratatui::{
        crossterm::event::KeyCode,
        layout::Rect,
        style::{Modifier, Style},
        text::{Line, Span},
        widgets::Padding,
    };

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn should_create_default_popup() {
        let popup: Popup = Popup::new();

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.message, "");
        assert_eq!(popup.title, None);
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );
        assert_eq!(popup.position, WidgetPosition::Center);
        assert!(popup.styles.show_title);
        assert_eq!(popup.styles.padding, Padding::new(2, 2, 1, 1));
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup: Popup = Popup::new()
            .kind(PopupKind::Success)
            .with_message("Task completed!")
            .title("Great!")
            .close_on_any_key()
            .position(WidgetPosition::TopRight);

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Task completed!");
        assert_eq!(popup.title, Some("Great!".to_string()));
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
        assert_eq!(popup.position, WidgetPosition::TopRight);
        assert_eq!(
            popup.styles.border_color,
            color_based_on_popup_kind(PopupKind::Success)
        );
    }

    #[test]
    fn should_create_area_for_popup() {
        let frame: Rect = create_helper_frame();
        let popup: Popup = Popup::new().with_message("Short message").title("Test");
        let area: Rect = popup.area(frame);

        assert!(area.x > 20 && area.x < 60);
        assert!(area.y > 10 && area.y < 30);
        assert!(area.width > 15 && area.width < 50);
        assert!(area.height > 3 && area.height < 15);
    }

    #[test]
    fn should_calculate_popup_titles_length() {
        let popup: Popup = Popup::new().title("Test").close_on(KeyCode::Enter);
        let (top_len, bottom_len): (usize, usize) = popup.titles_len();

        assert!(top_len > 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_calculate_popup_titles_length_with_no_top_title() {
        let popup: Popup = Popup::new().no_title();
        let (top_len, bottom_len): (usize, usize) = popup.titles_len();

        assert_eq!(top_len, 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup: Popup = Popup::new().close_on_any_key();

        assert_eq!(
            popup.handle_key(KeyCode::Char('q')),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Enter),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Esc),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup: Popup = Popup::new().close_on(KeyCode::Char('y'));

        assert_eq!(
            popup.handle_key(KeyCode::Char('y')),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    #[test]
    fn should_popup_not_close() {
        let mut popup: Popup = Popup::new().not_closable();

        assert_eq!(popup.handle_key(KeyCode::Char('q')), None);
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    // Utils functions
    #[test]
    fn should_return_border_color_based_on_popup_kind() {
        let mut popup: Popup = Popup::new().kind(PopupKind::Error);
        assert_eq!(popup.styles.border_color, ERROR_POPUP_FG);

        popup = popup.kind(PopupKind::Success);
        assert_eq!(popup.styles.border_color, SUCCESS_POPUP_FG);

        popup = popup.kind(PopupKind::Help);
        assert_eq!(popup.styles.border_color, HELP_POPUP_FG);

        popup = popup.kind(PopupKind::Info);
        assert_eq!(popup.styles.border_color, INFO_POPUP_FG);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_with_title() {
        let lines: (Line, Line) = lines_based_on_popup(
            Some("Test".to_string()),
            PopupKind::Info,
            PopupCloseBehavior::Specific(KeyCode::Enter),
            true,
        );

        let expected_top_line: Line = Line::from(Span::styled(
            " Test ",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));
        assert_eq!(lines.0, expected_top_line);

        let expected_bottom_line: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                "<Return>",
                Style::default()
                    .fg(COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ]);
        assert_eq!(lines.1, expected_bottom_line);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_without_title() {
        let lines: (Line, Line) =
            lines_based_on_popup(None, PopupKind::Info, PopupCloseBehavior::AnyKey, false);
        assert_eq!(lines.0, Line::default());

        let expected_bottom_line: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                "any key",
                Style::default()
                    .fg(COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ]);
        assert_eq!(lines.1, expected_bottom_line);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_defaults() {
        let lines: (Line, Line) =
            lines_based_on_popup(None, PopupKind::Error, PopupCloseBehavior::None, true);

        let expected_top_line: Line = Line::from(Span::styled(
            " Error ",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));

        assert_eq!(lines.0, expected_top_line);
        assert_eq!(lines.1, Line::default());
    }
}
