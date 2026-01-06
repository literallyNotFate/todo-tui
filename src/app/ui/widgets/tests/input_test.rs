// Unit-tests for input widget
#[cfg(test)]
mod tests {
    use crate::app::{
        ui::{
            renderer::state::WidgetPosition,
            widgets::input::{
                input::{Input, InputMode, InputResult},
                utils::render_input_titles,
            },
        },
        utils::colors::theme::*,
    };
    use ratatui::{
        crossterm::event::KeyCode,
        layout::{Alignment, Rect},
        style::Color,
        widgets::Padding,
    };

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 30)
    }

    #[test]
    fn should_create_input_insert() {
        let input: Input = Input::insert();

        assert_eq!(input.buffer, "");
        assert_eq!(input.cursor, 0);
        assert_eq!(input.mode, InputMode::Insert);
        assert_eq!(input.position, WidgetPosition::Center);
        assert!(input.title.is_none());
        assert!(input.styles.show_title);
        assert_eq!(input.styles.fg_color, INPUT_ADD_FG);
    }

    #[test]
    fn should_create_input_insert_with_value() {
        let input: Input = Input::edit("Hello World");

        assert_eq!(input.buffer, "Hello World");
        assert_eq!(input.cursor, 11);
        assert_eq!(input.mode, InputMode::Edit);
        assert_eq!(input.styles.fg_color, INPUT_EDIT_FG);
    }

    #[test]
    fn should_create_input_with_chaining_api() {
        let input: Input = Input::insert()
            .title("Custom Title")
            .no_title()
            .with_fg_color(Color::Yellow)
            .with_padding(Padding::new(2, 3, 1, 1))
            .position(WidgetPosition::TopLeft);

        assert_eq!(input.title, Some("Custom Title".to_string()));
        assert!(!input.styles.show_title);
        assert_eq!(input.styles.fg_color, Color::Yellow);
        assert_eq!(input.styles.padding, Padding::new(2, 3, 1, 1));
        assert_eq!(input.position, WidgetPosition::TopLeft);
    }

    #[test]
    fn should_create_area_for_input() {
        let input: Input = Input::insert();
        let frame_area: Rect = create_helper_frame();

        let area: Rect = input.area(frame_area);

        assert_eq!(area.width, 50);
        assert_eq!(area.height, 3);
        assert_eq!(area.x, 25);
        assert_eq!(area.y, 14);
    }

    #[test]
    fn should_input_handle_key_char_insert() {
        let mut input: Input = Input::insert();
        let result: InputResult = input.handle_key(KeyCode::Char('a'));

        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.buffer, "a");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn should_input_handle_key_backspace() {
        let mut input: Input = Input::edit("abc");

        input.handle_key(KeyCode::Left);
        input.handle_key(KeyCode::Left);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.buffer, "bc");
        assert_eq!(input.cursor, 0);

        input.handle_key(KeyCode::Right);
        input.handle_key(KeyCode::Right);
        input.handle_key(KeyCode::Backspace);
        assert_eq!(input.buffer, "b");
        assert_eq!(input.cursor, 1);
    }

    #[test]
    fn should_input_handle_key_enter_and_esc() {
        let mut input: Input = Input::edit("Test");

        let result_enter: InputResult = input.handle_key(KeyCode::Enter);
        assert!(matches!(result_enter, InputResult::Submit(s) if s == "Test"));

        let mut input_cancel: Input = Input::insert();
        let result_esc: InputResult = input_cancel.handle_key(KeyCode::Esc);
        assert_eq!(result_esc, InputResult::Cancel);
    }

    #[test]
    fn should_test_max_chars_limit_for_input() {
        let mut input: Input = Input::insert();

        for _ in 0..Input::MAX_CHARS {
            input.handle_key(KeyCode::Char('x'));
        }
        assert_eq!(input.buffer.len(), Input::MAX_CHARS);
        assert_eq!(input.cursor, Input::MAX_CHARS);

        let result = input.handle_key(KeyCode::Char('y'));
        assert_eq!(result, InputResult::Continue);
        assert_eq!(input.buffer.len(), Input::MAX_CHARS);
        assert_eq!(input.cursor, Input::MAX_CHARS);
    }

    #[test]
    fn should_render_input_titles() {
        let (top, bottom) =
            render_input_titles(Some("Test".to_string()), InputMode::Insert, true, 10, 46);

        assert_eq!(top.spans.len(), 1);
        assert_eq!(top.spans[0].content, " Test ");

        assert_eq!(bottom.spans.len(), 1);
        assert_eq!(bottom.spans[0].content, " 10 / 46 ");
        assert_eq!(bottom.alignment, Some(Alignment::Right));
    }
}
