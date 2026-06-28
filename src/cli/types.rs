use crate::{models::Task, state::SidebarTab};
/// Enum to filter tasks via CLI (list command)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
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
