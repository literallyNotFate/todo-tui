use super::*;
use crate::{
    core::{ApplicationError, StorageError},
    state::UIState,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Main application config
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub ui: UIConfig,
    pub storage: StorageConfig,
    pub log: LogConfig,
    pub behavior: BehaviorConfig,
    pub task: TaskConfig,
}

impl Config {
    /// Get default config path to save/load from
    pub fn get_config_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("toodles").join("config.toml");
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

        let content: String = fs::read_to_string(&p).map_err(|e| StorageError::IO {
            path: p.clone(),
            src: e.to_string(),
        })?;
        let config: Self = toml::from_str::<Self>(&content).map_err(|e| StorageError::TOML {
            path: p,
            src: e.to_string(),
        })?;
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
                fs::create_dir_all(parent).map_err(|e| StorageError::IO {
                    path: parent.to_path_buf(),
                    src: e.to_string(),
                })?;
            }
        }

        let content: String = toml::to_string_pretty(self).map_err(|e| StorageError::TOML {
            path: p.clone(),
            src: e.to_string(),
        })?;

        fs::write(&p, content).map_err(|e| StorageError::IO {
            path: p,
            src: e.to_string(),
        })?;
        Ok(())
    }

    /// Update config from UI
    pub fn update_from_ui(&mut self, ui: &UIState) {
        self.ui = ui.config.clone();
    }
}

/// Unit-tests for config
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Theme, ThemeName};
    use tempdir::TempDir;

    #[test]
    fn should_return_default_data_path() {
        let path: PathBuf = Config::get_config_path();

        assert!(path.to_string_lossy().contains("toodles"));
        assert!(path.to_string_lossy().contains("config.toml"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_save_and_load_config_successfully() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        let test_theme = ThemeID::Builtin(ThemeName::RosePineMoon);

        config.ui.theme = test_theme.clone();
        config.ui.use_system_theme = true;

        let result = config.save(Some(&path));
        assert!(result.is_ok(), "Save should succeed");

        let loaded_config = Config::load(Some(&path)).unwrap();
        assert_eq!(loaded_config.ui.theme, test_theme);
        assert!(loaded_config.ui.use_system_theme);
    }

    #[test]
    fn should_return_default_config_if_path_not_found() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("non_existent.toml");

        let result = Config::load(Some(&path));
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.ui.theme, Config::default().ui.theme);
        assert!(config.ui.show_sidebar);
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
            Err(ApplicationError::Storage(StorageError::TOML { .. }))
        ));

        if let Err(ApplicationError::Storage(StorageError::TOML { path: err_path, .. })) = result {
            assert_eq!(err_path, path);
        }
    }

    #[test]
    fn should_correctly_handle_optional_fields_in_toml() {
        let temp_dir = TempDir::new("config_test").unwrap();
        let path = temp_dir.path().join("config.toml");

        let mut config = Config::default();
        let dark = ThemeID::Builtin(ThemeName::MelangeDark);
        config.ui.last_dark = Some(dark.clone());
        config.ui.last_light = None;

        config.save(Some(&path)).unwrap();
        let content = fs::read_to_string(&path).unwrap();

        assert!(content.contains("last_dark"));
        assert!(content.contains("builtin = \"melange-dark\""));
        assert!(!content.contains("last_light"));
    }

    #[test]
    fn should_update_config_from_ui_state() {
        let initial_config = Config::default();
        let mut ui = UIState::new(initial_config.ui.clone());

        let new_theme_id = ThemeID::Builtin(ThemeName::GruvboxDark);
        ui.theme = Theme::new(new_theme_id.clone());
        ui.config.show_sidebar = false;
        ui.config.use_system_theme = false;
        ui.config.symbols.completed = "DONE".to_string();

        let mut config_to_update = Config::default();
        config_to_update.update_from_ui(&ui);

        assert_eq!(config_to_update.ui.theme, new_theme_id);
        assert_eq!(config_to_update.ui.last_dark, Some(new_theme_id));
        assert!(!config_to_update.ui.show_sidebar);
        assert_eq!(config_to_update.ui.symbols.completed, "DONE");
    }
}
