use crate::{core::Autosave, state::ApplicationState, theme::ThemeColors};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
};

/// Bottom bar widget
pub struct BottomBarWidget<'a> {
    state: &'a ApplicationState,
    autosave: &'a Autosave,
    theme: &'a ThemeColors,
}

impl<'a> BottomBarWidget<'a> {
    pub fn new(
        state: &'a ApplicationState,
        autosave: &'a Autosave,
        theme: &'a ThemeColors,
    ) -> Self {
        Self {
            state,
            autosave,
            theme,
        }
    }

    /// Bottom bar rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Paragraph;

        let chunks: std::rc::Rc<[Rect]> = self.layout(area);
        let status_layout: std::rc::Rc<[Rect]> = self.status_layout(chunks[1]);

        frame.render_widget(self.left_info(), status_layout[0]);

        if let Some(n) = &self.state.notification {
            if !n.is_expired() {
                n.render(frame, status_layout[1], self.theme);
            }
        }

        frame.render_widget(
            Paragraph::new(self.right_status()).right_aligned(),
            status_layout[2],
        );
    }

    /// Layout of whole bottom bar
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1), // Theme w/autosave + Messages + Status
            ])
            .split(area)
    }

    /// Layout for status
    fn status_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Theme + Autosave
                Constraint::Percentage(40), // Notification
                Constraint::Percentage(30), // Status + Timer
            ])
            .split(area)
    }

    /// Left part: [Theme] | Autosave: [ON]/[OFF]
    fn left_info(&self) -> Line<'_> {
        let as_status = if self.autosave.enabled {
            Span::styled(" Autosave: [ON]", Style::default().fg(self.theme.success))
        } else {
            Span::styled(" Autosave: [OFF]", Style::default().fg(self.theme.text_dim))
        };

        Line::from(vec![
            Span::styled(
                format!("[{}] ", self.theme.name),
                Style::default().fg(self.theme.warning),
            ),
            Span::styled("|", Style::default().fg(self.theme.text_primary)),
            as_status,
        ])
    }

    /// Right part: (15s) ● Unsaved
    fn right_status(&self) -> Line<'_> {
        let mut spans = Vec::new();
        let has_changes = self.state.any_unsaved_changes();

        if self.autosave.enabled && has_changes {
            if self.autosave.is_debouncing(has_changes) {
                spans.push(Span::styled("(waiting...) ", self.theme.warning));
            } else {
                let s = self.autosave.time_until_next_save();
                spans.push(Span::styled(format!("({}s) ", s), self.theme.text_dim));
            }
        }

        let (msg, color) = if has_changes {
            ("● Unsaved ", self.theme.error)
        } else {
            ("✓ Saved ", self.theme.success)
        };

        spans.push(Span::styled(msg, Style::default().fg(color)));
        Line::from(spans)
    }
}

/// Unit-tests for bottom bar
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Todo;
    use std::{path::PathBuf, time::Duration};
    use tempdir::TempDir;

    fn line_to_string(line: Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn should_render_left_info_with_theme_name_and_autosave() {
        let state = ApplicationState::default();
        let theme = ThemeColors::default();

        let mut autosave = Autosave::new(true);
        let widget = BottomBarWidget::new(&state, &autosave, &theme);
        let text = line_to_string(widget.left_info());
        assert!(text.contains("Autosave: [ON]"));
        assert!(text.contains(theme.name));

        autosave.enabled = false;
        let widget_off = BottomBarWidget::new(&state, &autosave, &theme);
        let text_off = line_to_string(widget_off.left_info());
        assert!(text_off.contains("Autosave: [OFF]"));
    }

    #[test]
    fn should_render_right_status_with_timer_and_unsaved() {
        let mut state = ApplicationState::default();
        let theme = ThemeColors::default();
        let mut autosave = Autosave::new(true);

        state.todos.push(Todo::new("Test", "", None));
        state.mark_as_dirty();
        autosave.reset_timer();

        let widget = BottomBarWidget::new(&state, &autosave, &theme);
        let text = line_to_string(widget.right_status());

        assert!(text.contains("30s"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_waiting_status_during_debounce() {
        let mut state = ApplicationState::default();
        let theme = ThemeColors::default();
        let mut autosave = Autosave::new(true);

        state.todos.push(Todo::new("Test", "", None));
        state.mark_as_dirty();

        autosave.interval = Duration::from_millis(0);
        autosave.register_activity();

        let widget = BottomBarWidget::new(&state, &autosave, &theme);
        let text = line_to_string(widget.right_status());

        assert!(text.contains("(waiting...)"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_only_saved_status_when_no_changes() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut state = ApplicationState::default();
        let theme = ThemeColors::default();
        let autosave = Autosave::new(true);

        state.todos.clear();
        let _ = state.save(Some(&path));

        assert!(
            !state.any_unsaved_changes(),
            "State must be clean for this test"
        );

        let widget = BottomBarWidget::new(&state, &autosave, &theme);
        let text = line_to_string(widget.right_status());

        assert!(text.contains("✓ Saved"));
        assert!(!text.contains("s)"), "Should not contain timer when saved");
    }
}
