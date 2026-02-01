use crate::{state::ApplicationState, theme::ThemeColors};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
};

pub struct MenuBottomBar;

impl MenuBottomBar {
    pub fn render(frame: &mut Frame, area: Rect, state: &ApplicationState, theme: &ThemeColors) {
        use ratatui::{layout::Alignment, style::Stylize, widgets::Paragraph};

        let chunks: std::rc::Rc<[Rect]> = Self::layout(area);
        let status_layout: std::rc::Rc<[Rect]> = Self::status_layout(chunks[1]);

        frame.render_widget(
            Paragraph::new("todo-tui").fg(theme.accent),
            status_layout[0],
        );

        if let Some(n) = &state.notification {
            if !n.is_expired() {
                n.render(frame, status_layout[1], theme);
            }
        }

        let (status_msg, status_color): (&str, Color) = Self::status(state, theme);

        frame.render_widget(
            Paragraph::new(status_msg)
                .alignment(Alignment::Right)
                .fg(status_color),
            status_layout[2],
        );
    }

    // Layout of whole bottom bar
    fn layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1), // App name + Messages + Status
            ])
            .split(area)
    }

    // Layout for status
    fn status_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // App name
                Constraint::Percentage(40), // Notification
                Constraint::Percentage(30), // Status
            ])
            .split(area)
    }

    // Return status with corresponding theme color
    fn status(state: &ApplicationState, theme: &ThemeColors) -> (&'static str, Color) {
        if state.any_unsaved_changes() {
            ("● Unsaved ", theme.error)
        } else {
            ("✓ Saved ", theme.success)
        }
    }
}

// Unit-tests for bottom bar
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Todo;

    #[test]
    fn should_return_status_for_bottom_bar() {
        let mut state: ApplicationState = ApplicationState::default();
        state.saved_todos_hash = state.calculate_todos_hash();
        let mut status = MenuBottomBar::status(&state, &ThemeColors::TOKYO_NIGHT);

        assert!(!state.any_unsaved_changes());
        assert_eq!(status.0, "✓ Saved ");
        assert_eq!(status.1, Color::Rgb(158, 206, 106));

        state.append(Todo::new("test", "test", None)).unwrap();
        status = MenuBottomBar::status(&state, &ThemeColors::TOKYO_NIGHT);

        assert!(state.any_unsaved_changes());
        assert_ne!(status.0, "✓ Saved ");
        assert_eq!(status.1, Color::Rgb(247, 118, 118));
    }
}
