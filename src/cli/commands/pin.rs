use crate::{Application, app::TaskService, cli::types::FindResult};

/// `pin` and `unpin` commands implementation
pub fn run(app: &mut Application, id_query: String, should_pin: bool) -> color_eyre::Result<()> {
    let found: FindResult = FindResult::find(&app.data.tasks, &id_query);

    match found {
        FindResult::Found(id) => {
            let is_already_pinned: bool =
                app.data.find_by_id(id).map(|t| t.pinned).unwrap_or(false);

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
        }

        FindResult::Ambiguous(list) => {
            eprintln!("ID '{}' is ambiguous. Matches:", id_query);
            for (id, title) in list {
                println!("  {} - {}", id, title);
            }
        }

        FindResult::NotFound => {
            eprintln!("Task with ID starting with '{}' not found.", id_query);
        }
    }

    Ok(())
}
