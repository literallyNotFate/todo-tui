pub mod folder;
pub mod task;

use crate::{
    cli::types::FilterMode,
    core::{SortBy, SortOrder},
    models::{FolderColor, Priority},
    theme::ThemeId,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "toodles")]
#[command(version = "1.0")]
#[command(about = "CLI version of Toodles TUI application", long_about = None)]
pub struct Cli {
    /// Theme for TUI application (ignored if running CLI commands)
    #[arg(long)]
    pub theme: Option<ThemeId>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage tasks
    #[command(subcommand)]
    Task(TaskCommands),

    /// Manage folders
    #[command(subcommand)]
    Folder(FolderCommands),
}

#[derive(Subcommand)]
pub enum TaskCommands {
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

    /// Remove selected task by ID
    #[command(alias = "d")]
    Rm {
        /// UUID of a task
        id: String,
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

    /// Show detailed info about a task
    #[command(alias = "det")]
    Details {
        /// ID of the task
        id: String,
    },
}

#[derive(Subcommand)]
pub enum FolderCommands {
    /// Adds folder to the list
    #[command(alias = "a")]
    Add {
        /// Name of the folder
        name: String,

        /// Color of a folder
        #[arg(short, long, value_enum, default_value_t = FolderColor::Neutral)]
        color: FolderColor,
    },

    /// Update selected folder by ID
    #[command(alias = "u")]
    Update {
        /// UUID of a folder
        id: String,

        /// New name
        #[arg(short, long)]
        name: Option<String>,

        /// New priority
        #[arg(short, long)]
        color: Option<FolderColor>,
    },

    /// Remove selected folder by ID
    #[command(alias = "d")]
    Rm {
        /// UUID of a folder
        id: String,
    },

    /// Shows the list of all folders
    #[command(alias = "ls")]
    List,

    /// Show detailed info about a folder
    #[command(alias = "det")]
    Details {
        /// ID of the folder
        id: String,
    },
}
