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
    pub fn update_from_editor(&mut self, editor: FolderEditor) {
        self.name = editor.name;
        self.color = editor.color.to_string();
        self.name_lower = self.name.to_lowercase();
        self.updated_at = Utc::now();
    }
}

/// Folder color struct
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, strum::EnumIter, strum::EnumString,
)]
pub enum FolderColor {
    #[default]
    Blue,
    Red,
    Green,
    Yellow,
    Orange,
    Purple,
    Pink,
    Teal,
    Mint,
    Lavender,
    Coral,
    Peach,
    Salmon,
    Cyan,
    Magenta,
}

impl FolderColor {
    /// Returns hex for every folder color
    pub fn hex(&self) -> &'static str {
        match self {
            Self::Blue => "#458588",
            Self::Red => "#fb4934",
            Self::Green => "#b8bb26",
            Self::Yellow => "#fabd2f",
            Self::Orange => "#fe8019",
            Self::Purple => "#d3869b",
            Self::Pink => "#ff75a0",
            Self::Teal => "#83a598",
            Self::Mint => "#8ec07c",
            Self::Lavender => "#b16286",
            Self::Coral => "#fe8019",
            Self::Peach => "#f2e5bc",
            Self::Salmon => "#fa1561",
            Self::Cyan => "#076678",
            Self::Magenta => "#8f3f71",
        }
    }

    /// Converts string from database to ratatui Color
    pub fn from_string(s: &str) -> Color {
        use std::str::FromStr;

        if let Ok(color_enum) = FolderColor::from_str(s) {
            return color_enum.to_ratatui_color();
        }

        if let Some(color) = Self::parse_hex(s) {
            return color;
        }

        Self::default().to_ratatui_color()
    }

    /// Converts hex to ratatui rgb color
    pub fn to_ratatui_color(&self) -> Color {
        Self::parse_hex(self.hex()).unwrap_or(Color::Reset)
    }

    /// Helper to parse hex color to rgb
    fn parse_hex(hex_str: &str) -> Option<Color> {
        let clean_hex: &str = hex_str.trim_start_matches('#');
        if let Ok(val) = u32::from_str_radix(clean_hex, 16) {
            let r = ((val >> 16) & 0xFF) as u8;
            let g = ((val >> 8) & 0xFF) as u8;
            let b = (val & 0xFF) as u8;
            Some(Color::Rgb(r, g, b))
        } else {
            None
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
    fn should_convert_enum_to_ratatui_rgb() {
        let blue_tui = FolderColor::Blue.to_ratatui_color();
        assert_eq!(blue_tui, Color::Rgb(69, 133, 136));
    }

    #[test]
    fn should_parse_color_from_string_name_or_hex() {
        let color_from_name = FolderColor::from_string("Red");
        assert_eq!(color_from_name, FolderColor::Red.to_ratatui_color());

        let color_from_hex = FolderColor::from_string("#b8bb26");
        assert_eq!(color_from_hex, FolderColor::Green.to_ratatui_color());

        let invalid_color = FolderColor::from_string("NotAColor");
        assert_eq!(invalid_color, FolderColor::default().to_ratatui_color());
    }

    #[test]
    fn should_handle_hex_parsing_edge_cases() {
        assert!(FolderColor::parse_hex("fb4934").is_some());
        assert!(FolderColor::parse_hex("#fb4934").is_some());
        assert!(FolderColor::parse_hex("invalid").is_none());
    }
}
