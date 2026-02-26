use serde::{Deserialize, Serialize};
use simplelog::LevelFilter;

/// Configuration for logger
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogConfig {
    pub enabled: bool,
    pub level: LogLevel,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: LogLevel::Info,
        }
    }
}

/// Levels for log
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
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
