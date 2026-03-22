use crate::{core::Selectable, theme::ThemePalette};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Themes for application
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::Display,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "title_case")]
#[non_exhaustive]
pub enum ThemeName {
    // Dark themes
    #[default]
    GruvboxDark,
    CatppuccinMocha,
    TokyoNight,
    KanagawaWave,
    MelangeDark,
    RosePineMoon,
    Oxocarbon,

    // Light themes
    GruvboxLight,
    CatppuccinLatte,
    KanagawaLotus,
    MelangeLight,
    RosePineDawn,

    #[serde(rename = "github-light")]
    #[strum(to_string = "GitHub Light")]
    GitHubLight,
    SolarizedLight,
}

impl ThemeName {
    /// Returns the color palette for this theme.
    pub const fn palette(self) -> ThemePalette {
        match self {
            // Dark
            Self::GruvboxDark => ThemePalette {
                accent: Color::Rgb(250, 189, 47),     // Yellow
                secondary: Color::Rgb(211, 134, 155), // Purple
                bg: Color::Rgb(40, 40, 40),           // bg0
                bg2: Color::Rgb(50, 48, 47),          // bg2
                fg: Color::Rgb(235, 219, 178),        // fg
                muted: Color::Rgb(146, 131, 116),     // gray
                selection: Color::Rgb(80, 73, 69),    // bg2
                error: Color::Rgb(251, 73, 52),       // red
                warning: Color::Rgb(254, 128, 25),    // orange
                success: Color::Rgb(184, 187, 38),    // green
                info: Color::Rgb(131, 165, 152),      // aqua
            },

            Self::CatppuccinMocha => ThemePalette {
                accent: Color::Rgb(180, 190, 254),    // Blue
                secondary: Color::Rgb(245, 194, 231), // Pink
                bg: Color::Rgb(30, 30, 46),           // Base
                bg2: Color::Rgb(17, 17, 27),          // bg2
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
                bg2: Color::Rgb(36, 40, 59),          // bg2
                fg: Color::Rgb(192, 202, 245),        // Foreground
                muted: Color::Rgb(86, 95, 137),       // Comment
                selection: Color::Rgb(41, 46, 66),    // Selection
                error: Color::Rgb(247, 118, 142),     // Red
                warning: Color::Rgb(224, 175, 104),   // Yellow
                success: Color::Rgb(158, 206, 106),   // Green
                info: Color::Rgb(125, 207, 255),      // Cyan
            },

            Self::KanagawaWave => ThemePalette {
                accent: Color::Rgb(228, 192, 123),    // Carp Yellow
                secondary: Color::Rgb(149, 123, 172), // Oni Violet
                bg: Color::Rgb(30, 30, 44),           // Sumi Ink
                bg2: Color::Rgb(22, 22, 29),          // bg2
                fg: Color::Rgb(220, 215, 186),        // Old White
                muted: Color::Rgb(114, 113, 105),     // Sage Gray
                selection: Color::Rgb(45, 47, 58),    // Wave Blue
                error: Color::Rgb(196, 74, 76),       // Autumn Red
                warning: Color::Rgb(255, 157, 0),     // Ronin Orange
                success: Color::Rgb(118, 148, 82),    // Spring Green
                info: Color::Rgb(101, 143, 145),      // Sui Blue
            },

            Self::MelangeDark => ThemePalette {
                accent: Color::Rgb(235, 140, 97),     // Carrot Orange
                secondary: Color::Rgb(163, 138, 122), // Taupe
                bg: Color::Rgb(41, 37, 34),           // Coffee Bean
                bg2: Color::Rgb(53, 49, 44),          // bg2
                fg: Color::Rgb(236, 225, 206),        // Parchment
                muted: Color::Rgb(134, 122, 110),     // Dust
                selection: Color::Rgb(63, 58, 54),    // Roasted
                error: Color::Rgb(190, 83, 83),       // Berry
                warning: Color::Rgb(220, 155, 110),   // Apricot
                success: Color::Rgb(120, 137, 100),   // Moss
                info: Color::Rgb(123, 158, 155),      // Eucalyptus
            },

            Self::RosePineMoon => ThemePalette {
                accent: Color::Rgb(235, 188, 186),    // Rose
                secondary: Color::Rgb(196, 167, 231), // Iris
                bg: Color::Rgb(35, 33, 54),           // Base
                bg2: Color::Rgb(42, 40, 62),          // bg2
                fg: Color::Rgb(224, 222, 244),        // Text
                muted: Color::Rgb(144, 140, 170),     // Subtle
                selection: Color::Rgb(42, 40, 62),    // Surface
                error: Color::Rgb(235, 111, 146),     // Love
                warning: Color::Rgb(246, 193, 119),   // Gold
                success: Color::Rgb(49, 116, 143),    // Pine
                info: Color::Rgb(156, 207, 216),      // Foam
            },

            Self::Oxocarbon => ThemePalette {
                accent: Color::Rgb(61, 214, 255),     // Cyan
                secondary: Color::Rgb(255, 123, 215), // Pink
                bg: Color::Rgb(22, 22, 22),           // Jet Black
                bg2: Color::Rgb(38, 38, 38),          // bg2
                fg: Color::Rgb(242, 242, 242),        // White
                muted: Color::Rgb(82, 82, 82),        // Gray
                selection: Color::Rgb(38, 38, 38),    // Gray 80
                error: Color::Rgb(255, 123, 123),     // Red
                warning: Color::Rgb(190, 149, 255),   // Purple
                success: Color::Rgb(66, 190, 101),    // Green
                info: Color::Rgb(130, 175, 255),      // Blue
            },

            // Light
            Self::GruvboxLight => ThemePalette {
                accent: Color::Rgb(175, 58, 3),       // Dark Orange/Rust
                secondary: Color::Rgb(143, 63, 113),  // Purple
                bg: Color::Rgb(251, 241, 213),        // bg0_hard light
                bg2: Color::Rgb(235, 219, 178),       // bg2
                fg: Color::Rgb(60, 56, 54),           // fg0 light (Dark Gray)
                muted: Color::Rgb(146, 131, 116),     // Gray
                selection: Color::Rgb(235, 219, 178), // bg2 light
                error: Color::Rgb(157, 0, 6),         // Red
                warning: Color::Rgb(181, 118, 20),    // Yellow/Ochre
                success: Color::Rgb(121, 116, 14),    // Green
                info: Color::Rgb(7, 102, 120),        // Blue
            },

            Self::CatppuccinLatte => ThemePalette {
                accent: Color::Rgb(30, 102, 245),     // Blue
                secondary: Color::Rgb(136, 57, 239),  // Mauve
                bg: Color::Rgb(239, 241, 245),        // Base (Off-white)
                bg2: Color::Rgb(220, 224, 232),       // bg2
                fg: Color::Rgb(76, 79, 105),          // Text (Darker blue-gray)
                muted: Color::Rgb(156, 160, 176),     // Overlay
                selection: Color::Rgb(204, 208, 218), // Surface
                error: Color::Rgb(210, 15, 57),       // Red
                warning: Color::Rgb(223, 142, 29),    // Yellow
                success: Color::Rgb(64, 160, 43),     // Green
                info: Color::Rgb(4, 165, 184),        // Sky
            },

            Self::KanagawaLotus => ThemePalette {
                accent: Color::Rgb(77, 95, 128),      // Crystal Blue
                secondary: Color::Rgb(98, 81, 107),   // Lotus Violet
                bg: Color::Rgb(242, 236, 214),        // Lotus White
                bg2: Color::Rgb(227, 222, 199),       // bg2
                fg: Color::Rgb(84, 82, 77),           // Ink (Ink dark gray)
                muted: Color::Rgb(147, 144, 131),     // Dust
                selection: Color::Rgb(227, 222, 199), // Selection
                error: Color::Rgb(196, 74, 76),       // Autumn Red
                warning: Color::Rgb(230, 154, 0),     // Orange
                success: Color::Rgb(118, 148, 82),    // Spring Green
                info: Color::Rgb(12, 144, 141),       // Teal
            },

            Self::RosePineDawn => ThemePalette {
                accent: Color::Rgb(144, 122, 169),    // Iris
                secondary: Color::Rgb(180, 99, 122),  // Love
                bg: Color::Rgb(250, 244, 237),        // Base
                bg2: Color::Rgb(242, 233, 222),       // bg2
                fg: Color::Rgb(87, 82, 121),          // Text
                muted: Color::Rgb(152, 147, 165),     // Subtle
                selection: Color::Rgb(242, 233, 222), // Surface
                error: Color::Rgb(180, 99, 122),      // Love (Reddish)
                warning: Color::Rgb(234, 157, 52),    // Gold
                success: Color::Rgb(40, 105, 131),    // Pine
                info: Color::Rgb(86, 148, 159),       // Foam
            },

            Self::MelangeLight => ThemePalette {
                accent: Color::Rgb(140, 68, 32),      // Terracotta
                secondary: Color::Rgb(72, 89, 44),    // Forest Green
                bg: Color::Rgb(244, 241, 232),        // Parchment
                bg2: Color::Rgb(235, 229, 216),       // bg2
                fg: Color::Rgb(53, 47, 43),           // Dark Brown
                muted: Color::Rgb(171, 156, 145),     // Sand
                selection: Color::Rgb(225, 219, 203), // Milled
                error: Color::Rgb(152, 47, 47),       // Red
                warning: Color::Rgb(173, 112, 44),    // Ochre
                success: Color::Rgb(90, 105, 71),     // Moss
                info: Color::Rgb(61, 98, 95),         // Teal
            },

            Self::GitHubLight => ThemePalette {
                accent: Color::Rgb(5, 112, 239),      // Blue
                secondary: Color::Rgb(130, 80, 223),  // Purple
                bg: Color::Rgb(255, 255, 255),        // Pure White
                bg2: Color::Rgb(246, 248, 250),       // bg2
                fg: Color::Rgb(31, 35, 40),           // Dark Gray
                muted: Color::Rgb(101, 108, 118),     // Gray/Comments
                selection: Color::Rgb(234, 238, 242), // Selection
                error: Color::Rgb(207, 34, 46),       // Red
                warning: Color::Rgb(154, 103, 0),     // Golden
                success: Color::Rgb(17, 129, 51),     // Green
                info: Color::Rgb(10, 103, 213),       // Blue
            },

            Self::SolarizedLight => ThemePalette {
                accent: Color::Rgb(38, 139, 210),     // Blue
                secondary: Color::Rgb(211, 54, 130),  // Magenta
                bg: Color::Rgb(253, 246, 227),        // Base3 (Cream/Yellowish)
                bg2: Color::Rgb(238, 232, 213),       // bg2
                fg: Color::Rgb(101, 123, 131),        // Base00 (Blue-Gray)
                muted: Color::Rgb(147, 161, 161),     // Base1
                selection: Color::Rgb(238, 232, 213), // Base2
                error: Color::Rgb(220, 50, 47),       // Red
                warning: Color::Rgb(181, 137, 0),     // Yellow
                success: Color::Rgb(133, 153, 0),     // Green
                info: Color::Rgb(42, 161, 152),       // Cyan
            },
        }
    }

