mod app;
mod handlers;
mod models;
mod state;
mod ui;
mod utils;

use color_eyre::Result;

fn main() -> Result<()> {
    use app::Application;
    use ratatui::DefaultTerminal;

    color_eyre::install()?;

    let mut app: Application = Application::new();
    let terminal: DefaultTerminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
