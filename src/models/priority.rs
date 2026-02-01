use crate::{theme::ThemeColors, traits::InteractableEnum};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Default, Hash, Eq, PartialEq, Copy, PartialOrd, Ord,
)]
pub enum Priority {
    High,
    Medium,
    #[default]
    Low,
}

impl InteractableEnum for Priority {
    fn all_variants() -> &'static [Self] {
        &[Self::Low, Self::Medium, Self::High]
    }

    fn to_string(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

impl Priority {
    pub fn color(&self, theme: &ThemeColors) -> Color {
        match self {
            Priority::High => theme.error,
            Priority::Medium => theme.warning,
            Priority::Low => theme.success,
        }
    }
}

// Unit-tests for priority
#[cfg(test)]
mod tests {
    use crate::theme::Theme;

    use super::*;

    #[test]
    fn should_return_string_from_enum() {
        assert_eq!(Priority::Low.to_string(), "Low");
        assert_eq!(Priority::Medium.to_string(), "Medium");
        assert_eq!(Priority::High.to_string(), "High");
    }

    #[test]
    fn should_return_variants_from_enum() {
        assert_eq!(
            Priority::all_variants(),
            [Priority::Low, Priority::Medium, Priority::High]
        );
    }

    #[test]
    fn should_return_corresponding_index_for_enum() {
        assert_eq!(Priority::Low.index(), 0);
        assert_eq!(Priority::Medium.index(), 1);
        assert_eq!(Priority::High.index(), 2);
    }

    #[test]
    fn should_iterate_through_enum() {
        let mut priority: Priority = Priority::Low;

        priority = priority.next();
        assert_eq!(priority, Priority::Medium);

        priority = priority.next();
        assert_eq!(priority, Priority::High);

        priority = priority.next();
        assert_eq!(priority, Priority::Low);

        priority = priority.prev();
        assert_eq!(priority, Priority::High);
    }

    #[test]
    fn should_return_right_color_of_priority_with_theme() {
        let theme: Theme = Theme::Gruvbox;
        let colors: ThemeColors = theme.colors();

        let mut priority: Priority = Priority::Low;
        assert_eq!(priority.color(&colors), colors.success);

        priority = Priority::Medium;
        assert_eq!(priority.color(&colors), colors.warning);

        priority = Priority::High;
        assert_eq!(priority.color(&colors), colors.error);
    }
}
