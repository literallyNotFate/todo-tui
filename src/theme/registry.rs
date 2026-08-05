use crate::theme::{BuiltinTheme, PaletteDisk, SystemTheme, ThemePalette};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;

/// Unique theme identifier.
/// Used for saving to the configuration file and searching the registry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeId {
    /// Themes from the `BuiltinTheme` enum
    Builtin(BuiltinTheme),
    /// Themes from the `SystemTheme` enum
    System(SystemTheme),
    /// Custom theme that is being loaded from the external .toml file
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeRegistry {
    /// Load custom palettes storage
    pub custom_palettes: HashMap<String, ThemePalette>,
    /// List of all IDs for navigation (first - builtin, then - custom)
    pub all_ids: Vec<ThemeId>,
    /// Index of currently selected theme
    pub current_index: usize,
}

impl ThemeRegistry {
    /// Initializes registry collecting all themes into one list
    pub fn load(initial: ThemeId) -> Self {
        let mut custom_palettes = HashMap::new();
        let mut all_ids = Vec::new();

        for builtin_theme in BuiltinTheme::iter() {
            all_ids.push(ThemeId::Builtin(builtin_theme));
        }

        for system_theme in SystemTheme::iter() {
            all_ids.push(ThemeId::System(system_theme));
        }

        let external_themes: Vec<(String, ThemePalette)> = PaletteDisk::load_all();
        for (name, palette) in external_themes {
            all_ids.push(ThemeId::Custom(name.clone()));
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
    pub fn get_palette(&self, id: &ThemeId) -> ThemePalette {
        match id {
            ThemeId::Builtin(b) => b.palette(),
            ThemeId::System(s) => s.palette(),
            ThemeId::Custom(c) => {
                if let Some(palette) = self.custom_palettes.get(c) {
                    return palette.clone();
                }

                if let Some(palette) = PaletteDisk::load_single(c) {
                    return palette;
                }

                BuiltinTheme::default().palette()
            }
        }
    }

    /// Ref to current active ID
    pub fn current_id(&self) -> &ThemeId {
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

impl std::fmt::Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeId::Builtin(b_name) => write!(f, "{}", b_name),
            ThemeId::System(s_name) => write!(f, "{}", s_name),
            ThemeId::Custom(c_name) => {
                let mut chars = c_name.chars();
                let capitalized = match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().chain(chars).collect(),
                };
                write!(f, "{}", capitalized)
            }
        }
    }
}

impl std::str::FromStr for ThemeId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(builtin) = BuiltinTheme::from_str(s, true) {
            return Ok(ThemeId::Builtin(builtin));
        }

        if let Ok(system) = SystemTheme::from_str(s, true) {
            return Ok(ThemeId::System(system));
        }

        if !s.is_empty() {
            return Ok(ThemeId::Custom(s.to_string()));
        }

        Err(format!("Invalid theme identifier: '{}'", s))
    }
}

/// Unit-tests for theme registry
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::BuiltinTheme;

    #[test]
    fn should_load_registry_with_builtin_themes() {
        let initial = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        let registry = ThemeRegistry::load(initial.clone());

        let builtin_count = BuiltinTheme::iter().count();
        assert!(registry.all_ids.len() >= builtin_count);
        assert_eq!(registry.current_id(), &initial);
    }

    #[test]
    fn should_fallback_to_index_zero_if_initial_not_found() {
        let ghost_id = ThemeId::Custom("non-existent".to_string());
        let registry = ThemeRegistry::load(ghost_id);

        assert_eq!(registry.current_index, 0);
    }

    #[test]
    fn should_cycle_next_properly() {
        let initial = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
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
        let mut registry = ThemeRegistry::load(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        let total = registry.all_ids.len();

        registry.current_index = 0;
        registry.prev();
        assert_eq!(registry.current_index, total - 1);

        registry.next();
        assert_eq!(registry.current_index, 0);
    }

    #[test]
    fn should_get_correct_palette_for_builtin() {
        let registry = ThemeRegistry::load(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        let id = ThemeId::Builtin(BuiltinTheme::GruvboxDark);

        let palette = registry.get_palette(&id);
        assert_eq!(palette, BuiltinTheme::GruvboxDark.palette());
    }

    #[test]
    fn should_fallback_palette_for_unknown_custom_theme() {
        let registry = ThemeRegistry::load(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        let ghost_id = ThemeId::Custom("missing".to_string());

        let palette = registry.get_palette(&ghost_id);
        assert_eq!(palette, BuiltinTheme::default().palette());
    }

    #[test]
    fn should_display_theme_id_correctly() {
        let builtin = ThemeId::Builtin(BuiltinTheme::TokyoNight);
        let custom = ThemeId::Custom("Monokai-Vibrant".to_string());

        assert_eq!(format!("{}", builtin), "Tokyo Night");
        assert_eq!(format!("{}", custom), "Monokai-Vibrant");
    }

    #[test]
    fn test_registry_order_logic() {
        let registry = ThemeRegistry::load(ThemeId::Builtin(BuiltinTheme::GruvboxDark));

        if let Some(first) = registry.all_ids.first() {
            assert!(matches!(first, ThemeId::Builtin(_)));
        }
    }

    #[test]
    fn should_load_registry_with_system_themes() {
        let initial = ThemeId::System(SystemTheme::Tty);
        let registry = ThemeRegistry::load(initial.clone());

        assert!(
            registry
                .all_ids
                .iter()
                .any(|id| matches!(id, ThemeId::System(SystemTheme::Tty)))
        );
        assert!(
            registry
                .all_ids
                .iter()
                .any(|id| matches!(id, ThemeId::System(SystemTheme::NoColor)))
        );
        assert_eq!(registry.current_id(), &initial);
    }

    #[test]
    fn should_get_correct_palette_for_system() {
        let registry = ThemeRegistry::load(ThemeId::System(SystemTheme::Tty));
        let id = ThemeId::System(SystemTheme::Tty);

        let palette = registry.get_palette(&id);
        assert_eq!(palette, SystemTheme::Tty.palette());
    }

    #[test]
    fn should_display_system_theme_id_correctly() {
        let system = ThemeId::System(SystemTheme::NoColor);
        assert_eq!(format!("{}", system), "No Color");
    }
}
