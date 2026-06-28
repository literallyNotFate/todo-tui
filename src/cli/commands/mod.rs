pub mod add;
pub mod list;
pub mod pin;
pub mod update;

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
    #[command(alias = "a")]
    Add {
        /// Title of the task
        title: String,

        /// Priority of the task
        #[arg(short, long, value_enum, default_value_t = Priority::Low)]
        priority: Priority,
    },

    /// Update selected task by ID
    #[command(alias = "u")]
    Update {
        /// UUID of a task
        id: String,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description
        #[arg(short, long)]
        desc: Option<String>,

        /// New priority
        #[arg(short, long)]
        priority: Option<Priority>,
    },
    /// Shows the list of all tasks
    #[command(alias = "ls")]
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

    /// Pin a task
    Pin {
        /// UUID of a task
        id: String,
    },

    /// Unpin a task
    Unpin {
        /// UUID of a task
        id: String,
    },
}
