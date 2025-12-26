// Wrap content in widgets
pub fn wrap_text(input: &str, max_width: usize) -> Vec<String> {
    if input.trim().is_empty() {
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
