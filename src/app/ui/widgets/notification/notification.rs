use crate::app::{
    ui::renderer::state::Anchor,
    utils::constants::{size::*, theme::*},
};
use ratatui::{
    Frame,
    layout::{Alignment, Margin, Rect},
};
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
        use crate::app::utils::layout::{anchored, calculate_content_size};
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
        use super::utils::render_lines_based_on_notifcation;
        use ratatui::{
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
