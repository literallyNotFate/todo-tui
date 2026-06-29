use crate::Application;
use comfy_table::{Cell, Color, Table};

/// `list` command implementation for folder subcommand
pub fn run(app: &Application) -> color_eyre::Result<()> {
    let mut table: Table = Table::new();
    table.set_header(vec!["ID", "Name", "Color"]);

    for f in &app.data.folders {
        table.add_row(vec![
            Cell::new(f.id.to_string()).fg(Color::Grey),
            Cell::new(f.name.clone()),
            Cell::new(f.color.to_string()),
        ]);
    }

    println!("{table}");
    Ok(())
}
