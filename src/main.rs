mod app;

use color_eyre::Result;

fn main() -> Result<()> {
    use app::application::Application;
    use ratatui::DefaultTerminal;

    color_eyre::install()?;

    let mut app: Application = Application::new();
    let terminal: DefaultTerminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
