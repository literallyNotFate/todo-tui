use serde::{Deserialize, Serialize};

/// Which mode user selects (for input handling)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Navigation,
    List,
    Search,
}

/// Which area of menu is being selected
#[derive(
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Debug,
    Clone,
    Copy,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
)]
pub enum FocusArea {
    #[default]
    Sidebar,
    Main,
}
