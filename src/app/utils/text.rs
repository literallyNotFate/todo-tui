pub fn wrap_text(input: &str, max_width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    for line in input.lines() {
        let mut current_line: String = String::new();
        for word in line.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width {
                lines.push(current_line.trim_end().to_string());
                current_line = String::new();
            }

            current_line.push_str(word);
            current_line.push(' ');
        }

        if !current_line.is_empty() {
            lines.push(current_line.trim_end().to_string());
        }
    }

    lines
}
