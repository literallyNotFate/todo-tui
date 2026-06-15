pub mod config;
pub mod keymaps;

pub use config::Config;
pub use keymaps::KeyMaps;

use crate::{
    common::{is_default, is_default_dark, is_default_light},
    theme::{ThemeID, ThemeName},
};
use serde::{Deserialize, Serialize};
use simplelog::LevelFilter;

/// All config related to app/UI behavior
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct BehaviorConfig {
    pub confirm_before_remove: bool,
    pub confirm_before_save: bool,
    pub wrap_scrolling: bool,
    pub case_insensitive_search: bool,
    pub show_empty_folders: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            confirm_before_remove: true,
            confirm_before_save: true,
            wrap_scrolling: true,
            case_insensitive_search: true,
            show_empty_folders: true,
        }
    }
}

/// Configuration for logger
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LogConfig {
    pub enabled: bool,
    pub level: LogLevel,
}

/// Levels for log
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Debug,
    Info,
    Warn,
    Error,
    Trace,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// Configuration related to storage (autosave)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct StorageConfig {
    pub autosave_enabled: bool,
    pub autosave_interval: u64,
    pub backup_enabled: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            autosave_enabled: false,
            autosave_interval: 30,
            backup_enabled: true,
        }
    }
}

impl StorageConfig {
    pub fn safe_interval(&self) -> u64 {
        self.autosave_interval.clamp(5, 3600)
    }
}

/// All configuration related to UI
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UIConfig {
    pub theme: ThemeID,
    pub use_system_theme: bool,

    #[serde(skip_serializing_if = "is_default_dark")]
    pub last_dark: Option<ThemeID>,
    #[serde(skip_serializing_if = "is_default_light")]
    pub last_light: Option<ThemeID>,

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
            theme: ThemeID::Builtin(ThemeName::default()),
            use_system_theme: true,
            last_dark: Some(ThemeID::Builtin(ThemeName::default())),
            last_light: Some(ThemeID::Builtin(ThemeName::GruvboxLight)),
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
