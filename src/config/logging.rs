use serde::{Deserialize, Serialize};
use simplelog::LevelFilter;

/// Configuration for logger
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct LogConfig {
    pub enabled: bool,
    pub level: LogLevel,
}

impl LogConfig {
    /// Checks whether log is active
    pub fn is_active(&self) -> bool {
        self.enabled
    }

    /// Returns log level in simplelog format
    pub fn level_filter(&self) -> LevelFilter {
        if !self.enabled {
            LevelFilter::Off
        } else {
            self.level.into()
        }
    }
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

/// Unit-tests for log config
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_log_level_mapping() {
        assert_eq!(LevelFilter::from(LogLevel::Debug), LevelFilter::Debug);
        assert_eq!(LevelFilter::from(LogLevel::Error), LevelFilter::Error);
    }
}
