mod app;

use color_eyre::Result;
use ratatui::DefaultTerminal;

use app::application::Application;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app: Application = Application::new();
    let terminal: DefaultTerminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
