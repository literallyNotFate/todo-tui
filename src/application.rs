use crate::{
    config::{Config, KeyMaps},
    core::{ApplicationError, ApplicationMode, Autosave, FocusArea, Storage},
    events::EventHandler,
    state::{
        ApplicationResult, ApplicationState, Session, TasksStateData, TasksStateSave, UIState,
    },
    ui::Renderer,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::{self, Event},
        terminal,
    },
};

/// Tick rate of a application
const TICK_RATE: std::time::Duration = std::time::Duration::from_millis(100);

/// Application with renderer, state, config and autosave
pub struct Application {
    pub data: ApplicationState,
    pub ui: UIState,
    pub running: bool,
    pub renderer: Renderer,
    pub mode: ApplicationMode,
    pub autosave: Autosave,

    pub storage: Storage,
    pub config: Config,
    pub keymaps: KeyMaps,
    pub size: (u16, u16),

    ticks_count: u64,
}

impl Application {
    pub fn new(config: Config, config_error: Option<ApplicationError>) -> Self {
        let size: (u16, u16) = terminal::size().unwrap_or((100, 100));
        let (keymaps, keymaps_error) = Self::load_keymaps();

        let storage: Storage = Storage::init(None, &config.storage).unwrap_or_else(|e| {
            log::error!("Failed to initialize SQLite storage: {}", e);
            panic!("Critical storage error: {}", e);
        });
        let storage_data: TasksStateData = storage.load().unwrap_or_default();

        let mut ui: UIState = UIState::new(config.ui.clone());
        storage_data.session.apply_to(&mut ui);

        let mut app = Self {
            data: ApplicationState::new(storage_data.tasks, storage_data.folders),
            ui,
            autosave: Autosave::from(&config.storage),
            config,
            mode: ApplicationMode::Navigation,
            running: true,
            renderer: Renderer,
            keymaps,
            size,
            storage,
            ticks_count: 0,
        };

        app.sync_ui(storage_data.session.last_selected_id);
        if let Some(e) = config_error {
            log::warn!("Application: Config loaded with errors: {}", e);
            app.ui.show_result_popup(Err(e));
        }

        if let Some(e) = keymaps_error {
            log::warn!("Application: Keymaps loaded with errors: {}", e);
            app.ui.show_result_popup(Err(e));
        }

        log::debug!("Application: Instance created, terminal size: {:?}", size);
        app
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        log::info!("Run: Entering main event loop, tick rate 100 ms");
        let mut last_tick = std::time::Instant::now();
        self.ui.request_redraw();

        while self.running {
            let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)? {
                match event::read()? {
                    Event::Resize(w, h) => {
                        self.size = (w, h);
                        self.ui.request_redraw();
                    }
                    Event::Key(key_event) => {
                        self.autosave.register_activity();
                        EventHandler::handle_key(self, key_event);
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.tick();
                last_tick = std::time::Instant::now();
            }

            if self.ui.needs_redraw() {
                terminal.draw(|frame| self.render(frame))?;
                self.ui.clear_redraw_flag();
            }
        }

        log::info!("Run: Exiting main event loop");
        Ok(())
    }

    /// Rendering application using Renderer
    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer.render(
            frame,
            &mut self.data,
            &self.ui,
            self.mode,
            &self.autosave,
            &self.config,
            &self.keymaps,
        );
    }

    /// Tick function (for notification and autosave)
    pub fn tick(&mut self) {
        let ticks_per_second = 10;
        self.ticks_count = self.ticks_count.wrapping_add(1);

        if self.ui.expire_notification(&mut self.data.notification) {
            self.ui.request_redraw();
        }

        if self.ticks_count % (ticks_per_second * 20) == 0 && self.ui.config.use_system_theme {
            if self.ui.apply_system_theme() {
                log::info!("Theme changed by system, redrawing...");
                self.ui.request_redraw();
            }
        }

        let has_changes = self.data.any_unsaved_changes();

        if self.autosave.enabled {
            if has_changes && !self.autosave.last_tick_had_changes {
                self.autosave.reset_timer();
                self.ui.request_redraw();
            }

            if self.autosave.tick(has_changes) {
                self.ui.request_redraw();
            }

            if has_changes && self.autosave.should_save(has_changes) {
                log::debug!("Autosave: Triggered");
                self.config.update_from_ui(&self.ui);

                match self.save_all() {
                    Ok(_) => {
                        log::info!("Autosave: Successfully saved everything");
                        self.autosave.reset_timer();
                        let _ = self.config.save(None);
                        self.ui.request_redraw();
                    }
                    Err(e) => log::error!("Autosave: Failed to save: {}", e),
                }
            }
            self.autosave.last_tick_had_changes = has_changes;
        }
    }

