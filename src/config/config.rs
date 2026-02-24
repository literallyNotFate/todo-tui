use super::*;
use crate::{
    core::{ApplicationError, StorageError},
    state::UIState,
    theme::ThemeName,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Main application config
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub ui: UIConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UIConfig {
                theme: ThemeName::default(),
                use_system_theme: true,
                last_dark: Some(ThemeName::default()),
                last_light: Some(ThemeName::GruvboxLight),
                show_sidebar: true,
                border_type: BorderTypeConfig::default(),
                symbols: SymbolsConfig::default(),
            },
        }
    }
}

impl Config {
    /// Get default config path to save/load from
    pub fn get_config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("todo-tui").join("config.toml");
        }

        PathBuf::from("config.toml")
    }

    /// Load config from a .toml file
    pub fn load(path: Option<&Path>) -> Result<Self, ApplicationError> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_config_path(),
        };

        if !p.exists() {
            return Ok(Self::default());
        }

        let content: String =
            fs::read_to_string(&p).map_err(|e| StorageError::IOError(e.to_string()))?;
        let config: Self =
            toml::from_str::<Self>(&content).map_err(|e| StorageError::TOMLError(e.to_string()))?;
        Ok(config)
    }

    /// Save config to a .toml file
    pub fn save(&self, path: Option<&Path>) -> Result<(), ApplicationError> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_config_path(),
        };

        if let Some(parent) = p.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| StorageError::IOError(e.to_string()))?;
            }
        }

        let content: String =
            toml::to_string_pretty(self).map_err(|e| StorageError::TOMLError(e.to_string()))?;
        fs::write(&p, content).map_err(|e| StorageError::IOError(e.to_string()))?;
        Ok(())
    }

    /// Update config from UI
    pub fn update_from_ui(&mut self, ui: &UIState) {
        let cfg = &mut self.ui;

        cfg.theme = ui.theme.name;
        cfg.last_dark = Some(ui.last_dark.name);
        cfg.last_light = Some(ui.last_light.name);
        cfg.show_sidebar = ui.config.show_sidebar;
        cfg.use_system_theme = ui.config.use_system_theme;
        cfg.symbols = ui.config.symbols.clone();
        cfg.border_type = ui.config.border_type;
    }
}

/// Unit-tests for config
#[cfg(test)]
mod tests {
    use crate::theme::Theme;

    use super::*;
    use tempdir::TempDir;

    #[test]
    fn should_return_default_data_path() {
        let path: PathBuf = Config::get_config_path();

        assert!(path.ends_with("todo-tui/config.toml"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_save_and_load_config_successfully() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.ui.theme = ThemeName::RosePineMoon;
        config.ui.use_system_theme = true;

        let result = config.save(Some(&path));
        assert!(result.is_ok(), "Save should succeed");

        let loaded_config = Config::load(Some(&path)).unwrap();
        assert_eq!(loaded_config.ui.theme, ThemeName::RosePineMoon);
        assert_eq!(loaded_config.ui.use_system_theme, true);
    }

    #[test]
    fn should_return_default_config_if_path_not_found() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("non_existent.toml");

        let result = Config::load(Some(&path));
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.ui.theme, Config::default().ui.theme);
        assert_eq!(config.ui.show_sidebar, true);
    }

    #[test]
    fn should_create_directories_on_config_save() {
        let temp_dir = TempDir::new("config_test").unwrap();

        let path = temp_dir
            .path()
            .join("deep")
            .join("path")
            .join("config.toml");

        let config = Config::default();
        let result = config.save(Some(&path));

        assert!(result.is_ok());
        assert!(path.exists());
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn should_invoke_toml_error_on_invalid_format() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("broken_config.toml");

        fs::write(&path, "ui = { theme = NotAStringWithoutQuotes }").unwrap();

        let result = Config::load(Some(&path));

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::TOMLError(..)))
        ));
    }

    #[test]
    fn should_correctly_handle_optional_fields_in_toml() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.ui.last_dark = Some(ThemeName::MelangeDark);
        config.ui.last_light = None;

        config.save(Some(&path)).unwrap();
        let content = fs::read_to_string(&path).unwrap();

        assert!(content.contains("last_dark = \"melange-dark\""));
        assert!(
            !content.contains("last_light"),
            "Field with None should be skipped"
        );
    }

    #[test]
    fn should_update_config_from_ui_state() {
        let initial_config = Config::default();
        let mut ui = UIState::new(initial_config.ui.clone());

        ui.theme = Theme::new(ThemeName::GruvboxDark);
        ui.last_dark = Theme::new(ThemeName::GruvboxDark);
        ui.config.show_sidebar = false;
        ui.config.use_system_theme = false;
        ui.config.border_type = BorderTypeConfig::Double;
        ui.config.symbols.completed = "DONE".to_string();

        let mut config_to_update = Config::default();
        config_to_update.update_from_ui(&ui);

        assert_eq!(config_to_update.ui.theme, ThemeName::GruvboxDark);
        assert_eq!(config_to_update.ui.last_dark, Some(ThemeName::GruvboxDark));
        assert_eq!(config_to_update.ui.show_sidebar, false);
        assert_eq!(config_to_update.ui.use_system_theme, false);
        assert_eq!(config_to_update.ui.border_type, BorderTypeConfig::Double);
        assert_eq!(config_to_update.ui.symbols.completed, "DONE");
        assert!(config_to_update.ui.last_light.is_some());
    }
}
