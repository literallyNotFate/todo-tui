use crate::theme::ThemePalette;
use ratatui::style::Color;

/// Themes for application
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum ThemeName {
    #[default]
    GruvboxDark,
    CatppuccinMocha,
    TokyoNight,
    Everforest,
    OneDark,
}

impl ThemeName {
    /// Returns a slice containing all available theme names
    pub const fn all() -> &'static [Self] {
        &[
            Self::GruvboxDark,
            Self::CatppuccinMocha,
            Self::TokyoNight,
            Self::Everforest,
            Self::OneDark,
        ]
    }

    /// Returns the human-readable display name for the theme
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::TokyoNight => "Tokyo Night",
            Self::Everforest => "Everforest",
            Self::OneDark => "One Dark",
        }
    }

    /// Returns the next theme in the list
    pub fn next(&self) -> Self {
        match self {
            Self::GruvboxDark => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::TokyoNight,
            Self::TokyoNight => Self::Everforest,
            Self::Everforest => Self::OneDark,
            Self::OneDark => Self::GruvboxDark,
        }
    }

    /// Returns the color palette for this theme.
    pub const fn palette(self) -> ThemePalette {
        match self {
            Self::GruvboxDark => ThemePalette {
                accent: Color::Rgb(250, 189, 47),     // Yellow
                secondary: Color::Rgb(211, 134, 155), // Purple
                bg: Color::Rgb(40, 40, 40),           // bg0
                fg: Color::Rgb(235, 219, 178),        // fg
                muted: Color::Rgb(146, 131, 116),     // gray
                selection: Color::Rgb(80, 73, 69),    // bg2
                error: Color::Rgb(251, 73, 52),       // red
                warning: Color::Rgb(254, 128, 25),    // orange
                success: Color::Rgb(184, 187, 38),    // green
                info: Color::Rgb(131, 165, 152),      // aqua
            },

            Self::OneDark => ThemePalette {
                accent: Color::Rgb(97, 175, 239),     // Blue
                secondary: Color::Rgb(198, 120, 221), // Magenta
                bg: Color::Rgb(40, 44, 52),           // Background
                fg: Color::Rgb(171, 178, 191),        // Foreground
                muted: Color::Rgb(92, 99, 112),       // Comment
                selection: Color::Rgb(62, 68, 81),    // Selection
                error: Color::Rgb(224, 108, 117),     // Red
                warning: Color::Rgb(229, 192, 123),   // Yellow
                success: Color::Rgb(152, 195, 121),   // Green
                info: Color::Rgb(86, 182, 194),       // Cyan
            },

            Self::CatppuccinMocha => ThemePalette {
                accent: Color::Rgb(180, 190, 254),    // Blue
                secondary: Color::Rgb(245, 194, 231), // Pink
                bg: Color::Rgb(30, 30, 46),           // Base
                fg: Color::Rgb(205, 214, 244),        // Text
                muted: Color::Rgb(108, 112, 134),     // Overlay0
                selection: Color::Rgb(88, 91, 112),   // Surface0
                error: Color::Rgb(243, 139, 168),     // Red
                warning: Color::Rgb(249, 226, 175),   // Yellow
                success: Color::Rgb(166, 227, 161),   // Green
                info: Color::Rgb(148, 226, 213),      // Teal
            },

            Self::TokyoNight => ThemePalette {
                accent: Color::Rgb(122, 162, 247),    // Blue
                secondary: Color::Rgb(187, 154, 247), // Magenta
                bg: Color::Rgb(26, 27, 38),           // Background
                fg: Color::Rgb(192, 202, 245),        // Foreground
                muted: Color::Rgb(86, 95, 137),       // Comment
                selection: Color::Rgb(41, 46, 66),    // Selection
                error: Color::Rgb(247, 118, 142),     // Red
                warning: Color::Rgb(224, 175, 104),   // Yellow
                success: Color::Rgb(158, 206, 106),   // Green
                info: Color::Rgb(125, 207, 255),      // Cyan
            },

            Self::Everforest => ThemePalette {
                accent: Color::Rgb(131, 193, 120),    // Green
                secondary: Color::Rgb(214, 153, 182), // Purple
                bg: Color::Rgb(47, 53, 55),           // bg0
                fg: Color::Rgb(211, 198, 170),        // fg
                muted: Color::Rgb(133, 146, 137),     // gray
                selection: Color::Rgb(68, 78, 79),    // bg2
                error: Color::Rgb(230, 126, 128),     // red
                warning: Color::Rgb(219, 188, 127),   // yellow
                success: Color::Rgb(167, 192, 128),   // green
                info: Color::Rgb(124, 195, 191),      // aqua
            },
        }
    }
}

impl std::fmt::Display for ThemeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for ThemeName {
    type Err = String;

    /// Parse a theme name from a string
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized: String = s
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        match normalized.as_str() {
            "onedark" => Ok(Self::OneDark),
            "catppuccinmocha" | "mocha" => Ok(Self::CatppuccinMocha),
            "gruvboxdark" | "gruvbox" => Ok(Self::GruvboxDark),
            "tokyonight" | "tokyo" => Ok(Self::TokyoNight),
            "everforest" => Ok(Self::Everforest),
            _ => Err(format!("Unknown theme: {s}")),
        }
    }
}

/// A theme configuration wrapper providing convenient access to theme colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Theme {
    pub name: ThemeName,
}

impl Theme {
    /// Create a new theme with the given name
    pub fn new(name: ThemeName) -> Self {
        Self { name }
    }

    /// Returns the color palette for the current theme
    pub fn palette(&self) -> ThemePalette {
        self.name.palette()
    }

    /// Check if this is a light theme
    pub fn is_light(&self) -> bool {
        self.palette().is_light()
    }

    /// Check if this is a dark theme
    pub fn is_dark(&self) -> bool {
        self.palette().is_dark()
    }

    /// Cycle to the next theme in the list
    pub fn next(&mut self) {
        self.name = self.name.next();
    }
}

impl From<ThemeName> for Theme {
    fn from(name: ThemeName) -> Self {
        Self::new(name)
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Unit-tests for theme
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_check_if_all_themes_have_palettes() {
        for theme in ThemeName::all() {
            let palette = theme.palette();
            assert_ne!(palette.fg, palette.bg);
        }
    }

    #[test]
    fn should_test_theme_cycling() {
        let mut theme = ThemeName::GruvboxDark;
        let original = theme;

        for _ in 0..ThemeName::all().len() {
            theme = theme.next();
        }

        assert_eq!(theme, original);
    }

    #[test]
    fn should_test_light_dark_detection_for_themes() {
        assert!(ThemeName::GruvboxDark.palette().is_dark());
        assert!(ThemeName::CatppuccinMocha.palette().is_dark());
        assert!(ThemeName::TokyoNight.palette().is_dark());
    }

    #[test]
    fn should_display_name_for_theme() {
        assert_eq!(ThemeName::GruvboxDark.display_name(), "Gruvbox Dark");
        assert_eq!(ThemeName::TokyoNight.display_name(), "Tokyo Night");
        assert_eq!(
            ThemeName::CatppuccinMocha.display_name(),
            "Catppuccin Mocha"
        );
    }

    #[test]
    fn should_test_theme_display_trait() {
        assert_eq!(format!("{}", ThemeName::GruvboxDark), "Gruvbox Dark");
        assert_eq!(format!("{}", ThemeName::TokyoNight), "Tokyo Night");
    }

    #[test]
    fn should_test_theme_from_name() {
        let theme: Theme = ThemeName::OneDark.into();
        assert_eq!(theme.name, ThemeName::OneDark);
    }
}