    /// Wrapper function to check whether current theme is light theme
    pub fn is_light(&self) -> bool {
        self.palette().is_light()
    }

    /// Wrapper function to check whether current theme is light theme
    pub fn is_dark(&self) -> bool {
        self.palette().is_dark()
    }
}

/// A theme configuration wrapper providing convenient access to theme colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Theme {
    pub name: Selectable<ThemeName>,
}

impl Theme {
    /// Create a new theme with the given name
    pub fn new(name: ThemeName) -> Self {
        Self {
            name: Selectable::new(name),
        }
    }

    /// Gets name of a theme
    pub fn name(&self) -> ThemeName {
        *self.name
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

    /// Switch to the next theme
    pub fn next(&mut self) {
        self.name.next();
    }

    /// Switch to the prev theme
    pub fn prev(&mut self) {
        self.name.prev();
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
    use strum::IntoEnumIterator;

    #[test]
    fn should_check_if_all_themes_have_palettes() {
        for theme in ThemeName::iter() {
            let palette = theme.palette();
            assert_ne!(palette.fg, palette.bg);
        }
    }

    #[test]
    fn should_test_theme_cycling_next() {
        let mut theme = Theme::new(ThemeName::GruvboxDark);
        let original = theme.name.value;
        let count = ThemeName::iter().count();

        for _ in 0..count {
            theme.next();
        }

        assert_eq!(theme.name, original);
    }

    #[test]
    fn should_test_theme_cycling_prev() {
        let mut theme = Theme::new(ThemeName::GruvboxDark);
        let count = ThemeName::iter().count();

        theme.prev();
        assert_ne!(theme.name, ThemeName::GruvboxDark);

        theme.next();
        assert_eq!(theme.name, ThemeName::GruvboxDark);

        for _ in 0..count {
            theme.prev();
        }
        assert_eq!(theme.name, ThemeName::GruvboxDark);
    }

    #[test]
    fn should_verify_specific_transition() {
        let mut theme = Theme::new(ThemeName::GruvboxDark);
        theme.next();
        assert_eq!(theme.name, ThemeName::CatppuccinMocha);
    }

    #[test]
    fn should_test_light_dark_detection_for_themes() {
        assert!(ThemeName::GruvboxDark.palette().is_dark());
        assert!(ThemeName::CatppuccinMocha.palette().is_dark());
        assert!(ThemeName::TokyoNight.palette().is_dark());
    }

    #[test]
    fn should_test_to_string_for_theme() {
        assert_eq!(ThemeName::GruvboxDark.to_string(), "Gruvbox Dark");
        assert_eq!(ThemeName::TokyoNight.to_string(), "Tokyo Night");
        assert_eq!(ThemeName::CatppuccinMocha.to_string(), "Catppuccin Mocha");
    }

    #[test]
    fn should_test_theme_display_trait() {
        assert_eq!(format!("{}", ThemeName::GruvboxDark), "Gruvbox Dark");
        assert_eq!(format!("{}", ThemeName::TokyoNight), "Tokyo Night");
    }

    #[test]
    fn should_test_theme_from_name() {
        let theme: Theme = ThemeName::KanagawaWave.into();
        assert_eq!(theme.name, ThemeName::KanagawaWave);
    }
}
