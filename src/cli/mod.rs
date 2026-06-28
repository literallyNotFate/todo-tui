use crate::{
    Application,
    cli::commands::{add, list},
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
