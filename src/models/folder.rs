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
    pub color: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name_lower: String,
}

impl Folder {
    pub fn new<S: Into<String>>(name: S, color: S) -> Self {
        let name_str: String = name.into();
        let name_lower: String = name_str.to_lowercase();
        let now: DateTime<Utc> = Utc::now();

        Self {
            id: Uuid::new_v4(),
            name: name_str,
            color: color.into(),
            created_at: now,
            updated_at: now,
            name_lower,
        }
    }

    /// Helper to update name and color
    pub fn update<S: Into<String>>(&mut self, name: S, color: S) {
        let name_str = name.into();
        self.name_lower = name_str.to_lowercase();
        self.name = name_str;
        self.color = color.into();
        self.updated_at = Utc::now();
    }

    /// Update folder from FolderEditor
    pub fn update_from(&mut self, editor: FolderEditor) {
        self.name = editor.name;
        self.color = editor.color.to_string();
        self.name_lower = self.name.to_lowercase();
        self.updated_at = Utc::now();
    }

    // Static method to find folder by ID
    pub fn find_by_id(folders: &[Folder], id: &Uuid) -> Option<Self> {
        folders.iter().find(|f| f.id == *id).cloned()
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
    Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, strum::EnumIter, strum::EnumString,
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

impl FolderColor {
    /// Returns ratatui::style::Color for every folder color
    pub fn to_ratatui_color(&self) -> Color {
        match self {
            Self::Neutral => Color::Rgb(150, 150, 150),
            Self::Blue => Color::Rgb(80, 140, 220),
            Self::Sky => Color::Rgb(100, 180, 230),
            Self::Cyan => Color::Rgb(80, 200, 200),
            Self::Mint => Color::Rgb(120, 210, 170),
            Self::Green => Color::Rgb(130, 190, 120),
            Self::Lime => Color::Rgb(170, 210, 100),
            Self::Yellow => Color::Rgb(230, 200, 100),
            Self::Amber => Color::Rgb(240, 170, 80),
            Self::Orange => Color::Rgb(230, 140, 80),
            Self::Coral => Color::Rgb(230, 110, 90),
            Self::Red => Color::Rgb(220, 90, 100),
            Self::Rose => Color::Rgb(220, 110, 150),
            Self::Pink => Color::Rgb(230, 130, 190),
            Self::Magenta => Color::Rgb(200, 100, 200),
            Self::Purple => Color::Rgb(180, 110, 230),
            Self::Violet => Color::Rgb(150, 120, 230),
            Self::Lavender => Color::Rgb(170, 160, 230),
            Self::Indigo => Color::Rgb(110, 120, 220),
            Self::Teal => Color::Rgb(90, 170, 160),
            Self::Olive => Color::Rgb(160, 160, 100),
            Self::Sand => Color::Rgb(200, 180, 150),
            Self::Peach => Color::Rgb(230, 170, 140),
            Self::Chocolate => Color::Rgb(160, 130, 110),
        }
    }

    /// Converts string from database to ratatui Color
    pub fn from_string(s: &str) -> Color {
        use std::str::FromStr;

        if let Ok(color_enum) = FolderColor::from_str(s) {
            return color_enum.to_ratatui_color();
        }

        Self::parse_hex(s).unwrap_or(Self::default().to_ratatui_color())
    }

    /// Helper to parse hex color to rgb
    fn parse_hex(hex_str: &str) -> Option<Color> {
        let clean: &str = hex_str.trim_start_matches('#');
        u32::from_str_radix(clean, 16)
            .map(|v| {
                Color::Rgb(
                    ((v >> 16) & 0xFF) as u8,
                    ((v >> 8) & 0xFF) as u8,
                    (v & 0xFF) as u8,
                )
            })
            .ok()
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
        Folder::new(name.to_string(), "Red".into())
    }

    #[test]
    fn should_create_folder_with_defaults() {
        let folder = Folder::new("Work Stuff", &FolderColor::default().to_string());

        assert!(!folder.id.to_string().is_empty());
        assert_eq!(folder.name, "Work Stuff");
        assert_eq!(folder.name_lower, "work stuff");
        assert_eq!(folder.color, FolderColor::default().to_string());
        assert_eq!(folder.created_at, folder.updated_at);
    }

    #[test]
    fn should_create_folder_with_custom_color_string() {
        let folder = Folder::new("Personal", "Red");
        assert_eq!(folder.color, "Red");
    }

    #[test]
    fn should_update_folder_fields_and_timestamp() {
        let mut folder = Folder::new("Old Name", "Blue");
        let old_updated_at = folder.updated_at;

        sleep(Duration::from_millis(2));

        folder.update("New Name", "Green");

        assert_eq!(folder.name, "New Name");
        assert_eq!(folder.name_lower, "new name");
        assert_eq!(folder.color, "Green");
        assert!(folder.updated_at > old_updated_at);
    }

    #[test]
    fn should_return_folder_name_by_id() {
        let folders = vec![Folder::new("Test1", "Blue"), Folder::new("Test2", "Red")];
        let id = folders[0].id;

        let found: Option<Folder> = Folder::find_by_id(&folders, &id);
        assert_eq!(found.unwrap().name, "Test1");

        let found: Option<Folder> = Folder::find_by_id(&folders, &Uuid::new_v4());
        assert!(found.is_none());
    }

    #[test]
    fn should_convert_enum_to_ratatui_rgb() {
        let blue_tui = FolderColor::Blue.to_ratatui_color();
        assert_eq!(blue_tui, Color::Rgb(80, 140, 220));
    }

    #[test]
    fn should_parse_color_from_string_name_or_hex() {
        let color_from_name = FolderColor::from_string("Red");
        assert_eq!(color_from_name, FolderColor::Red.to_ratatui_color());

        let color_from_hex = FolderColor::from_string("#b8bb26");
        assert_eq!(color_from_hex, Color::Rgb(184, 187, 38));
    }

    #[test]
    fn should_handle_hex_parsing_edge_cases() {
        assert!(FolderColor::parse_hex("fb4934").is_some());
        assert!(FolderColor::parse_hex("#fb4934").is_some());
        assert!(FolderColor::parse_hex("invalid").is_none());
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
