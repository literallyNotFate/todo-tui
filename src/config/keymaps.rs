use crate::{
    core::{Action, ApplicationError, KeyMapError, StorageError},
    theme::ThemePalette,
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::{Style, Stylize},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use strum::IntoEnumIterator;

/// Keymaps that are being saved on the disk
#[derive(Default, Serialize, Deserialize)]
struct KeyMapsDisk {
    #[serde(default)]
    navigation: HashMap<Action, Vec<String>>,
    #[serde(default)]
    tasks: HashMap<Action, Vec<String>>,
    #[serde(default)]
    filters: HashMap<Action, Vec<String>>,
    #[serde(default)]
    ui: HashMap<Action, Vec<String>>,
    #[serde(default)]
    system: HashMap<Action, Vec<String>>,
}

/// Keymaps for toodles application
#[derive(Clone, Debug)]
pub struct KeyMaps {
    pub mappings: HashMap<(KeyCode, KeyModifiers), Action>,
}

impl KeyMaps {
    /// Get default keymap path to save/load from
    pub fn get_keymap_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            return home.join(".config").join("toodles").join("keymaps.toml");
        }

        PathBuf::from("keymaps.toml")
    }

    /// Check if that action key is pressed
    pub fn is(&self, event: &KeyEvent, action: Action) -> bool {
        self.action(event) == Some(action)
    }

    // Get action by pressed key with modifier
    pub fn action(&self, event: &KeyEvent) -> Option<Action> {
        self.mappings.get(&(event.code, event.modifiers)).cloned()
    }

    /// Load config from a .toml file
    pub fn load(path: Option<&Path>) -> Result<Self, ApplicationError> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_keymap_path(),
        };

        if !p.exists() {
            return Ok(Self::default());
        }

        let content: String = fs::read_to_string(&p).map_err(|e| StorageError::IO {
            path: p.clone(),
            src: e.to_string(),
        })?;

        let disk_map: KeyMapsDisk = toml::from_str(&content).map_err(|e| StorageError::TOML {
            path: p,
            src: e.to_string(),
        })?;

        let mut mappings = HashMap::new();

        let mut process_section =
            |map: HashMap<Action, Vec<String>>| -> Result<(), ApplicationError> {
                for (action, keys) in map {
                    for key_str in keys {
                        let parsed = Self::parse(&key_str);

                        if let Some(existing_action) = mappings.get(&parsed) {
                            if *existing_action != action {
                                return Err(KeyMapError::DuplicateKey {
                                    key: key_str,
                                    first_action: format!("{:?}", existing_action),
                                    second_action: format!("{:?}", action),
                                }
                                .into());
                            }
                        }

                        mappings.insert(parsed, action);
                    }
                }
                Ok(())
            };

        process_section(disk_map.navigation)?;
        process_section(disk_map.tasks)?;
        process_section(disk_map.filters)?;
        process_section(disk_map.ui)?;
        process_section(disk_map.system)?;

        Ok(Self { mappings })
    }

    /// Save config to a .toml file
    pub fn save(&self, path: Option<&Path>) -> Result<(), ApplicationError> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_keymap_path(),
        };

        let mut disk_map: KeyMapsDisk = KeyMapsDisk::default();

        for (&(code, mods), &action) in &self.mappings {
            let key_str = Self::stringify_key(code, mods);
            let (_, category) = action.info();

            let target_map = match category {
                "Navigation" => &mut disk_map.navigation,
                "Actions" => &mut disk_map.tasks,
                "Filters" => &mut disk_map.filters,
                "UI" => &mut disk_map.ui,
                _ => &mut disk_map.system,
            };

            target_map.entry(action).or_default().push(key_str);
        }

        let content: String =
            toml::to_string_pretty(&disk_map).map_err(|e| StorageError::TOML {
                path: p.clone(),
                src: e.to_string(),
            })?;

        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(p.clone(), content).map_err(|e| StorageError::IO {
            path: p,
            src: e.to_string(),
        })?;
        Ok(())
    }

    /// Parse keymap string literal to crossterm keycodes w/keymodifiers
    fn parse(s: &str) -> (KeyCode, KeyModifiers) {
        let parts: Vec<&str> = s.split('+').collect();
        let mut mods = KeyModifiers::empty();
        let mut code = KeyCode::Null;

        for part in parts {
            let trimmed = part.trim();

            match trimmed.to_lowercase().as_str() {
                "ctrl" => mods.insert(KeyModifiers::CONTROL),
                "alt" => mods.insert(KeyModifiers::ALT),
                "shift" => mods.insert(KeyModifiers::SHIFT),
                "enter" => code = KeyCode::Enter,
                "esc" => code = KeyCode::Esc,
                "space" => code = KeyCode::Char(' '),
                "tab" => code = KeyCode::Tab,
                "up" => code = KeyCode::Up,
                "down" => code = KeyCode::Down,
                "left" => code = KeyCode::Left,
                "right" => code = KeyCode::Right,
                _ if trimmed.len() == 1 => {
                    let c = trimmed.chars().next().unwrap();
                    code = KeyCode::Char(c);
                    if c.is_uppercase() {
                        mods.insert(KeyModifiers::SHIFT);
                    }
                }
                _ => {}
            }
        }

        (code, mods)
    }

    /// Helper function to transform key code to string that is being save to keymaps.toml
    fn stringify_key(code: KeyCode, mods: KeyModifiers) -> String {
        let mut s = String::new();

        if mods.contains(KeyModifiers::CONTROL) {
            s.push_str("ctrl+");
        }
        if mods.contains(KeyModifiers::ALT) {
            s.push_str("alt+");
        }
        if mods.contains(KeyModifiers::SHIFT) && !matches!(code, KeyCode::Char(_)) {
            s.push_str("shift+");
        }

        match code {
            KeyCode::Char(' ') => s.push_str("space"),
            KeyCode::Char(c) => s.push(c),
            KeyCode::Enter => s.push_str("enter"),
            KeyCode::Esc => s.push_str("esc"),
            KeyCode::Tab => s.push_str("tab"),
            KeyCode::Backspace => s.push_str("backspace"),
            KeyCode::Delete => s.push_str("delete"),
            KeyCode::Up => s.push_str("up"),
            KeyCode::Down => s.push_str("down"),
            KeyCode::Left => s.push_str("left"),
            KeyCode::Right => s.push_str("right"),
            KeyCode::F(n) => s.push_str(&format!("f{}", n)),
            _ => s.push_str("unknown"),
        }

        s
    }

    /// Get key string based on action
    pub fn key(&self, action: Action) -> String {
        let mut keys: Vec<String> = self
            .mappings
            .iter()
            .filter(|&(_, mapped_action)| *mapped_action == action)
            .map(|(&(code, mods), _)| format!("<{}>", Self::stringify_key(code, mods)))
            .collect();

        keys.sort();
        keys.join(" | ")
    }

    /// Get first assigned key (if there are more than 1 option)
    pub fn first_assigned(&self, action: Action) -> String {
        let mut keys: Vec<(KeyCode, KeyModifiers)> = self
            .mappings
            .iter()
            .filter(|&(_, mapped_action)| *mapped_action == action)
            .map(|(&key, _)| key)
            .collect();

        keys.sort_by_key(|(code, _)| match code {
            KeyCode::Char(_) => 0,
            _ => 1,
        });

        if let Some(&(code, mods)) = keys.first() {
            format!("<{}>", Self::stringify_key(code, mods))
        } else {
            String::new()
        }
    }

    /// Check whether is a process kill key (Ctrl+C)
    pub fn is_kill_process(e: &KeyEvent) -> bool {
        let killed = e.code == KeyCode::Char('c') && e.modifiers.contains(KeyModifiers::CONTROL);
        if killed {
            log::warn!("Process kill signal (Ctrl+C) detected");
        }

        killed
    }

    /// Check whether exit key is pressed
    pub fn is_exit(&self, e: &KeyEvent) -> bool {
        if Self::is_kill_process(e) || e.code == KeyCode::Esc {
            return true;
        }

        self.is(e, Action::Quit)
    }

    /// Generate hotkeys for help popup
    pub fn hotkeys_info(&self, palette: &ThemePalette) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let all_actions: Vec<Action> = Action::iter().collect();
        let categories: [&str; 5] = ["Navigation", "Actions", "Filters", "UI", "System"];

        for cat in categories {
            let cat_actions: Vec<Action> = all_actions
                .iter()
                .filter(|a| a.info().1 == cat)
                .cloned()
                .collect();

            if cat_actions.is_empty() {
                continue;
            }

            self.add_section(&mut lines, cat, palette);

            for action in cat_actions {
                let (desc, _) = action.info();
                let keys = self.key(action);

                if !keys.is_empty() {
                    self.add_command(&mut lines, keys, desc, palette);
                }
            }
        }

        lines
    }

    /// Adds hotkey command
    fn add_command(
        &self,
        lines: &mut Vec<Line<'static>>,
        key: String,
        desc: &'static str,
        palette: &ThemePalette,
    ) {
        let key_col_width: usize = 20;
        let key_str: String = format!("{:>width$}", key, width = key_col_width);

        lines.push(Line::from(vec![
            Span::styled(key_str, Style::default().fg(palette.info).bold()),
            Span::styled(" │ ", Style::default().fg(palette.muted)),
            Span::styled(desc, Style::default().fg(palette.fg)),
        ]));
    }

    /// Add hotkeys section
    fn add_section(
        &self,
        lines: &mut Vec<Line<'static>>,
        title: &'static str,
        palette: &ThemePalette,
    ) {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines.push(
            Line::from(vec![
                Span::styled("── ", Style::default().fg(palette.accent)),
                Span::styled(
                    title.to_uppercase(),
                    Style::default().fg(palette.accent).bold(),
                ),
                Span::styled(" ──", Style::default().fg(palette.accent)),
            ])
            .centered(),
        );
    }
}

