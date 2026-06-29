use crate::{
    Application,
    app::{OperationResult, TaskService},
    cli::types::EntitySelector,
};

/// `remove` command implementation for task subcommand
pub fn run(app: &mut Application, id_query: String) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.tasks, &id_query).execute(&id_query, |task| {
        let result = TaskService::remove_task(&mut app.data.tasks, &task.id)?;
        app.save_all()?;

        if let OperationResult::TaskRemoved { task } = result {
            println!(
                "Task '{}' (ID: {}) has been permanently removed!",
                task.title,
                &task.id.to_string()[..8]
            );
        }

        Ok(())
    })
}
