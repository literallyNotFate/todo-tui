use crate::{
    Application,
    cli::commands::{FolderCommands, TaskCommands, folder, task},
    core::{ApplicationError, Sort},
};

pub mod commands;
pub mod types;

pub use commands::{Cli, Commands};

/// Run toodles CLI
pub fn run_cli(app: &mut Application, command: Commands) -> color_eyre::Result<()> {
    log::info!("Run CLI: Entering CLI mode");

    let result = match command {
        Commands::Task(task_cmd) => match task_cmd {
            TaskCommands::List {
                filter,
                limit,
                query,
                sort_by,
                order,
            } => {
                let sort = Sort::new(sort_by, order);
                task::list::run(&app.data.tasks, filter, limit, query, sort)
            }
            TaskCommands::Add { title, priority } => task::add::run(app, title, priority),
            TaskCommands::Pin { id } => task::pin::run(app, id, true),
            TaskCommands::Unpin { id } => task::pin::run(app, id, false),
            TaskCommands::Update {
                id,
                title,
                desc,
                priority,
            } => task::update::run(app, id, title, desc, priority),
            TaskCommands::Rm { id } => task::remove::run(app, id),
            TaskCommands::Details { id } => task::details::run(app, id),
        },
        Commands::Folder(folder_cmd) => match folder_cmd {
            FolderCommands::Add { name, color } => folder::add::run(app, name, color),
            FolderCommands::List => folder::list::run(app),
            FolderCommands::Rm { id } => folder::remove::run(app, id),
            FolderCommands::Details { id } => folder::details::run(app, id),
            FolderCommands::Update { id, name, color } => folder::update::run(app, id, name, color),
        },
    };

    if let Err(e) = result {
        if let Some(app_err) = e.downcast_ref::<ApplicationError>() {
            eprintln!("CLI error occurred: {}", app_err);
        } else {
            return Err(e);
        }
    }

    Ok(())
}