impl Default for KeyMaps {
    fn default() -> Self {
        let mut mappings = HashMap::new();

        let defaults = [
            (Action::MoveUp, "k"),
            (Action::MoveUp, "up"),
            (Action::MoveDown, "j"),
            (Action::MoveDown, "down"),
            (Action::MoveTaskUp, "K"),
            (Action::MoveTaskDown, "J"),
            (Action::MoveLeft, "h"),
            (Action::MoveLeft, "left"),
            (Action::MoveRight, "l"),
            (Action::MoveRight, "right"),
            // Filters
            (Action::FilterInbox, "1"),
            (Action::FilterActive, "2"),
            (Action::FilterCompleted, "3"),
            (Action::FilterHigh, "4"),
            (Action::FilterToday, "5"),
            // Task Actions
            (Action::AddTask, "a"),
            (Action::Complete, "enter"),
            (Action::Update, "e"),
            (Action::Remove, "d"),
            (Action::Details, "tab"),
            (Action::Search, "/"),
            (Action::Pin, "p"),
            (Action::ClearAll, "x"),
            // Folder actions
            (Action::AddFolder, "f"),
            // System & UI
            (Action::Sort, "s"),
            (Action::SortReverse, "r"),
            (Action::NextTheme, "t"),
            (Action::PrevTheme, "ctrl+t"),
            (Action::ToggleThemeMode, "m"),
            (Action::ToggleSidebar, "b"),
            (Action::Save, "ctrl+s"),
            (Action::ToggleAutosave, "alt+a"),
            (Action::Quit, "q"),
            (Action::Quit, "esc"),
            (Action::ShowHelp, "?"),
        ];

        for (action, key) in defaults {
            let parsed = Self::parse(key);
            mappings.insert(parsed, action);
        }

        KeyMaps { mappings }
    }
}

