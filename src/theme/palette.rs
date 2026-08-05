use crate::theme::BuiltinTheme;
use ratatui::style::Color;

/// A semantic color palette for a theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    /// Primary accent color for highlights and active elements
    pub accent: Color,

    /// Secondary accent color for less prominent highlights
    pub secondary: Color,

    /// Main background color
    pub bg: Color,

    /// Secondary background color
    pub bg2: Color,

    /// Primary foreground/text color
    pub fg: Color,

    /// Muted/dimmed text color
    pub muted: Color,

    /// Selection/highlight background color
    pub selection: Color,

    /// Error/red color for critical states
    pub error: Color,

    /// Warning/yellow color for caution states
    pub warning: Color,

    /// Success/green color for positive states
    pub success: Color,

    /// Info/blue color for informational states
    pub info: Color,
}

impl ThemePalette {
    /// Check if this is a light theme based on background brightness
    pub fn is_light(&self) -> bool {
        if let Color::Rgb(r, g, b) = self.bg {
            let brightness = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000;
            brightness > 127
        } else {
            false
        }
    }

    /// Check if this is a dark theme
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }
}

impl Default for ThemePalette {
    /// Returns the default palette
    fn default() -> Self {
        BuiltinTheme::default().palette()
    }
}

/// Unit-tests for theme palette
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn should_correctly_detect_light_and_dark_colors() {
        let light_palette = ThemePalette {
            bg: Color::Rgb(255, 255, 255),
            ..ThemePalette::default()
        };
        assert!(light_palette.is_light());
        assert!(!light_palette.is_dark());

        let dark_palette = ThemePalette {
            bg: Color::Rgb(0, 0, 0),
            ..ThemePalette::default()
        };
        assert!(!dark_palette.is_light());
        assert!(dark_palette.is_dark());

        let border_dark = ThemePalette {
            bg: Color::Rgb(127, 127, 127),
            ..ThemePalette::default()
        };
        assert!(border_dark.is_dark());
    }

    #[test]
    fn should_handle_non_rgb_colors_as_dark() {
        let indexed_palette = ThemePalette {
            bg: Color::Indexed(15),
            ..ThemePalette::default()
        };
        assert!(!indexed_palette.is_light());
        assert!(indexed_palette.is_dark());
    }

    #[test]
    fn should_detect_brightness_based_on_luma_formula() {
        let deep_blue = ThemePalette {
            bg: Color::Rgb(0, 0, 255),
            ..ThemePalette::default()
        };
        assert!(deep_blue.is_dark());

        let bright_yellow = ThemePalette {
            bg: Color::Rgb(255, 255, 0),
            ..ThemePalette::default()
        };
        assert!(bright_yellow.is_light());
    }

    #[test]
    fn should_verify_default_palette_matches_default_theme() {
        let palette = ThemePalette::default();
        let expected = BuiltinTheme::default().palette();

        assert_eq!(palette, expected);
        assert!(palette.is_dark());
    }

    #[test]
    fn should_implement_equality_correctly() {
        let p1 = ThemePalette::default();
        let p2 = ThemePalette::default();
        let mut p3 = ThemePalette::default();
        p3.accent = Color::Red;

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }
}
