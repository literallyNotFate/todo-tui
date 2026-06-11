pub mod actions;
pub mod autosave;
pub mod errors;
pub mod sorting;
pub mod storage;

pub use actions::Action;
pub use autosave::Autosave;
pub use errors::{ApplicationError, FolderError, KeyMapError, StorageError, TaskError};
pub use sorting::{Sort, SortBy, SortOrder};
pub use storage::{SessionRepository, Storage, TaskRepository};

use crate::config::LogLevel;
use simplelog::*;
use std::{fs::File, ops::Deref, path::Path};

/// Which mode user selects (for input handling)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Navigation,
    List,
    Search,
}

use serde::{Deserialize, Serialize};

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

/// Initialize logger with selected log filter from config
pub fn init_logger(path: &Path, level: LogLevel) {
    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_level_padding(LevelPadding::Right)
        .build();

    if let Ok(file) = File::create(path) {
        let _ = CombinedLogger::init(vec![WriteLogger::new(level.into(), config, file)]);
    }

    log::info!(
        "Logger for 'toodles' initialized on {} {} with log level: {:?}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        level
    );
}

use std::fmt::Display;
use strum::IntoEnumIterator;

/// Container-wrapper for data that can be switched (like enums)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Selectable<T> {
    pub value: T,
}

impl<T> Selectable<T>
where
    T: IntoEnumIterator + Copy + PartialEq + Default + 'static,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Returns current selected index
    pub fn index(&self) -> usize {
        T::iter().position(|t| t == self.value).unwrap_or(0)
    }

    /// Count total elements of enum
    pub fn count(&self) -> usize {
        T::iter().count()
    }

    /// Set specific enum value to selectable
    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    /// Switch to the next element
    pub fn next(&mut self) {
        let mut iter = T::iter();
        let pos = iter.position(|t| t == self.value).unwrap_or(0);
        self.value = T::iter()
            .nth(pos + 1)
            .unwrap_or_else(|| T::iter().next().unwrap());
    }

    /// Switch to the prev element
    pub fn prev(&mut self) {
        let pos = T::iter().position(|t| t == self.value).unwrap_or(0);
        if pos == 0 {
            self.value = T::iter().last().unwrap();
        } else {
            self.value = T::iter().nth(pos - 1).unwrap();
        }
    }
}

impl<T> Deref for Selectable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: PartialEq> PartialEq<T> for Selectable<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: PartialEq> PartialEq for Selectable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Selectable<T> {}

impl<T: Default> Default for Selectable<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: Display> Display for Selectable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
