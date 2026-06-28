use crate::{
    Application,
    cli::commands::{add, details, list, pin, remove, update},
    core::{ApplicationError, Sort},
};

pub mod commands;
pub mod types;

pub use commands::{Cli, Commands};

/// Run toodles CLI
pub fn run_cli(app: &mut Application, command: Commands) -> color_eyre::Result<()> {
    log::info!("Run CLI: Entering CLI mode");

    let result = match command {
        Commands::List {
            filter,
            limit,
            query,
            sort_by,
            order,
        } => {
            let sort: Sort = Sort::new(sort_by, order);
            list::run(&app.data.tasks, filter, limit, query, sort)
        }
        Commands::Add { title, priority } => add::run(app, title, priority),
        Commands::Pin { id } => pin::run(app, id, true),
        Commands::Unpin { id } => pin::run(app, id, false),
        Commands::Update {
            id,
            title,
            desc,
            priority,
        } => update::run(app, id, title, desc, priority),
        Commands::Rm { id } => remove::run(app, id),
        Commands::Details { id } => details::run(app, id),
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
