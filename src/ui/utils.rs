/// Truncates title if too long (for notifications/summary)
pub fn truncate(text: &str, max_width: usize) -> String {
    let char_count = text.chars().count();
    if char_count > max_width && max_width > 0 {
        let truncated: String = text.chars().take(max_width.saturating_sub(1)).collect();
        format!("{}…", truncated)
    } else {
        text.to_string()
    }
}

/// Unit-tests for util functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_truncate_text_shorter_than_limit() {
        let text = "Hello";
        assert_eq!(truncate(text, 10), "Hello");
    }

    #[test]
    fn should_truncate_text_longer_than_limit() {
        let text = "Long task description";
        let result = truncate(text, 10);

        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
        assert_eq!(result, "Long task…");
    }

    #[test]
    fn should_truncate_text_exact_limit() {
        let text = "Exact";
        assert_eq!(truncate(text, 5), "Exact");
    }

    #[test]
    fn should_truncate_text_unicode_support() {
        let text = "🦀🦀🦀🦀🦀";
        let result = truncate(text, 3);

        assert_eq!(result, "🦀🦀…");
        assert_eq!(result.chars().count(), 3);
    }
}
