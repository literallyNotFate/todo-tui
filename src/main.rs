fn main() -> color_eyre::Result<()> {
    use clap::Parser;
    use toodles::{
        Application,
        cli::{self, Cli},
    };

    color_eyre::install()?;
    let cli: Cli = Cli::parse();

    let (config, config_error) = Application::load_config();
    toodles::core::init_logger(&config.log);

    log::info!(
        "Starting application (version: {})",
        env!("CARGO_PKG_VERSION")
    );
    if config_error.is_some() {
        log::warn!("Config not found or corrupted, using defaults");
    }

    let mut app: Application = Application::new(config, config_error);

    if let Some(theme) = cli.theme {
        app.ui.apply_theme_id(theme);
    }

    if let Some(command) = cli.command {
        return cli::run_cli(&mut app, command);
    }

    let terminal = ratatui::init();
    let result = app.run_tui(terminal);
    ratatui::restore();

    if let Err(ref e) = result {
        log::error!("Critical error: {:?}", e);
    }

    log::info!("Application exited successfully");
    result
}
