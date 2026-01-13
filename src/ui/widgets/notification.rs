use crate::{state::Anchor, utils::widgets::notification::render_lines_based_on_notifcation};
use ratatui::{Frame, layout::Rect};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq)]
pub enum NotificationKind {
    Success,
    Error,
}

pub struct Notification {
    pub message: String,
    pub kind: NotificationKind,
    pub anchor: Anchor,
    pub duration: Duration,
    pub created_at: Instant,
}

impl Notification {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: NotificationKind::Success,
            anchor: Anchor::TopRight,
            duration: Duration::from_secs(3),
            created_at: Instant::now(),
        }
    }

    pub fn success_none() -> Self {
        Self::success("")
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: NotificationKind::Error,
            anchor: Anchor::TopRight,
            duration: Duration::from_secs(3),
            created_at: Instant::now(),
        }
    }

    pub fn error_none() -> Self {
        Self::error("")
    }

    // Calculate area for notification
    pub fn area(&self, frame_area: Rect) -> Rect {
        use crate::utils::{
            anchored, calculate_content_size, constants::size::NOTIFICATION_PERCENTAGE_WIDTH,
        };
        use ratatui::widgets::Padding;

        let top_title_len: usize = " Notification ".chars().count();
        let bottom_title_len: usize = " Closes in xxx seconds ".chars().count();

        let (width, height): (u16, u16) = calculate_content_size(
            frame_area,
            &self.message,
            top_title_len,
            bottom_title_len,
            Padding::uniform(1),
            NOTIFICATION_PERCENTAGE_WIDTH,
        );

        anchored(frame_area, width, height, self.anchor.clone())
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use crate::utils::constants::theme::{COLOR_GREEN, COLOR_RED};
        use ratatui::{
            layout::{Alignment, Margin},
            style::Style,
            text::Line,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let titles: (Line, Line) = render_lines_based_on_notifcation(self.remaining_secs() + 1);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(titles.0)
            .title_bottom(titles.1)
            .style(match self.kind {
                NotificationKind::Success => Style::default().fg(COLOR_GREEN),
                NotificationKind::Error => Style::default().fg(COLOR_RED),
            });

        let text = Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(block, area);
        frame.render_widget(text, area.inner(Margin::new(2, 2)));
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

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn duration(mut self, seconds: u64) -> Self {
        self.duration = Duration::from_secs(seconds);
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
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
