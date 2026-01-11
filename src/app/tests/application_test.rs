// Unit-tests for application structure
#[cfg(test)]
mod tests {
    use crate::app::{
        state::state::ApplicationState,
        ui::{
            renderer::state::{Anchor, UIState},
            widgets::notification::notification::{Notification, NotificationKind},
        },
    };
    use std::time::{Duration, Instant};

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
