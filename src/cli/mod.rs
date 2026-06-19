pub mod list;

pub use list::list_tasks;

use crate::state::SidebarTab;
use clap::{Parser, Subcommand, ValueEnum};

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
    /// Shows the list of all tasks
    List {
        /// Filter mode
        #[arg(short, long, value_enum, default_value_t = FilterMode::All)]
        filter: FilterMode,

        /// Task search query
        #[arg(short, long, value_name = "QUERY")]
        query: Option<String>,

        /// Limit the output
        #[arg(short, long, value_name = "LIMIT")]
        limit: Option<usize>,
    },
}

/// Enum to filter tasks via CLI
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum FilterMode {
    Active,
    Completed,
    High,
    Today,
    All,
}

impl From<FilterMode> for SidebarTab {
    fn from(filter: FilterMode) -> Self {
        match filter {
            FilterMode::Active => SidebarTab::Active,
            FilterMode::Completed => SidebarTab::Completed,
            FilterMode::High => SidebarTab::HighPriority,
            FilterMode::All => SidebarTab::Inbox,
            FilterMode::Today => SidebarTab::Today,
        }
    }
}
