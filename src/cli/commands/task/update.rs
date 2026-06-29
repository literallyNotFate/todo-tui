use crate::{
    Application,
    app::{OperationResult, TaskService},
    cli::types::EntitySelector,
    core::Selectable,
    models::{Priority, TaskEditor},
};

/// `update` command implementation for task subcommand
pub fn run(
    app: &mut Application,
    id_query: String,
    title: Option<String>,
    desc: Option<String>,
    priority: Option<Priority>,
) -> color_eyre::Result<()> {
    EntitySelector::find(&app.data.tasks, &id_query).execute(&id_query, |task| {
        let editor: TaskEditor = TaskEditor {
            title: title.clone().unwrap_or_else(|| task.title.clone()),
            description: desc.clone().unwrap_or_else(|| task.description.clone()),
            priority: Selectable::new(priority.unwrap_or_else(|| task.priority.clone())),
            folder_id: task.folder_id,
        };

        let result = TaskService::update_task(&mut app.data.tasks, &task.id, editor)?;
        app.save_all()?;

        if let OperationResult::TaskUpdated { old, new } = result {
            println!("Task updated:");
            if old.title != new.title {
                println!("   Title: '{}' -> '{}'", old.title, new.title);
            }
            if old.description != new.description {
                println!("   Desc:  '{}' -> '{}'", old.description, new.description);
            }
            if old.priority != new.priority {
                println!("   Prior: {:?} -> {:?}", old.priority, new.priority);
            }
        }

        Ok(())
    })
}
