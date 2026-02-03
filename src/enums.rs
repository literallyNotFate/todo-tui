// Basically shows which area of menu is being selected
#[derive(Default, PartialEq, Debug)]
pub enum FocusArea {
    #[default]
    LeftPanel, // Sidebar
    MainContent, // Task list/form
}

// Made for key handling (handles specific controls for each modes)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Browsing,
    List,
    Form,
    Search,
}

// What is being return from handle_key widget function
#[derive(Debug, PartialEq)]
pub enum WidgetResponse {
    Continue,
    Submit,
    Cancel,
}
