use crate::{
    Application,
    app::TaskService,
    models::{Priority, Task},
};

/// `add` command implementation
pub fn run(app: &mut Application, title: String, priority: Priority) -> color_eyre::Result<()> {
    let new_task: Task = Task::new(title).with_priority(priority);
    let result = TaskService::append_task(&mut app.data.tasks, new_task)?;

    app.save_all()?;
    let added: Task = result.unwrap_task_created();
    println!(
        "Task `{}` with {} priority was added and saved successfully!",
        added.title, added.priority
    );

    Ok(())
}
