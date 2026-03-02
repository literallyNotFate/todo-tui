use crate::{theme::ThemePalette, traits::InteractableEnum};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Task priority
#[derive(
    Serialize, Deserialize, Debug, Clone, Default, Hash, Eq, PartialEq, Copy, PartialOrd, Ord,
)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

impl InteractableEnum for Priority {
    fn all() -> &'static [Self] {
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
    pub fn palette(&self, palette: &ThemePalette) -> Color {
        match self {
            Priority::High => palette.error,
            Priority::Medium => palette.warning,
            Priority::Low => palette.success,
        }
    }
}

/// Unit-tests for priority
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    #[test]
    fn should_return_string_from_enum() {
        assert_eq!(Priority::Low.to_string(), "Low");
        assert_eq!(Priority::Medium.to_string(), "Medium");
        assert_eq!(Priority::High.to_string(), "High");
    }

    #[test]
    fn should_return_variants_from_enum() {
        assert_eq!(
            Priority::all(),
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
    fn should_compare_priorities() {
        assert!(Priority::High > Priority::Low);
        assert!(Priority::Medium > Priority::Low);
        assert!(Priority::High > Priority::Medium);
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
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();

        let mut priority: Priority = Priority::Low;
        assert_eq!(priority.palette(&palette), palette.success);

        priority = Priority::Medium;
        assert_eq!(priority.palette(&palette), palette.warning);

        priority = Priority::High;
        assert_eq!(priority.palette(&palette), palette.error);
    }
}
