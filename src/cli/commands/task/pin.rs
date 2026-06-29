use crate::{Application, app::TaskService, cli::types::EntitySelector};

/// `pin` and `unpin` commands implementation for task subcommand
pub fn run(app: &mut Application, id_query: String, should_pin: bool) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.tasks, &id_query).execute(&id_query, |task| {
        let is_already_pinned: bool = task.pinned == true;

        if is_already_pinned == should_pin {
            println!(
                "Task is already {}",
                if should_pin { "pinned" } else { "unpinned" }
            );
            return Ok(());
        }

        TaskService::toggle_pinned(&mut app.data.tasks, &task.id)?;
        app.save_all()?;

        println!(
            "Task has been {}!",
            if should_pin { "pinned" } else { "unpinned" }
        );
        Ok(())
    })
}
