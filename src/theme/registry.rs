use crate::theme::{PaletteDisk, ThemeName, ThemePalette};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;

/// Unique theme identifier.
/// Used for saving to the configuration file and searching the registry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeID {
    /// Builtin theme from ThemeName enum
    Builtin(ThemeName),
    /// Custom theme that is being loaded from the external .toml file
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeRegistry {
    /// Load custom palettes storage
    pub custom_palettes: HashMap<String, ThemePalette>,
    /// List of all IDs for navigation (first - builtin, then - custom)
    pub all_ids: Vec<ThemeID>,
    /// Index of currently selected theme
    pub current_index: usize,
}

impl ThemeRegistry {
    /// Initializes registry collecting all themes into one list
    pub fn load(initial: ThemeID) -> Self {
        let mut custom_palettes = HashMap::new();
        let mut all_ids = Vec::new();

        for name in ThemeName::iter() {
            all_ids.push(ThemeID::Builtin(name));
        }

        let external_themes: Vec<(String, ThemePalette)> = PaletteDisk::load_all();
        for (name, palette) in external_themes {
            all_ids.push(ThemeID::Custom(name.clone()));
            custom_palettes.insert(name, palette);
        }

        let current_index: usize = all_ids.iter().position(|id| id == &initial).unwrap_or(0);

        Self {
            custom_palettes,
            all_ids,
            current_index,
        }
    }

    /// Returns the palette of theme based on its ID
    pub fn get_palette(&self, id: &ThemeID) -> ThemePalette {
        match id {
            ThemeID::Builtin(name) => name.palette(),
            ThemeID::Custom(name) => self
                .custom_palettes
                .get(name)
                .cloned()
                .unwrap_or_else(|| ThemeName::default().palette()),
        }
    }

    /// Ref to current active ID
    pub fn current_id(&self) -> &ThemeID {
        &self.all_ids[self.current_index]
    }

    /// Switches to the next theme (cycle)
    pub fn next(&mut self) {
        if !self.all_ids.is_empty() {
            self.current_index = (self.current_index + 1) % self.all_ids.len();
        }
    }

    /// Switches to the prev theme (cycle)
    pub fn prev(&mut self) {
        if !self.all_ids.is_empty() {
            if self.current_index == 0 {
                self.current_index = self.all_ids.len() - 1;
            } else {
                self.current_index -= 1;
            }
        }
    }
}

impl std::fmt::Display for ThemeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeID::Builtin(name) => write!(f, "{}", name),
            ThemeID::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Unit-tests for theme registry
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    #[test]
    fn should_load_registry_with_builtin_themes() {
        let initial = ThemeID::Builtin(ThemeName::GruvboxDark);
        let registry = ThemeRegistry::load(initial.clone());

        let builtin_count = ThemeName::iter().count();
        assert!(registry.all_ids.len() >= builtin_count);
        assert_eq!(registry.current_id(), &initial);
    }

    #[test]
    fn should_fallback_to_index_zero_if_initial_not_found() {
        let ghost_id = ThemeID::Custom("non-existent".to_string());
        let registry = ThemeRegistry::load(ghost_id);

        assert_eq!(registry.current_index, 0);
    }

    #[test]
    fn should_cycle_next_properly() {
        let initial = ThemeID::Builtin(ThemeName::GruvboxDark);
        let mut registry = ThemeRegistry::load(initial);

        let start_index = registry.current_index;
        let total = registry.all_ids.len();

        registry.next();
        assert_eq!(registry.current_index, (start_index + 1) % total);

        for _ in 0..total {
            registry.next();
        }
        assert_eq!(registry.current_index, (start_index + 1) % total);
    }

    #[test]
    fn should_cycle_prev_properly() {
        let mut registry = ThemeRegistry::load(ThemeID::Builtin(ThemeName::GruvboxDark));
        let total = registry.all_ids.len();

        registry.current_index = 0;
        registry.prev();
        assert_eq!(registry.current_index, total - 1);

        registry.next();
        assert_eq!(registry.current_index, 0);
    }

    #[test]
    fn should_get_correct_palette_for_builtin() {
        let registry = ThemeRegistry::load(ThemeID::Builtin(ThemeName::GruvboxDark));
        let id = ThemeID::Builtin(ThemeName::GruvboxDark);

        let palette = registry.get_palette(&id);
        assert_eq!(palette, ThemeName::GruvboxDark.palette());
    }

    #[test]
    fn should_fallback_palette_for_unknown_custom_theme() {
        let registry = ThemeRegistry::load(ThemeID::Builtin(ThemeName::GruvboxDark));
        let ghost_id = ThemeID::Custom("missing".to_string());

        let palette = registry.get_palette(&ghost_id);
        assert_eq!(palette, ThemeName::default().palette());
    }

    #[test]
    fn should_display_theme_id_correctly() {
        let builtin = ThemeID::Builtin(ThemeName::TokyoNight);
        let custom = ThemeID::Custom("Monokai-Vibrant".to_string());

        assert_eq!(format!("{}", builtin), "Tokyo Night");
        assert_eq!(format!("{}", custom), "Monokai-Vibrant");
    }

    #[test]
    fn test_registry_order_logic() {
        let registry = ThemeRegistry::load(ThemeID::Builtin(ThemeName::GruvboxDark));

        if let Some(first) = registry.all_ids.first() {
            assert!(matches!(first, ThemeID::Builtin(_)));
        }
    }
}
