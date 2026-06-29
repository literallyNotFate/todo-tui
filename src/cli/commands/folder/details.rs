use crate::{Application, cli::types::EntitySelector, models::Task};

/// `details` command implementation for folder subcommand
pub fn run(app: &Application, id_query: String) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.folders, &id_query).execute(&id_query, |folder| {
        println!("{} [{}]", folder.name, folder.color);
        println!("ID: {}", folder.id);
        println!("Created: {}", folder.created_at);

        let folder_tasks: Vec<&Task> = app
            .data
            .tasks
            .iter()
            .filter(|t| t.folder_id == Some(folder.id))
            .collect();
        println!("Tasks in folder: {}", folder_tasks.len());

        for t in folder_tasks {
            println!("  - {}", t.title);
        }

        Ok(())
    })
}
