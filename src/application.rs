use crate::{
    config::Config,
    core::{ApplicationError, ApplicationMode, Autosave},
    enums::FocusArea,
    events::EventHandler,
    state::{ApplicationState, UIState},
    ui::Renderer,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::{self, Event},
        terminal,
    },
};
use std::time::{Duration, Instant};

pub struct Application {
    pub data: ApplicationState,
    pub ui: UIState,
    pub running: bool,
    pub renderer: Renderer,
    pub mode: ApplicationMode,
    pub autosave: Autosave,

    pub config: Config,
    pub size: (u16, u16),
}

impl Application {
    pub fn new() -> Self {
        let size: (u16, u16) = terminal::size().unwrap_or((100, 100));
        let (config, config_error): (Config, Option<ApplicationError>) = Self::load_config();

        let mut app = Self {
            config: config.clone(),
            ui: UIState::new(config.ui.clone()),
            mode: ApplicationMode::Browsing,
            data: ApplicationState::new(&config.storage),
            running: true,
            renderer: Renderer,
            autosave: Autosave::new(false),
            size,
        };

        app.setup_autosave();
        if let Some(e) = config_error {
            app.ui.show_result_popup(Err(e));
        }

        app
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        const TICK_RATE: Duration = Duration::from_millis(100);
        let mut last_tick: Instant = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout: Duration = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Resize(w, h) => self.size = (w, h),
                    Event::Key(key_event) => {
                        self.autosave.register_activity();
                        EventHandler::handle_key(self, key_event);
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.tick();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    /// Rendering application using Renderer
    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer
            .render(frame, &mut self.data, &self.ui, self.mode, &self.autosave);
    }

    /// Tick function (for notification and autosave)
    pub fn tick(&mut self) {
        self.ui.expire_notification(&mut self.data.notification);
        let has_changes: bool = self.data.any_unsaved_changes();

        if self.autosave.enabled {
            if has_changes && !self.autosave.last_tick_had_changes {
                self.autosave.reset_timer();
            }

            if has_changes && self.autosave.should_save(has_changes) {
                self.config.update_from_ui(&self.ui);

                let config_saved: bool = self.config.save(None).is_ok();
                let data_saved: bool = self.data.save(None, &self.config.storage).is_ok();

                if config_saved && data_saved {
                    self.autosave.reset_timer();
                }
            }

            self.autosave.last_tick_had_changes = has_changes;
        }
    }

    /// Synchronizing selection after tab switching
    pub fn sync_ui(&mut self) {
        let query = self.ui.search_query();
        let indices = self.ui.current_filter.apply(&self.data.todos, &query);
        let visible_count = indices.len();

        if visible_count == 0 {
            self.data.select_state.select(None);
        } else {
            let current = self.data.select_state.selected();
            match current {
                None => self.data.select_state.select(Some(0)),
                Some(idx) if idx >= visible_count => {
                    self.data.select_state.select(Some(visible_count - 1));
                }
                _ => {}
            }
        }
    }

    /// Restoring base mode (after form exit)
    pub fn restore_base_mode(&mut self) {
        self.mode = match self.ui.focus_area {
            FocusArea::LeftPanel => ApplicationMode::Browsing,
            FocusArea::MainContent => ApplicationMode::List,
        };
    }

    /// Helper function to load config
    fn load_config() -> (Config, Option<ApplicationError>) {
        match Config::load(None) {
            Ok(cfg) => (cfg, None),
            Err(e) => (Config::default(), Some(e)),
        }
    }

    /// Setup autosave with config values
    pub fn setup_autosave(&mut self) {
        self.autosave.enabled = self.config.storage.autosave_enabled;
        self.autosave.interval = Duration::from_secs(self.config.storage.safe_interval());
    }

    /// Create mock application (for testing)
    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            data: ApplicationState::default(),
            ui: UIState::default(),
            mode: ApplicationMode::Browsing,
            running: true,
            renderer: Renderer,
            autosave: Autosave::new(false),
            size: (80, 24),
            config: Config::default(),
        }
    }
}

/// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{Filter, Todo},
        ui::Notification,
    };
    use std::{
        path::{Path, PathBuf},
        thread::sleep,
        time::{Duration, Instant},
    };
    use tempdir::TempDir;

    fn mock_tick(app: &mut Application, path: Option<&Path>) {
        let has_changes: bool = app.data.any_unsaved_changes();

        if app.autosave.enabled {
            if has_changes && !app.autosave.last_tick_had_changes {
                app.autosave.reset_timer();
            }

            if has_changes && app.autosave.should_save(has_changes) {
                if app.data.save(path, &app.config.storage).is_ok() {
                    app.autosave.reset_timer();
                }
            }

            app.autosave.last_tick_had_changes = has_changes;
        }
    }

    #[test]
    fn should_create_application() {
        let app = Application::test();

        assert!(app.running, "running should be true by default");
        assert_eq!(app.data.todos.len(), 0, "todos should be empty");
        assert!(
            app.data.notification.is_none(),
            "notification should be none"
        );
        assert!(app.ui.modal.is_none(), "modal should be none");
    }

    #[test]
    fn should_test_tick_expires_notification() {
        let mut app = Application::test();
        let old_time = Instant::now() - Duration::from_secs(10);

        let mut notification: Notification = Notification::success("Test");
        notification.created_at = old_time;
        app.data.notification = Some(notification);

        app.tick();

        assert!(
            app.data.notification.is_none(),
            "expired notification should be removed"
        );
    }

    #[test]
    fn should_select_none_if_empty_with_sync_ui() {
        let mut app = Application::test();
        app.data.select_state.select(Some(5));
        app.data.todos.clear();

        app.sync_ui();

        assert_eq!(
            app.data.select_state.selected(),
            None,
            "If list is empty, should select None"
        );
    }

    #[test]
    fn should_adjust_out_of_bounds_index_with_sync_ui() {
        let mut app = Application::test();

        app.data.todos.push(Todo::new("1", "", None));
        app.data.todos.push(Todo::new("2", "", None));
        app.data.todos.push(Todo::new("3", "", None));

        app.data.select_state.select(Some(2));
        app.data.todos.truncate(1);

        app.sync_ui();

        assert_eq!(
            app.data.select_state.selected(),
            Some(0),
            "Index must stay at last accessible task"
        );
    }

    #[test]
    fn should_initialize_selection_with_sync_ui() {
        let mut app = Application::test();

        app.data.todos.push(Todo::new("Task", "", None));
        app.data.select_state.select(None);

        app.sync_ui();

        assert_eq!(
            app.data.select_state.selected(),
            Some(0),
            "If selection was None, but there are tasks, choose first"
        );
    }

    #[test]
    fn should_adjust_selection_on_filter_change_with_sync_ui() {
        let mut app = Application::test();

        app.data.todos.push(Todo::new("Active", "", None));
        app.data.todos.push(Todo::new("Done", "", None));
        app.data.todos[1].completed = true;

        app.ui.current_filter = Filter::Completed;
        app.data.select_state.select(Some(1));

        app.sync_ui();

        assert_eq!(app.data.select_state.selected(), Some(0));
    }

    #[test]
    fn should_reset_timer_on_first_change_transition() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = Application::test();
        app.autosave.enabled = true;
        app.autosave.interval = Duration::from_secs(30);
        app.autosave.last_tick_had_changes = false;

        app.data.todos.push(Todo::new("Task", "", None));
        app.data.mark_as_dirty();

        let time_before = app.autosave.time_until_next_save();
        mock_tick(&mut app, Some(&path));

        assert_eq!(time_before, 30);
        assert!(app.autosave.last_tick_had_changes);
    }

    #[test]
    fn should_reset_flow_after_undo() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = Application::test();
        app.autosave.enabled = true;

        app.data.todos.push(Todo::new("Initial", "", None));
        let _ = app.data.save(Some(&path), &app.config.storage);
        assert!(!app.data.any_unsaved_changes());

        app.data.todos.push(Todo::new("Change", "", None));
        app.data.mark_as_dirty();
        mock_tick(&mut app, Some(&path));
        assert!(app.autosave.last_tick_had_changes);

        app.data.todos.pop();
        app.data.mark_as_dirty();

        assert!(
            !app.data.any_unsaved_changes(),
            "Data should be equal to saved state"
        );

        mock_tick(&mut app, Some(&path));
        assert!(
            !app.autosave.last_tick_had_changes,
            "Flag should reset when data is clean"
        );

        app.data.todos.push(Todo::new("New change", "", None));
        app.data.mark_as_dirty();
        mock_tick(&mut app, Some(&path));

        assert_eq!(app.autosave.time_until_next_save(), 30);
    }

    #[test]
    fn should_go_autosave_full_cycle() {
        let temp_dir = TempDir::new("todo_test").unwrap();
        let path = temp_dir.path().join("todos.json");

        let mut app = Application::test();
        app.autosave.enabled = true;
        app.autosave.interval = Duration::from_millis(20);
        app.autosave.debounce = Duration::from_millis(20);

        app.data.todos.push(Todo::new("Test Task", "", None));
        app.data.mark_as_dirty();

        mock_tick(&mut app, Some(&path));

        sleep(Duration::from_millis(30));
        app.autosave.register_activity();
        mock_tick(&mut app, Some(&path));

        assert!(app.data.any_unsaved_changes(), "Still debouncing");

        sleep(Duration::from_millis(30));
        mock_tick(&mut app, Some(&path));

        assert!(!app.data.any_unsaved_changes(), "Saved successfully");
        assert!(path.exists());
    }

    #[test]
    fn should_do_nothing_if_autosave_disabled() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = Application::test();
        app.autosave.enabled = false;
        app.autosave.interval = Duration::from_millis(0);

        app.data.todos.push(Todo::new("Hidden Task", "", None));
        app.data.mark_as_dirty();

        sleep(Duration::from_millis(5));
        mock_tick(&mut app, Some(&path));

        assert!(app.data.any_unsaved_changes());
        assert!(
            !path.exists(),
            "File should NOT be created when autosave is disabled"
        );
    }
}
