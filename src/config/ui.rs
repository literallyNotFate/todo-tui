use super::utils::{is_default, is_default_dark, is_default_light};
use crate::theme::ThemeName;
use serde::{Deserialize, Serialize};

/// All configuration related to UI
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UIConfig {
    pub theme: ThemeName,
    pub use_system_theme: bool,

    #[serde(skip_serializing_if = "is_default_dark")]
    pub last_dark: Option<ThemeName>,
    #[serde(skip_serializing_if = "is_default_light")]
    pub last_light: Option<ThemeName>,

    pub show_sidebar: bool,

    #[serde(skip_serializing_if = "is_default")]
    pub border_type: BorderTypeConfig,
    #[serde(skip_serializing_if = "is_default")]
    pub symbols: SymbolsConfig,
}

/// Border configuration
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BorderTypeConfig {
    Plain,
    #[default]
    Rounded,
    Double,
    Thick,
}

/// Symbols configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SymbolsConfig {
    pub completed: String,
    pub pending: String,
    pub selection: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: ThemeName::default(),
            use_system_theme: true,
            last_dark: Some(ThemeName::default()),
            last_light: Some(ThemeName::GruvboxLight),
            show_sidebar: true,
            border_type: BorderTypeConfig::default(),
            symbols: SymbolsConfig::default(),
        }
    }
}

impl Default for SymbolsConfig {
    fn default() -> Self {
        Self {
            completed: "✓".to_string(),
            pending: "☐".to_string(),
            selection: "❯".to_string(),
        }
    }
}

/// Casting to ratatui BorderType with .into()
impl From<BorderTypeConfig> for ratatui::widgets::BorderType {
    fn from(config: BorderTypeConfig) -> Self {
        match config {
            BorderTypeConfig::Plain => Self::Plain,
            BorderTypeConfig::Rounded => Self::Rounded,
            BorderTypeConfig::Double => Self::Double,
            BorderTypeConfig::Thick => Self::Thick,
        }
    }
}
