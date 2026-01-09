// Unit-tests for notification widget
#[cfg(test)]
mod tests {
    use crate::app::ui::{
        renderer::state::Anchor,
        widgets::notification::{
            notification::{Notification, NotificationKind},
            utils::render_lines_based_on_notifcation,
        },
    };
    use ratatui::layout::Rect;
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

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

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn should_create_notification_success() {
        let notification: Notification = Notification::success("Success test");

        assert_eq!(notification.message, "Success test");
        assert_eq!(notification.kind, NotificationKind::Success);
        assert_eq!(notification.anchor, Anchor::TopRight);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_create_notification_success_without_title() {
        let notification: Notification = Notification::success_none();

        assert_eq!(notification.message, "");
        assert_eq!(notification.kind, NotificationKind::Success);
        assert_eq!(notification.anchor, Anchor::TopRight);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_create_notification_error() {
        let notification: Notification = Notification::error("Error test");

        assert_eq!(notification.message, "Error test");
        assert_eq!(notification.kind, NotificationKind::Error);
        assert_eq!(notification.anchor, Anchor::TopRight);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_create_notification_error_without_title() {
        let notification: Notification = Notification::error_none();

        assert_eq!(notification.message, "");
        assert_eq!(notification.kind, NotificationKind::Error);
        assert_eq!(notification.anchor, Anchor::TopRight);
        assert_eq!(notification.duration, Duration::from_secs(3));
        assert!(notification.created_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn should_create_notification_with_chaining_api() {
        let notification: Notification = Notification::success("Test")
            .with_message("Overridden")
            .duration(10)
            .anchor(Anchor::BottomLeft);

        assert_eq!(notification.message, "Overridden");
        assert_eq!(notification.duration, Duration::from_secs(10));
        assert_eq!(notification.anchor, Anchor::BottomLeft);
    }

    #[test]
    fn should_create_area_for_notification() {
        let notification: Notification = Notification::success("Hello").anchor(Anchor::TopRight);
        let frame_area: Rect = create_helper_frame();

        let area: Rect = notification.area(frame_area);

        assert!(area.x > 50);
        assert!(area.y < 10);
        assert!(area.width >= 20);
        assert!(area.height >= 3);
    }

    #[test]
    fn should_test_remaining_seconds_of_notification() {
        let now: Instant = Instant::now();
        let notification: Notification = Notification {
            message: "Test".to_string(),
            kind: NotificationKind::Success,
            anchor: Anchor::TopRight,
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
            anchor: Anchor::TopRight,
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
            anchor: Anchor::TopRight,
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
        let notification: Notification = Notification::success("Active").duration(5);

        app.notification = Some(notification);
        app.tick();

        assert!(app.notification.is_some());
        assert_eq!(app.notification.as_ref().unwrap().message, "Active");
    }

    // Utils
    #[test]
    fn should_render_lines_based_on_notifcation() {
        let (top, bottom) = render_lines_based_on_notifcation(3);
        assert_eq!(top.spans[0].content, " Notification ");
        assert!(bottom.spans[0].content.contains("Closes in "));
    }
}
