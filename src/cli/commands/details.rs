use crate::{Application, cli::types::FindResult};
use comfy_table::{Cell, Color, Table, presets};

/// `details` command implementation
pub fn run(app: &Application, id_query: String) -> color_eyre::Result<()> {
    FindResult::find(&app.data.tasks, &id_query).execute(&id_query, |id| {
        let task = app.data.find_by_id(id).unwrap();

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);

        table.set_header(vec![
            Cell::new("Field").fg(Color::Grey),
            Cell::new("Value").fg(Color::Grey),
        ]);

        table.add_row(vec!["ID", &task.id.to_string()]);
        table.add_row(vec!["Title", &task.title]);
        table.add_row(vec!["Priority", &format!("{:?}", task.priority)]);
        table.add_row(vec!["Pinned", if task.pinned { "Yes" } else { "No" }]);
        table.add_row(vec![
            "Folder",
            &task
                .folder_id
                .map(|f| f.to_string())
                .unwrap_or_else(|| "None".to_string()),
        ]);
        table.add_row(vec!["Created", &task.time_ago()]);

        if !task.description.is_empty() {
            table.add_row(vec!["Description", &task.description]);
        }

        println!("\nTask Details:");
        println!("{table}");

        Ok(())
    })
}
