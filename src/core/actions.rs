use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoStaticStr};

#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    EnumIter,
    Display,
    IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Quit,
    Save,

    #[strum(serialize = "help")]
    #[serde(rename = "help")]
    ShowHelp,

    #[strum(serialize = "autosave_toggle")]
    #[serde(rename = "autosave_toggle")]
    ToggleAutosave,

    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveTaskUp,
    MoveTaskDown,

    FilterInbox,
    FilterActive,
    FilterCompleted,
    FilterHigh,
    FilterToday,

    AddTask,
    UpdateTask,
    RemoveTask,
    Complete,
    ShowDetails,
    Search,
    Pin,
    Clear,

    AddFolder,
    UpdateFolder,
    RemoveFolder,

    NextTheme,
    PrevTheme,
    ToggleThemeMode,

    #[strum(serialize = "sidebar_toggle")]
    #[serde(rename = "sidebar_toggle")]
    ToggleSidebar,

    Sort,
    SortReverse,
}

impl Action {
    /// Returns description and section for Help UI
    pub fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::Quit => ("Quit application", "System"),
            Self::Save => ("Save changes", "System"),
            Self::ShowHelp => ("Show help", "System"),
            Self::ToggleAutosave => ("Toggle autosave", "System"),

            Self::MoveLeft => ("Focus sidebar", "Navigation"),
            Self::MoveRight => ("Focus list", "Navigation"),
            Self::MoveUp => ("Move selection up", "Navigation"),
            Self::MoveDown => ("Move selection down", "Navigation"),
            Self::MoveTaskUp => ("Move task up in list", "Navigation"),
            Self::MoveTaskDown => ("Move task down in list", "Navigation"),

            Self::FilterInbox => ("Filter: Inbox", "Filters"),
            Self::FilterActive => ("Filter: Active", "Filters"),
            Self::FilterCompleted => ("Filter: Completed", "Filters"),
            Self::FilterHigh => ("Filter: High Priority", "Filters"),
            Self::FilterToday => ("Filter: Today", "Filters"),

            Self::AddTask => ("Add new task", "Actions"),
            Self::UpdateTask => ("Update selected task", "Actions"),
            Self::RemoveTask => ("Remove selected task", "Actions"),
            Self::Complete => ("Mark task as Done/Undone", "Actions"),
            Self::ShowDetails => ("Show task details", "Actions"),
            Self::Search => ("Search tasks", "Actions"),
            Self::Pin => ("Pin selected task", "Actions"),
            Self::Clear => ("Clear all tasks", "Actions"),

            Self::AddFolder => ("Add new folder", "Actions"),
            Self::UpdateFolder => ("Update selected folder", "Actions"),
            Self::RemoveFolder => ("Remove selected folder", "Actions"),

            Self::NextTheme => ("Next theme", "UI"),
            Self::PrevTheme => ("Previous theme", "UI"),
            Self::ToggleThemeMode => ("Light/Dark mode", "UI"),
            Self::ToggleSidebar => ("Show/Hide sidebar", "UI"),
            Self::Sort => ("Sort list", "UI"),
            Self::SortReverse => ("Reverse sorting", "UI"),
        }
    }
}
