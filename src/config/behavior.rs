use serde::{Deserialize, Serialize};

/// All config related to app/UI behavior
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct BehaviorConfig {
    pub confirm_before_remove: bool,
    pub confirm_before_save: bool,
    pub wrap_scrolling: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            confirm_before_remove: true,
            confirm_before_save: true,
            wrap_scrolling: true,
        }
    }
}
