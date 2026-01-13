// Calculate percentage of a value
pub fn percentage_of(value: u16, percent: f32) -> u16 {
    let ratio = (percent / 100.0).clamp(0.0, 1.0);
    ((value as f32) * ratio).floor() as u16
}

// Calculate max line of a slice of strings
pub fn calculate_max_line_len(lines: &[&str]) -> usize {
    lines.iter().map(|l| l.chars().count()).max().unwrap_or(1)
}

// Unit-tests for math utils functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_percentage_of_value() {
        assert_eq!(percentage_of(50, 20.0), 10);
    }

    #[test]
    fn should_return_0_if_percent_le_zero() {
        assert_eq!(percentage_of(50, -10.0), 0);
    }

    #[test]
    fn should_return_value_if_percent_ge_100() {
        assert_eq!(percentage_of(50, 110.0), 50);
    }

    #[test]
    fn should_return_max_len_of_a_line() {
        let lines: &[&str] = &["Max", "Line", "Search"];
        assert_eq!(calculate_max_line_len(lines), 6);
    }

    #[test]
    fn should_return_1_on_empty_slice() {
        let lines: &[&str] = &[];
        assert_eq!(calculate_max_line_len(lines), 1);
    }
}
