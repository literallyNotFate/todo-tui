pub mod add;
pub mod list;

use crate::{
    cli::types::FilterMode,
    core::{SortBy, SortOrder},
    models::Priority,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "toodles")]
#[command(version = "1.0")]
#[command(about = "CLI version of Toodles TUI application", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds task to the list
    Add {
        /// Title of the task
        title: String,

        /// Priority of the task
        #[arg(short, long, value_enum, default_value_t = Priority::Low)]
        priority: Priority,
    },

    /// Shows the list of all tasks
    List {
        /// Filter mode
        #[arg(short, long, value_enum, default_value_t = FilterMode::All)]
        filter: FilterMode,

        /// Sort tasks by specific parameter
        #[arg(long, value_enum, default_value_t = SortBy::Priority)]
        sort_by: SortBy,

        /// Sorting order
        #[arg(long, value_enum, default_value_t = SortOrder::Desc)]
        order: SortOrder,

        /// Task search query
        #[arg(short, long, value_name = "QUERY")]
        query: Option<String>,

        /// Limit the output
        #[arg(short, long, value_name = "LIMIT")]
        limit: Option<usize>,
    },
}
