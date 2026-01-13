use crate::{
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
    pub state: ApplicationState,
    pub running: bool,
    pub ui: UIState,
    pub renderer: Renderer,
}

impl Application {
    pub fn new() -> Self {
        Self {
            state: ApplicationState::new(),
            running: true,
            ui: UIState::default(),
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
        self.ui.expire_notification();
    }

    // Rendering
    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer.render(
            frame,
            &self.state.todos,
            &mut self.state.select_state,
            &self.ui,
        );
    }
}

// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use super::*;

    // Mock application structure
    struct MockApplication {
        running: bool,
        state: ApplicationState,
        ui: UIState,
    }

    impl MockApplication {
        pub fn new() -> Self {
            Self {
                running: true,
                state: ApplicationState::default(),
                ui: UIState::default(),
            }
        }

        pub fn tick(&mut self) {
            if let Some(n) = &self.ui.notification
                && n.is_expired()
            {
                self.ui.notification = None;
            }
        }
    }

    #[test]
    fn should_create_application() {
        let app = MockApplication::new();

        assert!(app.running, "running should be true by default");
        assert_eq!(app.state.todos.len(), 0, "todos should be empty");
        assert!(app.ui.notification.is_none(), "notification should be none");
        assert!(app.ui.input.is_none(), "input should be none");
        assert!(app.ui.dialog.is_none(), "dialog should be none");
    }

    #[test]
    fn should_test_tick_expires_notification() {
        use crate::{
            state::Anchor,
            ui::{Notification, NotificationKind},
        };
        use std::time::{Duration, Instant};

        let mut app = MockApplication::new();

        let old_time = Instant::now() - Duration::from_secs(10);
        app.ui.notification = Some(Notification {
            created_at: old_time,
            duration: Duration::from_secs(5),
            message: String::from("Test"),
            kind: NotificationKind::Success,
            anchor: Anchor::TopRight,
        });

        app.tick();

        assert!(
            app.ui.notification.is_none(),
            "expired notification should be removed"
        );
    }
}
