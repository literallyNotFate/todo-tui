use crate::theme::{Theme, ThemeId, ThemePalette};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// System themes for application
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::Display,
    strum::EnumIter,
    clap::ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
#[strum(serialize_all = "title_case")]
pub enum SystemTheme {
    #[serde(rename = "tty")]
    #[strum(to_string = "TTY")]
    Tty,

    #[serde(rename = "no-color")]
    #[strum(to_string = "No Color")]
    NoColor,
}

impl SystemTheme {
    /// Returns the color palette for this theme.
    pub const fn palette(self) -> ThemePalette {
        match self {
            Self::Tty => ThemePalette {
                accent: Color::Yellow,
                secondary: Color::Magenta,
                bg: Color::Reset,
                bg2: Color::Indexed(0),
                fg: Color::Reset,
                muted: Color::Indexed(8),
                selection: Color::Indexed(4),
                error: Color::Red,
                warning: Color::Yellow,
                success: Color::Green,
                info: Color::Cyan,
            },
            Self::NoColor => ThemePalette {
                accent: Color::Reset,
                secondary: Color::Reset,
                bg: Color::Reset,
                bg2: Color::Reset,
                fg: Color::Reset,
                muted: Color::DarkGray,
                selection: Color::Reset,
                error: Color::Reset,
                warning: Color::Reset,
                success: Color::Reset,
                info: Color::Reset,
            },
        }
    }
}

impl From<SystemTheme> for Theme {
    fn from(theme: SystemTheme) -> Self {
        Self::new(ThemeId::System(theme))
    }
}
