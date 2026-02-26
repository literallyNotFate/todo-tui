use crate::config::LogLevel;
use simplelog::*;
use std::{fs::File, path::Path};

/// Initialize logger with selected log filter from config
pub fn init_logger(path: &Path, level: LogLevel) {
    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_level_padding(LevelPadding::Right)
        .build();

    if let Ok(file) = File::create(path) {
        let _ = CombinedLogger::init(vec![WriteLogger::new(level.into(), config, file)]);
    }

    log::info!(
        "Logger for 'todo-tui' initialized on {} {} with log level: {:?}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        level
    );
}