    /// Saves current application state to SQLite database
    pub fn save_all(&mut self) -> ApplicationResult<()> {
        let current_id = self.data.selected_id(
            self.ui.active_tab,
            self.ui.active_folder,
            &self.ui.search_query(),
        );

        let session: Session = Session::from_state(&self.ui, current_id);
        self.storage.save(&TasksStateSave::new(
            &self.data.tasks,
            &self.data.folders,
            &session,
        ))?;
        self.data.mark_saved();
        Ok(())
    }

    /// Synchronizing selection after tab switching
    pub fn sync_ui(&mut self, target_id: Option<uuid::Uuid>) {
        let query: &str = self.ui.search_query();
        let filtered_ids: Vec<uuid::Uuid> = ApplicationState::filter(
            &self.data.tasks,
            self.ui.active_tab,
            self.ui.active_folder,
            query,
        )
        .map(|t| t.id)
        .collect();

        let id_to_find = target_id.or(self.data.last_selected_id);

        if let Some(id) = id_to_find {
            if let Some(pos) = filtered_ids.iter().position(|&uid| uid == id) {
                self.data.select_state.select(Some(pos));
                self.data.last_selected_id = Some(id);
                return;
            }
        }

        self.data.clamp_selection(filtered_ids.len());

        self.data.last_selected_id = self
            .data
            .select_state
            .selected()
            .and_then(|idx| filtered_ids.get(idx).cloned());

        log::trace!(
            "UI Sync: Tab: {:?}, Folder: {:?}, Query: '{}', Visible IDs: {}, Selected: {:?}",
            self.ui.active_tab,
            self.ui.active_folder,
            self.ui.search_query(),
            filtered_ids.len(),
            self.data.select_state.selected()
        );
    }

    /// Restoring base mode (after form exit)
    pub fn restore_base_mode(&mut self) {
        self.mode = match *self.ui.focused {
            FocusArea::Sidebar => ApplicationMode::Navigation,
            FocusArea::Main => ApplicationMode::List,
        };
        log::debug!(
            "Mode restored to {:?} based on focus {:?}",
            self.mode,
            *self.ui.focused
        );
    }

    /// Helper function to load config
    pub fn load_config() -> (Config, Option<ApplicationError>) {
        match Config::load(None) {
            Ok(cfg) => (cfg, None),
            Err(e) => (Config::default(), Some(e)),
        }
    }

    /// Helper function to load keymaps
    pub fn load_keymaps() -> (KeyMaps, Option<ApplicationError>) {
        match KeyMaps::load(None) {
            Ok(kmps) => (kmps, None),
            Err(e) => (KeyMaps::default(), Some(e)),
        }
    }
}

impl Default for Application {
    fn default() -> Self {
        let config: Config = Config::default();
        let storage: Storage =
            Storage::init(Some(&std::path::PathBuf::from(":memory:")), &config.storage)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to initialize default in-memory storage for Application: {}",
                        e
                    );
                });

        Self {
            data: ApplicationState::default(),
            ui: UIState::default(),
            mode: ApplicationMode::Navigation,
            running: true,
            renderer: Renderer,
            autosave: Autosave::new(false),
            size: (80, 24),
            config,
            keymaps: KeyMaps::default(),
            storage,
            ticks_count: 0,
        }
    }
}

/// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::Task, state::SidebarTab, ui::Notification};
    use std::{
        path::PathBuf,
        thread::sleep,
        time::{Duration, Instant},
    };
    use tempdir::TempDir;

    fn create_test_app() -> (Application, TempDir) {
        let temp_dir: TempDir = TempDir::new("app_structure_test").unwrap();
        let db_path: PathBuf = temp_dir.path().join("test_app.db");
        let config: Config = Config::default();
        let storage: Storage = Storage::init(Some(&db_path), &config.storage).unwrap();

        let app = Application {
            data: ApplicationState::default(),
            ui: UIState::default(),
            running: true,
            renderer: Renderer,
            mode: ApplicationMode::Navigation,
            autosave: Autosave::new(false),
            config,
            keymaps: KeyMaps::default(),
            size: (100, 100),
            storage,
            ticks_count: 0,
        };

        (app, temp_dir)
    }

    fn mock_tick(app: &mut Application) {
        let has_changes: bool = app.data.any_unsaved_changes();

        if app.autosave.enabled {
            if has_changes && !app.autosave.last_tick_had_changes {
                app.autosave.reset_timer();
            }

            let session = Session::from_state(&app.ui, None);
            if has_changes && app.autosave.should_save(has_changes) {
                if app
                    .storage
                    .save(&TasksStateSave::new(
                        &app.data.tasks,
                        &app.data.folders,
                        &session,
                    ))
                    .is_ok()
                {
                    app.data.mark_saved();
                    app.autosave.reset_timer();
                }
            }

            app.autosave.last_tick_had_changes = app.data.any_unsaved_changes();
        }
    }

    #[test]
    fn should_create_application() {
        let (app, _tmp) = create_test_app();

        assert!(app.running, "running should be true by default");
        assert_eq!(app.data.tasks.len(), 0, "tasks should be empty");
        assert!(
            app.data.notification.is_none(),
            "notification should be none"
        );
        assert!(app.ui.modal.is_none(), "modal should be none");
    }

    #[test]
    fn should_test_tick_expires_notification() {
        let (mut app, _tmp) = create_test_app();
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
        let (mut app, _tmp) = create_test_app();
        app.data.select_state.select(Some(5));
        app.data.tasks.clear();

        app.sync_ui(None);

        assert_eq!(
            app.data.select_state.selected(),
            None,
            "If list is empty, should select None"
        );
    }

    #[test]
    fn should_adjust_out_of_bounds_index_with_sync_ui() {
        let (mut app, _tmp) = create_test_app();

        app.data.tasks.push(Task::new("1"));
        app.data.tasks.push(Task::new("2"));
        app.data.tasks.push(Task::new("3"));

        app.data.select_state.select(Some(2));
        app.data.tasks.truncate(1);

        app.sync_ui(None);

        assert_eq!(
            app.data.select_state.selected(),
            Some(0),
            "Index must stay at last accessible task"
        );
    }

    #[test]
    fn should_initialize_selection_with_sync_ui() {
        let (mut app, _tmp) = create_test_app();

        app.data.tasks.push(Task::new("Task"));
        app.data.select_state.select(None);

        app.sync_ui(None);

        assert_eq!(
            app.data.select_state.selected(),
            Some(0),
            "If selection was None, but there are tasks, choose first"
        );
    }

    #[test]
    fn should_adjust_selection_on_tab_change_with_sync_ui() {
        let (mut app, _tmp) = create_test_app();

        app.data.tasks.push(Task::new("Active"));
        app.data.tasks.push(Task::new("Done"));
        app.data.tasks[1].completed = true;

        app.ui.active_tab = SidebarTab::Completed;
        app.data.select_state.select(Some(1));

        app.sync_ui(None);

        assert_eq!(app.data.select_state.selected(), Some(0));
    }

    #[test]
    fn should_reset_timer_on_first_change_transition() {
        let (mut app, _tmp) = create_test_app();
        app.autosave.enabled = true;
        app.autosave.interval = Duration::from_secs(30);
        app.autosave.last_tick_had_changes = false;

        app.data.tasks.push(Task::new("Task"));
        app.data.mark_as_dirty();

        let time_before = app.autosave.time_until_next_save();
        mock_tick(&mut app);

        assert_eq!(time_before, 30);
        assert!(app.autosave.last_tick_had_changes);
    }

    #[test]
    fn should_reset_flow_after_undo() {
        let (mut app, _tmp) = create_test_app();
        app.autosave.enabled = true;

        app.data.tasks.push(Task::new("Initial"));

        let session = Session::default();
        let save_data = TasksStateSave::new(&app.data.tasks, &app.data.folders, &session);
        assert!(app.storage.save(&save_data).is_ok());
        app.data.mark_saved();
        assert!(!app.data.any_unsaved_changes());

        app.data.tasks.push(Task::new("Change"));
        app.data.mark_as_dirty();
        mock_tick(&mut app);
        assert!(app.autosave.last_tick_had_changes);

        app.data.tasks.pop();
        app.data.mark_as_dirty();

        assert!(
            !app.data.any_unsaved_changes(),
            "Data should be equal to saved state"
        );

        mock_tick(&mut app);
        assert!(
            !app.autosave.last_tick_had_changes,
            "Flag should reset when data is clean"
        );

        app.data.tasks.push(Task::new("New change"));
        app.data.mark_as_dirty();
        mock_tick(&mut app);

        assert_eq!(app.autosave.time_until_next_save(), 30);
    }

    #[test]
    fn should_go_autosave_full_cycle() {
        let (mut app, _tmp) = create_test_app();
        app.autosave.enabled = true;
        app.autosave.interval = Duration::from_millis(20);
        app.autosave.debounce = Duration::from_millis(20);

        app.data.tasks.push(Task::new("Test Task"));
        app.data.mark_as_dirty();

        mock_tick(&mut app);

        sleep(Duration::from_millis(30));
        app.autosave.register_activity();
        mock_tick(&mut app);

        assert!(app.data.any_unsaved_changes(), "Still debouncing");

        sleep(Duration::from_millis(30));
        mock_tick(&mut app);

        assert!(!app.data.any_unsaved_changes(), "Saved successfully");

        let loaded = app.storage.load().unwrap();
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks[0].title, "Test Task");
    }

    #[test]
    fn should_do_nothing_if_autosave_disabled() {
        let (mut app, _tmp) = create_test_app();
        app.autosave.enabled = false;
        app.autosave.interval = Duration::from_millis(0);

        app.data.tasks.push(Task::new("Hidden Task"));
        app.data.mark_as_dirty();

        sleep(Duration::from_millis(5));
        mock_tick(&mut app);

        assert!(app.data.any_unsaved_changes());

        let loaded = app.storage.load().unwrap();
        assert!(
            loaded.tasks.is_empty(),
            "Storage should NOT be updated when autosave is disabled"
        );
    }
}
