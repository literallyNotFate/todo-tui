use crate::{
    enums::ApplicationMode,
    handlers::handle_key_event,
    state::{ApplicationState, UIState},
    ui::Renderer,
};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event},
    {DefaultTerminal, Frame},
};

pub struct Application {
    pub mode: ApplicationMode,
    pub state: ApplicationState,
    pub running: bool,
    pub ui: UIState,
    pub renderer: Renderer,
}

impl Application {
    pub fn new() -> Self {
        Self {
            mode: ApplicationMode::Browsing,
            state: ApplicationState::new(),
            running: true,
            ui: UIState::default(),
            renderer: Renderer,
        }
    }

    // Create mock application (for testing)
    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            state: ApplicationState::default(),
            ui: UIState::default(),
            mode: ApplicationMode::Browsing,
            running: true,
            renderer: Renderer,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        use std::time::{Duration, Instant};

        const TICK_RATE: Duration = Duration::from_millis(100);
        let mut last_tick: Instant = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout: Duration = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)?
                && let Event::Key(key_event) = event::read()?
                && let Ok(size) = terminal.size()
            {
                handle_key_event(self, key_event, (size.width, size.height));
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.tick();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    // Tick function (for notification)
    pub fn tick(&mut self) {
        self.ui.expire_notification(&mut self.state.notification);
    }

    // Synchronize selection after tab switching
    pub fn sync_ui(&mut self) {
        let indices = self.ui.current_filter.filter(&self.state.todos);
        let visible_count = indices.len();

        if visible_count == 0 {
            self.state.select_state.select(None);
        } else {
            let current = self.state.select_state.selected();
            match current {
                None => self.state.select_state.select(Some(0)),
                Some(idx) if idx >= visible_count => {
                    self.state.select_state.select(Some(visible_count - 1));
                }
                _ => {}
            }
        }
    }

    // Rendering
    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer
            .render(frame, &self.state, &self.ui, self.mode);
    }
}

// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{Filter, Todo},
        ui::Notification,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn should_create_application() {
        let app = Application::test();

        assert!(app.running, "running should be true by default");
        assert_eq!(app.state.todos.len(), 0, "todos should be empty");
        assert!(
            app.state.notification.is_none(),
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
        app.state.notification = Some(notification);

        app.tick();

        assert!(
            app.state.notification.is_none(),
            "expired notification should be removed"
        );
    }

    #[test]
    fn should_select_none_if_empty_with_sync_ui() {
        let mut app = Application::test();
        app.state.select_state.select(Some(5));
        app.state.todos.clear();

        app.sync_ui();

        assert_eq!(
            app.state.select_state.selected(),
            None,
            "If list is empty, should select None"
        );
    }

    #[test]
    fn should_adjust_out_of_bounds_index_with_sync_ui() {
        let mut app = Application::test();

        app.state.append(Todo::new("1", "", None)).unwrap();
        app.state.append(Todo::new("2", "", None)).unwrap();
        app.state.append(Todo::new("3", "", None)).unwrap();

        app.state.select_state.select(Some(2));
        app.state.todos.truncate(1);

        app.sync_ui();

        assert_eq!(
            app.state.select_state.selected(),
            Some(0),
            "Index must stay at last accessible task"
        );
    }

    #[test]
    fn should_initialize_selection_with_sync_ui() {
        let mut app = Application::test();

        app.state.append(Todo::new("Task", "", None)).unwrap();
        app.state.select_state.select(None);

        app.sync_ui();

        assert_eq!(
            app.state.select_state.selected(),
            Some(0),
            "If selection was None, but there are tasks, choose first"
        );
    }

    #[test]
    fn should_adjust_selection_on_filter_change_with_sync_ui() {
        let mut app = Application::test();

        app.state.append(Todo::new("Active", "", None)).unwrap();
        app.state.append(Todo::new("Done", "", None)).unwrap();
        app.state.todos[1].completed = true;

        app.ui.current_filter = Filter::Completed;
        app.state.select_state.select(Some(1));

        app.sync_ui();

        assert_eq!(app.state.select_state.selected(), Some(0));
    }
}
