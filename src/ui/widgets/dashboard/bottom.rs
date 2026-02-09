use crate::{state::ApplicationState, theme::ThemeColors};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

/// Bottom bar widget
pub struct BottomBarWidget<'a> {
    state: &'a ApplicationState,
    theme: &'a ThemeColors,
}

impl<'a> BottomBarWidget<'a> {
    pub fn new(state: &'a ApplicationState, theme: &'a ThemeColors) -> Self {
        Self { state, theme }
    }

    /// Bottom bar rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::{layout::Alignment, style::Stylize, widgets::Paragraph};

        let chunks: std::rc::Rc<[Rect]> = self.layout(area);
        let status_layout: std::rc::Rc<[Rect]> = self.status_layout(chunks[1]);

        frame.render_widget(
            Paragraph::new("todo-tui").fg(self.theme.accent),
            status_layout[0],
        );

        if let Some(n) = &self.state.notification {
            if !n.is_expired() {
                n.render(frame, status_layout[1], self.theme);
            }
        }

        let (status_msg, status_color): (&str, Color) = self.status();

        frame.render_widget(
            Paragraph::new(status_msg)
                .alignment(Alignment::Right)
                .fg(status_color),
            status_layout[2],
        );
    }

    /// Layout of whole bottom bar
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1), // App name + Messages + Status
            ])
            .split(area)
    }

    /// Layout for status
    fn status_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // App name
                Constraint::Percentage(40), // Notification
                Constraint::Percentage(30), // Status
            ])
            .split(area)
    }

    /// Return status with corresponding theme color
    fn status(&self) -> (&str, Color) {
        if self.state.any_unsaved_changes() {
            ("● Unsaved ", self.theme.error)
        } else {
            ("✓ Saved ", self.theme.success)
        }
    }
}

/// Unit-tests for bottom bar
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Todo;

    #[test]
    fn should_return_status_for_bottom_bar() {
        let mut state: ApplicationState = ApplicationState::default();
        state.saved_todos_hash = state.hash_state();

        let mut bottom: BottomBarWidget = BottomBarWidget::new(&state, &ThemeColors::TOKYO_NIGHT);
        let mut status: (&str, Color) = bottom.status();

        assert!(!state.any_unsaved_changes());
        assert_eq!(status.0, "✓ Saved ");
        assert_eq!(status.1, Color::Rgb(158, 206, 106));

        state.todos.push(Todo::new("test", "test", None));
        bottom = BottomBarWidget::new(&state, &ThemeColors::TOKYO_NIGHT);
        status = bottom.status();

        assert!(state.any_unsaved_changes());
        assert_ne!(status.0, "✓ Saved ");
        assert_eq!(status.1, Color::Rgb(247, 118, 118));
    }
}
