// Unit-tests for math utils functions
#[cfg(test)]
mod tests {
    use crate::app::utils::math::{calculate_max_line_len, percentage_of};

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
