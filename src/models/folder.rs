use crate::{core::FolderError, state::ApplicationResult};
use chrono::{DateTime, Utc};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Folder of tasks
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub color: FolderColor,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name_lower: String,
}

impl Folder {
    pub fn new(name: impl Into<String>, color: FolderColor) -> Self {
        let name_str: String = name.into();
        let name_lower: String = name_str.to_lowercase();
        let now: DateTime<Utc> = Utc::now();

        Self {
            id: Uuid::new_v4(),
            name: name_str,
            color,
            created_at: now,
            updated_at: now,
            name_lower,
        }
    }

    /// Helper to update name and color
    pub fn update(&mut self, name: impl Into<String>, color: FolderColor) {
        let name_str = name.into();
        self.name_lower = name_str.to_lowercase();
        self.name = name_str;
        self.color = color;
        self.updated_at = Utc::now();
    }

    /// Update folder from FolderEditor
    pub fn update_from(&mut self, editor: FolderEditor) {
        self.name = editor.name;
        self.color = editor.color;
        self.name_lower = self.name.to_lowercase();
        self.updated_at = Utc::now();
    }

    /// Validate folder
    pub fn validate(
        name: &str,
        exclude_id: Option<Uuid>,
        folders: &[Folder],
    ) -> ApplicationResult<()> {
        let name: &str = name.trim();
        if name.is_empty() {
            return Err(FolderError::EmptyName.into());
        }

        if folders
            .iter()
            .any(|f| f.name.trim() == name && Some(f.id) != exclude_id)
        {
            return Err(FolderError::DuplicateName.into());
        }
        Ok(())
    }
}

/// Folder color struct
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    clap::ValueEnum,
    Serialize,
    Deserialize,
    Hash,
)]
pub enum FolderColor {
    #[default]
    Neutral,
    Blue,
    Sky,
    Cyan,
    Mint,
    Green,
    Lime,
    Yellow,
    Amber,
    Orange,
    Coral,
    Red,
    Rose,
    Pink,
    Magenta,
    Purple,
    Violet,
    Lavender,
    Indigo,
    Teal,
    Olive,
    Sand,
    Peach,
    Chocolate,
}

/// Returns ratatui::style::Color for every folder color
impl From<FolderColor> for Color {
    fn from(color: FolderColor) -> Self {
        match color {
            FolderColor::Neutral => Color::Rgb(150, 150, 150),
            FolderColor::Blue => Color::Rgb(80, 140, 220),
            FolderColor::Sky => Color::Rgb(100, 180, 230),
            FolderColor::Cyan => Color::Rgb(80, 200, 200),
            FolderColor::Mint => Color::Rgb(120, 210, 170),
            FolderColor::Green => Color::Rgb(130, 190, 120),
            FolderColor::Lime => Color::Rgb(170, 210, 100),
            FolderColor::Yellow => Color::Rgb(230, 200, 100),
            FolderColor::Amber => Color::Rgb(240, 170, 80),
            FolderColor::Orange => Color::Rgb(230, 140, 80),
            FolderColor::Coral => Color::Rgb(230, 110, 90),
            FolderColor::Red => Color::Rgb(220, 90, 100),
            FolderColor::Rose => Color::Rgb(220, 110, 150),
            FolderColor::Pink => Color::Rgb(230, 130, 190),
            FolderColor::Magenta => Color::Rgb(200, 100, 200),
            FolderColor::Purple => Color::Rgb(180, 110, 230),
            FolderColor::Violet => Color::Rgb(150, 120, 230),
            FolderColor::Lavender => Color::Rgb(170, 160, 230),
            FolderColor::Indigo => Color::Rgb(110, 120, 220),
            FolderColor::Teal => Color::Rgb(90, 170, 160),
            FolderColor::Olive => Color::Rgb(160, 160, 100),
            FolderColor::Sand => Color::Rgb(200, 180, 150),
            FolderColor::Peach => Color::Rgb(230, 170, 140),
            FolderColor::Chocolate => Color::Rgb(160, 130, 110),
        }
    }
}

/// Model for folder task
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEditor {
    pub name: String,
    pub color: FolderColor,
}

impl FolderEditor {
    pub fn new(name: impl Into<String>, color: FolderColor) -> Self {
        Self {
            name: name.into(),
            color,
        }
    }
}

/// Unit-tests for folder and folder color
#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread::sleep, time::Duration};

    fn create_folder(name: &str) -> Folder {
        Folder::new(name.to_string(), FolderColor::Red)
    }

    #[test]
    fn should_create_folder_with_defaults() {
        let folder = Folder::new("Work Stuff", FolderColor::default());

        assert!(!folder.id.to_string().is_empty());
        assert_eq!(folder.name, "Work Stuff");
        assert_eq!(folder.name_lower, "work stuff");
        assert_eq!(folder.color, FolderColor::default());
        assert_eq!(folder.created_at, folder.updated_at);
    }

    #[test]
    fn should_create_folder_with_custom_color() {
        let folder = Folder::new("Personal", FolderColor::Pink);
        assert_eq!(folder.color, FolderColor::Pink);
    }

    #[test]
    fn should_update_folder_fields_and_timestamp() {
        let mut folder = Folder::new("Old Name", FolderColor::Amber);
        let old_updated_at = folder.updated_at;

        sleep(Duration::from_millis(2));

        folder.update("New Name", FolderColor::Chocolate);

        assert_eq!(folder.name, "New Name");
        assert_eq!(folder.name_lower, "new name");
        assert_eq!(folder.color, FolderColor::Chocolate);
        assert!(folder.updated_at > old_updated_at);
    }

    #[test]
    fn should_convert_enum_to_ratatui_rgb() {
        let blue_tui: Color = FolderColor::Blue.into();
        assert_eq!(blue_tui, Color::Rgb(80, 140, 220));
    }

    #[test]
    fn should_validate_folder() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let folders = vec![
            Folder {
                id: id1,
                name: "Inbox".to_string(),
                ..create_folder("Inbox")
            },
            Folder {
                id: id2,
                name: "Work".to_string(),
                ..create_folder("Work")
            },
        ];

        assert!(Folder::validate("Personal", None, &folders).is_ok());
        assert!(Folder::validate("", None, &folders).is_err());
        assert!(Folder::validate("   ", None, &folders).is_err());
        assert!(Folder::validate("Inbox", None, &folders).is_err());
        assert!(Folder::validate("Inbox", Some(id1), &folders).is_ok());
        assert!(Folder::validate("Work", Some(id1), &folders).is_err());
    }
}
