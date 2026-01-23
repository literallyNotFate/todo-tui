use color_eyre::Result;

fn main() -> Result<()> {
    use ratatui::DefaultTerminal;
    use todo_tui::Application;

    color_eyre::install()?;

    let mut app: Application = Application::new();
    let terminal: DefaultTerminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