/// Unit-tests for keymaps
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;
    use tempdir::TempDir;

    #[test]
    fn should_return_default_data_path() {
        let path: PathBuf = KeyMaps::get_keymap_path();

        assert!(path.ends_with("toodles/keymaps.toml"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_parse_simple_keys() {
        let (code, mods) = KeyMaps::parse("enter");
        assert_eq!(code, KeyCode::Enter);
        assert!(mods.is_empty());

        let (code, mods) = KeyMaps::parse("j");
        assert_eq!(code, KeyCode::Char('j'));
        assert!(mods.is_empty());
    }

    #[test]
    fn should_parse_modifier_keys() {
        let (code, mods) = KeyMaps::parse("ctrl+s");
        assert_eq!(code, KeyCode::Char('s'));
        assert_eq!(mods, KeyModifiers::CONTROL);

        let (code, mods) = KeyMaps::parse("alt+shift+up");
        assert_eq!(code, KeyCode::Up);
        assert!(mods.contains(KeyModifiers::ALT));
        assert!(mods.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn should_stringify_keys_correctly() {
        let s = KeyMaps::stringify_key(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(s, "ctrl+s");

        let s = KeyMaps::stringify_key(KeyCode::Enter, KeyModifiers::empty());
        assert_eq!(s, "enter");

        let s = KeyMaps::stringify_key(KeyCode::Char(' '), KeyModifiers::empty());
        assert_eq!(s, "space");
    }

    #[test]
    fn should_save_with_proper_sections() {
        let temp_dir = TempDir::new("keymap_sections_test").unwrap();
        let path = temp_dir.path().join("sections.toml");

        let mut mappings = HashMap::new();
        mappings.insert((KeyCode::Up, KeyModifiers::empty()), Action::MoveUp);
        mappings.insert((KeyCode::Char('q'), KeyModifiers::empty()), Action::Quit);

        let keymaps = KeyMaps { mappings };
        keymaps.save(Some(&path)).expect("Should save");

        let content = std::fs::read_to_string(&path).unwrap();

        assert!(
            content.contains("[navigation]"),
            "Should have navigation section"
        );
        assert!(content.contains("[system]"), "Should have system section");
        assert!(content.contains("move_up = [\"up\"]"));
    }

    #[test]
    fn should_handle_incomplete_toml_gracefully() {
        let temp_dir = TempDir::new("keymap_incomplete").unwrap();
        let path = temp_dir.path().join("partial.toml");

        let toml_content = r#"
            [navigation]
            move_up = ["k"]
        "#;
        std::fs::write(&path, toml_content).unwrap();

        let loaded = KeyMaps::load(Some(&path)).expect("Should load partial config");

        let event_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());
        assert_eq!(loaded.action(&event_k), Some(Action::MoveUp));
    }

    #[test]
    fn should_fail_on_duplicate_keys_in_toml() {
        let temp_dir = TempDir::new("keymap_error_test").unwrap();
        let path = temp_dir.path().join("error_keymaps.toml");

        let toml_content = r#"
            [tasks]
            add_task = ["a"]
            remove = ["a"]
        "#;
        std::fs::write(&path, toml_content).unwrap();

        let result = KeyMaps::load(Some(&path));

        match result {
            Err(ApplicationError::KeyMap(KeyMapError::DuplicateKey { key, .. })) => {
                assert_eq!(key, "a");
            }
            _ => panic!("Should have returned DuplicateKey error, got {:?}", result),
        }
    }

    #[test]
    fn should_load_default_when_file_is_missing() {
        let dir = TempDir::new("keymap_missing_test").unwrap();
        let non_existent = dir.path().join("nothing.toml");
        let loaded = KeyMaps::load(Some(&non_existent)).expect("Should not fail if file missing");

        assert!(matches!(
            loaded.mappings.get(&(KeyCode::Esc, KeyModifiers::NONE)),
            Some(Action::Quit)
        ));
    }

    #[test]
    fn should_handle_uppercase_as_shift() {
        let (code, mods) = KeyMaps::parse("J");
        assert_eq!(code, KeyCode::Char('J'));
        assert!(mods.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn should_check_if_its_kill_process_key() {
        let key_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(KeyMaps::is_kill_process(&key_ctrl_c));

        let key_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert!(!KeyMaps::is_kill_process(&key_c));

        let key_other = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(!KeyMaps::is_kill_process(&key_other));
    }

    #[test]
    fn should_check_if_its_is_exit_key() {
        let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let keymap: KeyMaps = KeyMaps::default();

        assert!(keymap.is_exit(&key_q));

        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(keymap.is_exit(&key_esc));

        let key_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(keymap.is_exit(&key_ctrl_c));

        let key_other = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!keymap.is_exit(&key_other));
    }

    #[test]
    fn should_test_hotkeys_alignment_consistency() {
        let keymap: KeyMaps = KeyMaps::default();
        let palette = ThemeName::GruvboxDark.palette();
        let lines = keymap.hotkeys_info(&palette);

        for line in lines {
            let spans: Vec<_> = line.spans.iter().collect();
            if spans.len() >= 3 && spans[1].content.contains('│') {
                assert_eq!(spans[0].content.len(), 20, "Key column width must be 20");
            }
        }
    }

    #[test]
    fn should_add_section_spacing() {
        let keymap: KeyMaps = KeyMaps::default();
        let palette = ThemeName::GruvboxDark.palette();
        let lines = keymap.hotkeys_info(&palette);

        let empty_lines = lines
            .iter()
            .filter(|l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content.is_empty()))
            .count();
        assert!(
            empty_lines >= 2,
            "Sections should be separated by empty lines"
        );
    }
}
