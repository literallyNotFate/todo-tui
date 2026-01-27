use crate::theme::ThemeColors;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum NotificationKind {
    Success,
    Error,
}

#[derive(Debug)]
pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
    pub duration: Duration,
    pub created_at: Instant,
}

impl Notification {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: NotificationKind::Success,
            duration: Duration::from_secs(3),
            created_at: Instant::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: NotificationKind::Error,
            duration: Duration::from_secs(3),
            created_at: Instant::now(),
        }
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect, theme: &ThemeColors) {
        use ratatui::style::Style;
        use ratatui::widgets::Paragraph;

        let (color, icon) = match self.kind {
            NotificationKind::Success => (theme.success, "✔"),
            NotificationKind::Error => (theme.error, "✘"),
        };

        let text = format!("{} {} ({}s)", icon, self.message, self.remaining_secs() + 1);

        let text_block = Paragraph::new(text)
            .style(Style::default().fg(color))
            .alignment(Alignment::Center);

        frame.render_widget(text_block, area);
    }

    // To count remaining seconds for title bottom and check if expired that period
    pub fn remaining_secs(&self) -> u64 {
        self.duration
            .saturating_sub(self.created_at.elapsed())
            .as_secs()
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }
}

// Unit-tests for notification widget
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    // Mock application for tick() and automatic notifcation closing
    struct MockApplication {
        pub notification: Option<Notification>,
    }

    impl MockApplication {
        pub fn new() -> Self {
            Self { notification: None }
        }

        pub fn tick(&mut self) {
            if let Some(n) = &self.notification
                && n.created_at.elapsed() >= n.duration
            {
                self.notification = None;
            }
        }
    }

    #[test]
    fn should_create_notification_success() {
        let notification: Notification = Notification::success("Success test");

        assert_eq!(notification.message, "Success test");
        assert_eq!(notification.kind, NotificationKind::Success);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_create_notification_error() {
        let notification: Notification = Notification::error("Error test");

        assert_eq!(notification.message, "Error test");
        assert_eq!(notification.kind, NotificationKind::Error);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_test_remaining_seconds_of_notification() {
        let now: Instant = Instant::now();
        let notification: Notification = Notification {
            message: "Test".to_string(),
            kind: NotificationKind::Success,
            duration: Duration::from_secs(5),
            created_at: now,
        };

        assert!(notification.remaining_secs() <= 4);

        sleep(Duration::from_secs(1));
        assert!(notification.remaining_secs() <= 3);
    }

    #[test]
    fn should_notification_be_expired() {
        let notification: Notification = Notification {
            message: "Test".to_string(),
            kind: NotificationKind::Success,
            duration: Duration::from_millis(100),
            created_at: Instant::now(),
        };

        assert!(!notification.is_expired());

        sleep(Duration::from_millis(150));
        assert!(notification.is_expired());
    }

    #[test]
    fn should_remove_notification_after_time_expired() {
        let mut app: MockApplication = MockApplication::new();

        let past: Instant = Instant::now() - Duration::from_secs(4);
        let notification: Notification = Notification {
            message: "Expired".to_string(),
            kind: NotificationKind::Success,
            duration: Duration::from_secs(3),
            created_at: past,
        };

        app.notification = Some(notification);
        app.tick();

        assert!(app.notification.is_none());
    }

    #[test]
    fn should_keep_active_notification_if_not_expired() {
        let mut app: MockApplication = MockApplication::new();
        let notification: Notification = Notification::success("Active");

        app.notification = Some(notification);
        app.tick();

        assert!(app.notification.is_some());
        assert_eq!(app.notification.as_ref().unwrap().message, "Active");
    }
}
