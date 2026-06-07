fn main() -> color_eyre::Result<()> {
    use toodles::{Application, core::Storage};

    color_eyre::install()?;
    let (config, config_error) = Application::load_config();

    if config.log.enabled {
        if let Ok(log_path) = Storage::get_log_path() {
            if let Some(parent) = log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            toodles::core::init_logger(&log_path, config.log.level);
        }
    }

    log::info!(
        "Starting application (version: {})",
        env!("CARGO_PKG_VERSION")
    );
    if config_error.is_some() {
        log::warn!("Config not found or corrupted, using defaults");
    }

    let mut app = Application::new(config, config_error);
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    if let Err(ref e) = result {
        log::error!("Critical error: {:?}", e);
    }

    log::info!("Application exited successfully");
    result
}
