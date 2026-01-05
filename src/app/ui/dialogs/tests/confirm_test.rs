// Unit-tests for confirm widget
#[cfg(test)]
mod tests {
    use crate::app::ui::{
        dialogs::{
            confirm::{
                confirm::{Confirm, ConfirmOption},
                utils::render_confirm_buttons,
            },
            dialog::{Dialog, DialogResult},
        },
        renderer::state::WidgetPosition,
    };
    use ratatui::{crossterm::event::KeyCode, layout::Rect, style::Color, widgets::Padding};

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn should_create_default_confirm() {
        let confirm: Confirm = Confirm::new();

        assert_eq!(confirm.message, "");
        assert!(matches!(confirm.select, ConfirmOption::Cancel));
        assert!(matches!(confirm.position, WidgetPosition::Center));
        assert_eq!(confirm.styles.border_color, Color::Rgb(252, 252, 252));
        assert_eq!(confirm.styles.padding, Padding::new(2, 2, 3, 3));
    }

    #[test]
    fn should_create_confirm_with_chaining_api() {
        let confirm: Confirm = Confirm::new()
            .with_message("Delete all tasks?")
            .position(WidgetPosition::BottomLeft)
            .with_border_color(Color::Red)
            .with_padding(Padding::uniform(4));

        assert_eq!(confirm.message, "Delete all tasks?");
        assert!(matches!(confirm.position, WidgetPosition::BottomLeft));
        assert_eq!(confirm.styles.border_color, Color::Red);
        assert_eq!(confirm.styles.padding, Padding::uniform(4));
        assert!(matches!(confirm.select, ConfirmOption::Cancel));
    }

    #[test]
    fn should_create_area_for_confirm() {
        let frame: Rect = create_helper_frame();
        let confirm: Confirm =
            Confirm::new().with_message("Are you sure? This action cannot be undone.");

        let area: Rect = confirm.area(frame);

        assert!(area.x > 10 && area.x < 60);
        assert!(area.y > 10 && area.y < 30);
        assert!(area.width > 30 && area.width < 80);
        assert!(area.height > 6 && area.height < 20);
    }

    #[test]
    fn should_calculate_confirm_titles_length() {
        let confirm: Confirm = Confirm::new();
        let (top_len, bottom_len): (usize, usize) = confirm.titles_len();

        assert_eq!(top_len, 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_handle_left_right_key_confirm() {
        let mut confirm: Confirm = Confirm::new();

        assert!(matches!(confirm.select, ConfirmOption::Cancel));

        confirm.handle_key(KeyCode::Left);
        assert!(matches!(confirm.select, ConfirmOption::Yes));

        confirm.handle_key(KeyCode::Right);
        assert!(matches!(confirm.select, ConfirmOption::Cancel));

        confirm.handle_key(KeyCode::Char('h'));
        assert!(matches!(confirm.select, ConfirmOption::Yes));

        confirm.handle_key(KeyCode::Char('l'));
        assert!(matches!(confirm.select, ConfirmOption::Cancel));
    }

    #[test]
    fn should_handle_key_enter_confirm() {
        let mut confirm: Confirm = Confirm::new();

        confirm.select = ConfirmOption::Yes;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(DialogResult::Confirmed)
        );

        confirm.select = ConfirmOption::Cancel;
        assert_eq!(
            confirm.handle_key(KeyCode::Enter),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_esc_confirm() {
        let mut confirm: Confirm = Confirm::new();

        assert_eq!(
            confirm.handle_key(KeyCode::Esc),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_other_keys_confirm() {
        let mut confirm: Confirm = Confirm::new();

        assert_eq!(confirm.handle_key(KeyCode::Char('a')), None);
        assert_eq!(confirm.handle_key(KeyCode::Down), None);
        assert!(matches!(confirm.select, ConfirmOption::Cancel));

        assert_eq!(
            confirm.handle_key(KeyCode::Char('y')),
            Some(DialogResult::Confirmed)
        );
        assert_eq!(
            confirm.handle_key(KeyCode::Char('n')),
            Some(DialogResult::Cancelled)
        );
    }

    // Utils
    #[test]
    fn should_render_confirm_buttons_yes_selected() {
        let line = render_confirm_buttons(ConfirmOption::Yes);

        let spans: Vec<&str> = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();

        assert_eq!(spans, vec!["[ ", "Yes", " ]", "   ", "[ ", "Cancel", " ]"]);

        let colors: Vec<Color> = line
            .spans
            .iter()
            .map(|span| span.style.fg.unwrap_or(Color::Reset))
            .collect();

        assert!(matches!(colors[1], Color::Rgb(155, 201, 166)));
        assert!(matches!(colors[5], Color::Rgb(150, 150, 150)));
    }

    #[test]
    fn should_render_confirm_buttons_cancel_selected() {
        let line = render_confirm_buttons(ConfirmOption::Cancel);

        let colors: Vec<Color> = line
            .spans
            .iter()
            .map(|span| span.style.fg.unwrap_or(Color::Reset))
            .collect();

        assert!(matches!(colors[5], Color::Rgb(201, 155, 155)));
    }
}
