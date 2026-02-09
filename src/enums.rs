/// Which area of menu is being selected
#[derive(Default, PartialEq, Debug)]
pub enum FocusArea {
    #[default]
    LeftPanel, // Sidebar
    MainContent, // Task list/form
}

/// Which mode user selects (for input handling)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Browsing,
    List,
    Form,
    Search,
}

/// What is being returned from handle_key() widget function
#[derive(Debug, PartialEq)]
pub enum WidgetResponse {
    Continue,
    Submit,
    Cancel,
}
