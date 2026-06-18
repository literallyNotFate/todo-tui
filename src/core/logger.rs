use crate::{config::LogConfig, core::Storage};
use simplelog::{CombinedLogger, Config, ConfigBuilder, LevelPadding, WriteLogger};
use std::fs::{self, File};

/// Initialize logger with selected log filter from config
pub fn init_logger(config: &LogConfig) {
    if !config.is_active() {
        return;
    }

    if let Ok(log_path) = Storage::get_log_path() {
        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let log_config: Config = ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_level_padding(LevelPadding::Right)
            .build();

        if let Ok(file) = File::create(log_path) {
            let _ = CombinedLogger::init(vec![WriteLogger::new(
                config.level_filter(),
                log_config,
                file,
            )]);
        }
    }

    log::info!(
        "Logger for 'toodles' initialized on {} {} with log level: {:?}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        config.level
    );
}
