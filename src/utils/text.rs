// Wrap content in widgets
pub fn wrap_text(input: &str, max_width: usize) -> Vec<String> {
    if input.trim().is_empty() || max_width == 0 {
        return vec!["".to_string()];
    }

    let mut result: Vec<String> = Vec::new();

    for logical_line in input.lines() {
        let trimmed: &str = logical_line.trim_end();
        if trimmed.chars().count() <= max_width {
            result.push(trimmed.to_string());
            continue;
        }

        let mut current: String = String::new();
        for word in logical_line.split_whitespace() {
            let word_len: usize = word.chars().count();
            let added_space: usize = if current.is_empty() { 0 } else { 1 };

            if current.chars().count() + added_space + word_len > max_width && !current.is_empty() {
                result.push(current.trim_end().to_string());
                current.clear();
            }

            if !current.is_empty() {
                current.push(' ');
            }

            current.push_str(word);
        }

        if !current.is_empty() {
            result.push(current.trim_end().to_string());
        }
    }

    if result.is_empty() {
        vec!["".to_string()]
    } else {
        result
    }
}

// Unit-tests for wrap text function
#[cfg(test)]
mod tests {
    use super::*;

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
    fn should_return_nothing_if_max_width_is_0() {
        let wrapped: Vec<String> = wrap_text("Some text", 0);
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
