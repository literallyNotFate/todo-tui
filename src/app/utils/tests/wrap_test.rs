// Unit-tests for wrap text function
#[cfg(test)]
mod tests {
    use crate::app::utils::text::wrap_text;

    #[test]
    fn should_fit_single_line() {
        let input: &str = "Short message that fits completely";
        let wrapped: Vec<String> = wrap_text(input, 40);
        assert_eq!(wrapped, vec!["Short message that fits completely"]);
    }

    #[test]
    fn should_wrap_long_line() {
        let input: &str = "This is a very long line that definitely needs to be wrapped properly";
        let wrapped: Vec<String> = wrap_text(input, 20);

        assert_eq!(
            wrapped,
            vec![
                "This is a very long",
                "line that definitely",
                "needs to be wrapped",
                "properly"
            ]
        );
    }

    #[test]
    fn should_wrap_line_with_breaks() {
        let input: &str = "First line\nSecond line\nThird very long line that needs wrapping";
        let wrapped: Vec<String> = wrap_text(input, 25);

        assert_eq!(
            wrapped,
            vec![
                "First line",
                "Second line",
                "Third very long line that",
                "needs wrapping"
            ]
        );
    }

    #[test]
    fn should_wrap_text_in_controls_message() {
        let help_lines: Vec<&str> = vec![
            " a -> append a todo",
            " r -> rename a todo",
            " d -> delete a todo",
            " x -> clear all todos",
            " Enter -> mark as completed",
            " ? -> toggle help",
        ];
        let input: String = help_lines.join("\n");
        let wrapped: Vec<String> = wrap_text(&input, 40);

        assert_eq!(wrapped.len(), help_lines.len());
        assert_eq!(wrapped[0], " a -> append a todo");
        assert_eq!(wrapped[5], " ? -> toggle help");
    }

    #[test]
    fn should_wrap_empty_input() {
        let wrapped: Vec<String> = wrap_text("", 20);
        assert_eq!(wrapped, vec![""]);

        let wrapped: Vec<String> = wrap_text("   ", 20);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn should_wrap_with_long_word() {
        let input: &str = "supercalifragilisticexpialidocious and short words";
        let wrapped: Vec<String> = wrap_text(input, 20);

        assert_eq!(
            wrapped,
            vec!["supercalifragilisticexpialidocious", "and short words"]
        );
    }
}
