use crate::{
    common::{is_default, is_default_dark, is_default_light},
    theme::{BuiltinTheme, ThemeId},
};
use ratatui::widgets::BorderType;
use serde::{Deserialize, Serialize};

/// All configuration related to UI
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct UIConfig {
    pub theme: ThemeId,
    pub use_system_theme: bool,

    #[serde(skip_serializing_if = "is_default_dark")]
    pub last_dark: Option<ThemeId>,
    #[serde(skip_serializing_if = "is_default_light")]
    pub last_light: Option<ThemeId>,

    pub show_sidebar: bool,
    pub show_footer: bool,
    pub sidebar_width: u16,

    #[serde(skip_serializing_if = "is_default")]
    pub border_type: BorderTypeConfig,
    #[serde(skip_serializing_if = "is_default")]
    pub symbols: SymbolsConfig,

    pub date_format: String,
    pub use_24h: bool,
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
    pub pinned: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            theme: ThemeId::Builtin(BuiltinTheme::default()),
            use_system_theme: true,
            last_dark: Some(ThemeId::Builtin(BuiltinTheme::default())),
            last_light: Some(ThemeId::Builtin(BuiltinTheme::GruvboxLight)),
            show_sidebar: true,
            show_footer: true,
            sidebar_width: 30,
            border_type: BorderTypeConfig::default(),
            symbols: SymbolsConfig::default(),
            date_format: String::from("%d/%m/%Y"),
            use_24h: true,
        }
    }
}

impl Default for SymbolsConfig {
    fn default() -> Self {
        Self {
            completed: "✓".to_string(),
            pending: "☐".to_string(),
            selection: "❯".to_string(),
            pinned: "󰐃".to_string(),
        }
    }
}

/// Casting to ratatui BorderType with .into()
impl From<BorderTypeConfig> for BorderType {
    fn from(config: BorderTypeConfig) -> Self {
        match config {
            BorderTypeConfig::Plain => Self::Plain,
            BorderTypeConfig::Rounded => Self::Rounded,
            BorderTypeConfig::Double => Self::Double,
            BorderTypeConfig::Thick => Self::Thick,
        }
    }
}

impl UIConfig {
    /// Method to get ratatui border
    pub fn get_border_type(&self) -> BorderType {
        self.border_type.into()
    }

    /// Validates UI config
    pub fn validate(&mut self) {
        self.sidebar_width = self.sidebar_width.clamp(10, 80);

        if self.date_format.is_empty() {
            self.date_format = "%d/%m/%Y".to_string();
        }
    }
}

/// Unit-tests for UI config
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_ui_config_validation() {
        let mut config = UIConfig::default();
        config.validate();
        assert!(config.sidebar_width >= 10);

        config.sidebar_width = 2;
        config.validate();
        assert_eq!(config.sidebar_width, 10);

        config.date_format = "".to_string();
        config.validate();
        assert_eq!(config.date_format, "%d/%m/%Y");
    }

    #[test]
    fn should_test_border_type_conversion() {
        use BorderType as RatatuiBorder;

        let border = BorderTypeConfig::Double;
        let converted: RatatuiBorder = border.into();
        assert_eq!(converted, RatatuiBorder::Double);
    }
}
