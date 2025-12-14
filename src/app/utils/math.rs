// Calculate percentage of a value
pub fn percentage_of(value: u16, percent: f32) -> usize {
    (value as f32 * (percent / 100.0)).floor() as usize
}

// Calculate max line of a slice of strings
pub fn calculate_max_line_len(lines: &[&str]) -> usize {
    lines.iter().map(|l| l.chars().count()).max().unwrap_or(1)
}
