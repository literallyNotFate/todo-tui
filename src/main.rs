fn main() -> color_eyre::Result<()> {
    use toodles::Application;
    color_eyre::install()?;

    let (config, config_error) = Application::load_config();
    toodles::core::init_logger(&config.log);

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
