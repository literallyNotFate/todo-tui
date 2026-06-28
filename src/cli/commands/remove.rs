use crate::{
    Application,
    app::{OperationResult, TaskService},
    cli::types::FindResult,
};

/// `remove` command implementation
pub fn run(app: &mut Application, id_query: String) -> color_eyre::Result<()> {
    FindResult::find(&app.data.tasks, &id_query).execute(&id_query, |id| {
        let result = TaskService::remove_task(&mut app.data.tasks, &id)?;
        app.save_all()?;

        if let OperationResult::TaskRemoved { task } = result {
            println!(
                "Task '{}' (ID: {}) has been permanently removed!",
                task.title,
                &id.to_string()[..8]
            );
        }

        Ok(())
    })
}
