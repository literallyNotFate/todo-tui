use crate::{
    core::{ApplicationMode, Autosave},
    enums::FocusArea,
    events::EventHandler,
    state::{ApplicationState, UIState},
    ui::Renderer,
};
use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::{self, Event},
        terminal,
    },
};

pub struct Application {
    pub data: ApplicationState,
    pub ui: UIState,
    pub running: bool,
    pub renderer: Renderer,
    pub mode: ApplicationMode,
    pub autosave: Autosave,

    pub size: (u16, u16),
}

impl Application {
    pub fn new() -> Self {
        let size: (u16, u16) = terminal::size().unwrap_or((100, 100));
        Self {
            mode: ApplicationMode::Browsing,
            data: ApplicationState::new(),
            running: true,
            ui: UIState::default(),
            renderer: Renderer,
            autosave: Autosave::new(false),
            size,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        use std::time::{Duration, Instant};

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
        self.renderer.render(
            frame,
            &mut self.data,
            &self.ui,
            self.mode,
            self.autosave.enabled,
        );
    }

    /// Tick function (for notification and autosave)
    pub fn tick(&mut self) {
        self.ui.expire_notification(&mut self.data.notification);

        if self.autosave.should_save(self.data.any_unsaved_changes()) {
            if let Err(e) = self.data.save(None) {
                self.ui.push_notification(&mut self.data, Err(e));
            }

            self.autosave.reset_timer();
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
        }
    }
}

/// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;
    use crate::{
        models::{Filter, Todo},
        ui::Notification,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        thread::sleep,
        time::{Duration, Instant},
    };

    fn mock_tick(app: &mut Application, path: Option<&Path>) {
        if app.autosave.should_save(app.data.any_unsaved_changes()) {
            if let Err(e) = app.data.save(path) {
                app.ui.push_notification(&mut app.data, Err(e));
            }

            app.autosave.reset_timer();
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
    fn should_go_autosave_full_cycle() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut app = Application::test();
        app.autosave.enabled = true;
        app.autosave.interval = Duration::from_millis(10);
        app.autosave.debounce = Duration::from_millis(10);

        app.data.todos.push(Todo::new("Test Task", "Desc", None));
        app.data.mark_as_dirty();
        assert!(app.data.any_unsaved_changes());

        mock_tick(&mut app, Some(&path));
        assert!(
            app.data.any_unsaved_changes(),
            "Should NOT save because debounce is active"
        );

        sleep(Duration::from_millis(15));
        app.autosave.register_activity();
        mock_tick(&mut app, Some(&path));
        assert!(
            app.data.any_unsaved_changes(),
            "Should NOT save because user is active"
        );

        sleep(Duration::from_millis(15));
        mock_tick(&mut app, Some(&path));

        assert!(!app.data.any_unsaved_changes(), "Data should be saved");
        assert!(path.exists(), "File was not created");

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Test Task"), "Saved content is incorrect");
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
