use crate::{core::StorageError, theme::ThemePalette};
use ratatui::style::Color;
use std::{fs, path::PathBuf};

/// Theme structure, how its being presented in TOML file
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ThemeDisk {
    pub name: String,
    pub palette: ThemePaletteDisk,
}

/// Theme palette structure from TOML (string HEX or color names)
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ThemePaletteDisk {
    pub accent: String,
    pub secondary: String,
    pub bg: String,
    pub bg2: String,
    pub fg: String,
    pub muted: String,
    pub selection: String,
    pub error: String,
    pub warning: String,
    pub success: String,
    pub info: String,
}

impl From<ThemePaletteDisk> for ThemePalette {
    /// Casts string color values to ratatui::style::Color
    /// Returns Color::Reset if color is wrongly specified
    fn from(disk: ThemePaletteDisk) -> Self {
        let p = |s: &str| s.parse::<Color>().unwrap_or(Color::Reset);

        Self {
            accent: p(&disk.accent),
            secondary: p(&disk.secondary),
            bg: p(&disk.bg),
            bg2: p(&disk.bg2),
            fg: p(&disk.fg),
            muted: p(&disk.muted),
            selection: p(&disk.selection),
            error: p(&disk.error),
            warning: p(&disk.warning),
            success: p(&disk.success),
            info: p(&disk.info),
        }
    }
}

/// Struct for search and load theme files from the disk
pub struct PaletteDisk;

impl PaletteDisk {
    /// Returns directory with custom themes: ~/.config/toodles/themes
    pub fn themes_dir() -> Result<PathBuf, StorageError> {
        dirs::home_dir()
            .map(|d| d.join(".config").join("toodles").join("themes"))
            .ok_or(StorageError::Environment {
                context: "themes".to_string(),
            })
    }

    /// Scans themes directory and tries to load every .toml file.
    /// Returns list of tuples (theme key / file stem, palette)
    pub fn load_all() -> Vec<(String, ThemePalette)> {
        let mut custom_themes = Vec::new();

        let Ok(dir) = Self::themes_dir() else {
            log::error!("Could not determine themes directory");
            return custom_themes;
        };

        if let Err(e) = fs::create_dir_all(&dir) {
            log::error!("Failed to create themes directory {}: {}", dir.display(), e);
            return custom_themes;
        }

        let Ok(entries) = fs::read_dir(&dir) else {
            return custom_themes;
        };

        for entry in entries.flatten() {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };

                let theme_key: String = file_stem.to_lowercase();
                match Self::load_single_theme(&path) {
                    Ok((_, palette)) => custom_themes.push((theme_key, palette)),
                    Err(e) => log::error!("Skipping theme {:?}: {}", path.file_name(), e),
                }
            }
        }

        custom_themes
    }

    /// Helper method to load a single theme file by its name (e.g. "nord")
    pub fn load_single(name: &str) -> Option<ThemePalette> {
        let dir: PathBuf = Self::themes_dir().ok()?;
        let path: PathBuf = dir.join(format!("{}.toml", name.to_lowercase()));
        Self::load_single_theme(&path)
            .ok()
            .map(|(_, palette)| palette)
    }

    /// Helper method to load only one file theme (.toml) and return its display name and palette
    fn load_single_theme(path: &PathBuf) -> Result<(String, ThemePalette), StorageError> {
        let content: String = fs::read_to_string(path).map_err(|e| StorageError::IO {
            path: path.to_owned(),
            src: e.to_string(),
        })?;

        let disk_theme: ThemeDisk = toml::from_str(&content).map_err(|e| StorageError::TOML {
            path: path.to_owned(),
            src: e.to_string(),
        })?;

        Ok((disk_theme.name, ThemePalette::from(disk_theme.palette)))
    }
}

/// Unit-tests for theme disk operations
#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn should_parse_colors_from_disk_format() {
        let disk_palette = ThemePaletteDisk {
            accent: "#ff0000".to_string(),
            secondary: "blue".to_string(),
            bg: "invalid-color".to_string(),
            bg2: "#00ff00".to_string(),
            fg: "#ffffff".to_string(),
            muted: "#888888".to_string(),
            selection: "#444444".to_string(),
            error: "red".to_string(),
            warning: "yellow".to_string(),
            success: "green".to_string(),
            info: "cyan".to_string(),
        };

        let palette = ThemePalette::from(disk_palette);

        assert_eq!(palette.accent, Color::Rgb(255, 0, 0));
        assert_eq!(palette.secondary, Color::Blue);
        assert_eq!(palette.bg, Color::Reset);
    }

    #[test]
    fn should_load_single_theme_from_toml() {
        let temp_dir: TempDir = TempDir::new("toodles_themes").unwrap();
        let path: PathBuf = temp_dir.path().join("my_theme.toml");

        let toml_content: &str = r##"
        name = "Custom Theme"
        [palette]
        accent = "magenta"
        secondary = "cyan"
        bg = "#1a1a1a"
        bg2 = "#2a2a2a"
        fg = "#eeeeee"
        muted = "#666666"
        selection = "#333333"
        error = "red"
        warning = "yellow"
        success = "green"
        info = "blue"
        "##;

        fs::write(&path, toml_content).unwrap();

        let result = PaletteDisk::load_single_theme(&path);

        assert!(result.is_ok());
        let (name, palette) = result.unwrap();
        assert_eq!(name, "Custom Theme");
        assert_eq!(palette.accent, Color::Magenta);
        assert_eq!(palette.bg, Color::Rgb(26, 26, 26));
    }

    #[test]
    fn should_return_error_on_invalid_toml() {
        let temp_dir: TempDir = TempDir::new("toodles_themes").unwrap();
        let path: PathBuf = temp_dir.path().join("broken.toml");

        fs::write(&path, "not a toml content").unwrap();

        let result = PaletteDisk::load_single_theme(&path);
        assert!(result.is_err());
    }

    #[test]
    fn should_handle_missing_themes_dir_gracefully() {
        let _ = PaletteDisk::load_all();

        assert!(
            std::panic::catch_unwind(|| {
                PaletteDisk::load_all();
            })
            .is_ok()
        );
    }
}
