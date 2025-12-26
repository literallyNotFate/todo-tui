// Calculate percentage of a value
pub fn percentage_of(value: u16, percent: f32) -> u16 {
    let ratio = (percent / 100.0).clamp(0.0, 1.0);
    ((value as f32) * ratio).floor() as u16
}

// Calculate max line of a slice of strings
pub fn calculate_max_line_len(lines: &[&str]) -> usize {
    lines.iter().map(|l| l.chars().count()).max().unwrap_or(1)
}
