use serde::{Deserialize, Serialize};

/// Configuration related to storage (autosave)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct StorageConfig {
    pub autosave_enabled: bool,
    pub autosave_interval: u64,
    pub backup_enabled: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            autosave_enabled: false,
            autosave_interval: 30,
            backup_enabled: true,
        }
    }
}

impl StorageConfig {
    pub fn safe_interval(&self) -> u64 {
        self.autosave_interval.clamp(5, 3600)
    }
}
