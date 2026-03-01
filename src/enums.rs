use serde::{Deserialize, Serialize};

/// Which area of menu is being selected
#[derive(Default, Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum FocusArea {
    #[default]
    LeftPanel, // Sidebar
    MainContent, // Task list/form
}

/// What is being returned from handle_key() widget function
#[derive(Debug, PartialEq)]
pub enum WidgetResponse {
    Continue,
    Submit,
    Cancel,
}
