use crate::{
    Application,
    app::{OperationResult, TaskService},
    cli::types::FindResult,
    core::Selectable,
    models::{Priority, TaskEditor},
};

/// `update` command implementation
pub fn run(
    app: &mut Application,
    id_query: String,
    title: Option<String>,
    desc: Option<String>,
    priority: Option<Priority>,
) -> color_eyre::Result<()> {
    FindResult::find(&app.data.tasks, &id_query).execute(&id_query, |id| {
        let current_task = app.data.find_by_id(id).unwrap();

        let editor: TaskEditor = TaskEditor {
            title: title.clone().unwrap_or_else(|| current_task.title.clone()),
            description: desc
                .clone()
                .unwrap_or_else(|| current_task.description.clone()),
            priority: Selectable::new(priority.unwrap_or_else(|| current_task.priority.clone())),
            folder_id: current_task.folder_id,
        };

        let result = TaskService::update_task(&mut app.data.tasks, &id, editor)?;
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
