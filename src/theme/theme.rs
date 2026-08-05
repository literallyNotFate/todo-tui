use crate::theme::{ThemePalette, ThemeRegistry, registry::ThemeId};

/// Main theme interface for the UI
/// Has registry that manages all available theme palettes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub registry: ThemeRegistry,
}

impl Theme {
    /// Create a new theme with the given ID
    pub fn new(initial: ThemeId) -> Self {
        Self {
            registry: ThemeRegistry::load(initial),
        }
    }

    /// Returns the color palette for the current theme (automatically chooses between Builtin and Custom)
    pub fn palette(&self) -> ThemePalette {
        let id = self.theme_id();
        self.registry.get_palette(id)
    }

    /// Returns the currently selected theme ID
    pub fn theme_id(&self) -> &ThemeId {
        self.registry.current_id()
    }

    /// Returns the theme name for displaying in UI
    pub fn name(&self) -> String {
        self.theme_id().to_string()
    }

    /// Switch to the next theme wrapper (through registry)
    pub fn next(&mut self) {
        self.registry.next();
    }

    /// Switch to the prev theme wrapper (through registry)
    pub fn prev(&mut self) {
        self.registry.prev();
    }

    /// Check if this is a light theme
    pub fn is_light(&self) -> bool {
        self.palette().is_light()
    }

    /// Check if this is a dark theme
    pub fn is_dark(&self) -> bool {
        self.palette().is_dark()
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.theme_id())
    }
}

/// Unit-tests for theme
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::BuiltinTheme;
    use strum::IntoEnumIterator;

    #[test]
    fn should_check_if_all_builtin_themes_have_palettes() {
        for theme_name in BuiltinTheme::iter() {
            let palette = theme_name.palette();
            assert_ne!(palette.fg, palette.bg);
        }
    }

    #[test]
    fn should_test_theme_cycling_next() {
        let mut theme = Theme::new(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        let original_id = theme.theme_id().clone();
        let count = theme.registry.all_ids.len();

        for _ in 0..count {
            theme.next();
        }

        assert_eq!(theme.theme_id(), &original_id);
    }

    #[test]
    fn should_test_theme_cycling_prev() {
        let mut theme = Theme::new(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        let original_id = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        let count = theme.registry.all_ids.len();

        theme.prev();
        assert_ne!(theme.theme_id(), &original_id);

        theme.next();
        assert_eq!(theme.theme_id(), &original_id);

        for _ in 0..count {
            theme.prev();
        }
        assert_eq!(theme.theme_id(), &original_id);
    }

    #[test]
    fn should_test_light_dark_detection() {
        assert!(BuiltinTheme::GruvboxDark.palette().is_dark());
        assert!(!BuiltinTheme::GruvboxLight.palette().is_dark());

        let dark_theme: Theme = Theme::new(ThemeId::Builtin(BuiltinTheme::GruvboxDark));
        assert!(dark_theme.is_dark());

        let light_theme: Theme = Theme::new(ThemeId::Builtin(BuiltinTheme::GruvboxLight));
        assert!(!light_theme.is_dark());
    }

    #[test]
    fn should_test_to_string_for_theme_id() {
        let id = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        assert_eq!(id.to_string(), "Gruvbox Dark");

        let custom_id = ThemeId::Custom("My-cool-theme".to_string());
        assert_eq!(custom_id.to_string(), "My-cool-theme");
    }

    #[test]
    fn should_test_theme_initialization_from_id() {
        let id = ThemeId::Builtin(BuiltinTheme::KanagawaWave);
        let theme = Theme::new(id.clone());
        assert_eq!(theme.theme_id(), &id);
    }

    #[test]
    fn should_verify_registry_integrity() {
        let theme = Theme::default();
        let builtin_count = BuiltinTheme::iter().count();
        assert!(theme.registry.all_ids.len() >= builtin_count);
        assert!(theme.registry.current_index < theme.registry.all_ids.len());
    }

    #[test]
    fn should_handle_custom_theme_display_names() {
        assert_eq!(BuiltinTheme::TokyoNight.to_string(), "Tokyo Night");
        assert_eq!(
            BuiltinTheme::CatppuccinMocha.to_string(),
            "Catppuccin Mocha"
        );
    }
}
