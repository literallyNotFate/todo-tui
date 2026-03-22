use crate::{theme::ThemePalette, ui::RenderContext};
use ratatui::{layout::Rect, style::Color};
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

    #[cfg(not(test))]
    last_tick_secs: u64,
    #[cfg(test)]
    pub last_tick_secs: u64,
}

impl Notification {
    /// Creating success notification template
    pub fn success(message: impl Into<String>) -> Self {
        let duration: Duration = Duration::from_secs(3);
        let created_at: Instant = Instant::now();
        Self {
            message: message.into(),
            kind: NotificationKind::Success,
            duration,
            created_at,
            last_tick_secs: duration.as_secs(),
        }
    }

    /// Creating error notification template
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: NotificationKind::Error,
            ..Self::success(message)
        }
    }

    /// Notification rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::{style::Style, widgets::Paragraph};

        let available_width = (area.width as usize).saturating_sub(5);
        let truncated_message = RenderContext::truncate(&self.message, available_width);
        let palette: ThemePalette = ctx.palette();

        let (icon, color) = self.icon_with_color(&palette);
        let text = format!(
            "{} {} ({}s)",
            icon,
            truncated_message,
            self.remaining_secs() + 1
        );

        let text_block = Paragraph::new(text)
            .style(Style::default().fg(color))
            .centered();

        ctx.render_widget(text_block, area);
    }

    /// To count remaining seconds for title bottom and check if that time has been expired
    pub fn remaining_secs(&self) -> u64 {
        self.duration
            .saturating_sub(self.created_at.elapsed())
            .as_secs()
    }

    /// Check whether notification duration time is passed
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() >= self.duration
    }

    /// Updates the current state of a timer.
    /// Returns true, if sec changed and needs redrawing.
    pub fn tick(&mut self) -> bool {
        let current_remaining: u64 = self.remaining_secs();

        if current_remaining != self.last_tick_secs {
            self.last_tick_secs = current_remaining;
            return true;
        }

        false
    }

    /// Get icon with corresponding color
    fn icon_with_color(&self, palette: &ThemePalette) -> (&'static str, Color) {
        match self.kind {
            NotificationKind::Success => ("✔", palette.success),
            NotificationKind::Error => ("✘", palette.error),
        }
    }

    /// Testing method to create expired notification
    #[cfg(test)]
    pub fn with_age(mut self, secs: u64) -> Self {
        self.created_at = Instant::now() - Duration::from_secs(secs);
        self
    }

    /// Testing method to set duration
    #[cfg(test)]
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

/// Unit-tests for notification widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    /// Mock application for tick() and automatic notifcation closing
    struct MockApplication {
        pub notification: Option<Notification>,
    }

    impl MockApplication {
        pub fn new() -> Self {
            Self { notification: None }
        }

        pub fn tick(&mut self) -> bool {
            if let Some(n) = &mut self.notification {
                if n.is_expired() {
                    self.notification = None;
                    return true;
                }
                if n.tick() {
                    return true;
                }
            }
            false
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
    fn should_return_notification_icon_and_color_based_on_kind_and_theme() {
        let mut notification: Notification = Notification::success("Success");
        let mut res = notification.icon_with_color(&ThemeName::CatppuccinMocha.palette());

        assert_eq!(notification.message, "Success");
        assert_eq!(notification.kind, NotificationKind::Success);
        assert_eq!(res.0, "✔");
        assert_eq!(res.1, Color::Rgb(166, 227, 161));

        notification = Notification::error("Error");
        res = notification.icon_with_color(&ThemeName::CatppuccinMocha.palette());

        assert_eq!(notification.message, "Error");
        assert_eq!(notification.kind, NotificationKind::Error);
        assert_eq!(res.0, "✘");
        assert_eq!(res.1, Color::Rgb(243, 139, 168));
    }

    #[test]
    fn should_return_true_when_second_changes() {
        let mut n = Notification::success("Test");
        n.duration = Duration::from_secs(10);

        n = n.with_age(0);
        n.last_tick_secs = n.remaining_secs();

        assert!(!n.tick());

        n = n.with_age(2);
        assert!(n.tick(), "Should return true because 10 -> 8");
    }

    #[test]
    fn should_test_remaining_seconds_of_notification() {
        let notification = Notification::success("Test").with_duration(Duration::from_secs(5));
        assert!(notification.remaining_secs() <= 5);

        let aged_notification = notification.with_age(2);
        assert!(aged_notification.remaining_secs() <= 3);
    }

    #[test]
    fn should_notification_be_expired() {
        let notification = Notification::success("Test").with_duration(Duration::from_millis(100));
        assert!(!notification.is_expired());

        let expired_notification = notification.with_age(1);
        assert!(expired_notification.is_expired());
    }

    #[test]
    fn should_remove_notification_after_time_expired() {
        let mut app = MockApplication::new();
        app.notification = Some(Notification::success("Expired").with_age(4));

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
