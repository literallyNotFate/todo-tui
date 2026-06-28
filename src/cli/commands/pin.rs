use crate::{Application, app::TaskService, cli::types::FindResult};

/// `pin` and `unpin` commands implementation
pub fn run(app: &mut Application, id_query: String, should_pin: bool) -> color_eyre::Result<()> {
    FindResult::find(&app.data.tasks, &id_query).execute(&id_query, |id| {
        let is_already_pinned: bool = app.data.find_by_id(id).map(|t| t.pinned).unwrap_or(false);

        if is_already_pinned == should_pin {
            println!(
                "Task is already {}",
                if should_pin { "pinned" } else { "unpinned" }
            );
            return Ok(());
        }

        TaskService::toggle_pinned(&mut app.data.tasks, &id)?;
        app.save_all()?;

        println!(
            "Task has been {}!",
            if should_pin { "pinned" } else { "unpinned" }
        );
        Ok(())
    })
}
