use crate::{
    core::Autosave,
    state::ApplicationState,
    theme::{Theme, ThemePalette},
    ui::RenderContext,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
};

/// Bottom bar widget
pub struct BottomBarWidget<'a> {
    state: &'a ApplicationState,
    autosave: &'a Autosave,
}

impl<'a> BottomBarWidget<'a> {
    pub fn new(state: &'a ApplicationState, autosave: &'a Autosave) -> Self {
        Self { state, autosave }
    }

    /// Bottom bar rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::{Block, Paragraph};

        let theme: Theme = ctx.theme;
        let palette: ThemePalette = theme.palette();
        let chunks: std::rc::Rc<[Rect]> = self.layout(area);

        ctx.render_widget(
            Block::default().style(Style::default().bg(palette.bg)),
            area,
        );

        let status_layout: std::rc::Rc<[Rect]> = self.status_layout(chunks[1]);

        ctx.render_widget(
            self.left_info(theme.name.display_name(), &ctx.palette()),
            status_layout[0],
        );

        if let Some(n) = &self.state.notification {
            if !n.is_expired() {
                n.render(ctx, status_layout[1]);
            }
        }

        ctx.render_widget(
            Paragraph::new(self.right_status(&ctx.palette())).right_aligned(),
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
    fn left_info(&self, name: &str, palette: &ThemePalette) -> Line<'_> {
        let as_status = if self.autosave.enabled {
            Span::styled(" Autosave: [ON]", Style::default().fg(palette.success))
        } else {
            Span::styled(" Autosave: [OFF]", Style::default().fg(palette.muted))
        };

        Line::from(vec![
            Span::styled(format!("[{}] ", name), Style::default().fg(palette.warning)),
            Span::styled("|", Style::default().fg(palette.fg)),
            as_status,
        ])
    }

    /// Right part: (15s) ● Unsaved
    fn right_status(&self, palette: &ThemePalette) -> Line<'_> {
        let mut spans = Vec::new();
        let has_changes = self.state.any_unsaved_changes();

        if self.autosave.enabled && has_changes {
            if self.autosave.is_debouncing(has_changes) {
                spans.push(Span::styled("(waiting...) ", palette.warning));
            } else {
                let s = self.autosave.time_until_next_save();
                spans.push(Span::styled(format!("({}s) ", s), palette.muted));
            }
        }

        let (msg, color) = if has_changes {
            ("● Unsaved ", palette.error)
        } else {
            ("✓ Saved ", palette.success)
        };

        spans.push(Span::styled(msg, Style::default().fg(color)));
        Line::from(spans)
    }
}

/// Unit-tests for bottom bar
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::StorageConfig, models::Todo, theme::ThemeName};
    use std::{path::PathBuf, time::Duration};
    use tempdir::TempDir;

    fn line_to_string(line: Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn should_render_left_info_with_theme_name_and_autosave() {
        let state = ApplicationState::default();
        let theme: Theme = Theme::default();

        let mut autosave = Autosave::new(true);
        let widget = BottomBarWidget::new(&state, &autosave);
        let text = line_to_string(widget.left_info(theme.name.display_name(), &theme.palette()));
        assert!(text.contains("Autosave: [ON]"));
        assert!(text.contains(theme.name.display_name()));

        autosave.enabled = false;
        let widget_off = BottomBarWidget::new(&state, &autosave);
        let text_off =
            line_to_string(widget_off.left_info(theme.name.display_name(), &theme.palette()));
        assert!(text_off.contains("Autosave: [OFF]"));
    }

    #[test]
    fn should_render_right_status_with_timer_and_unsaved() {
        let mut state = ApplicationState::default();
        let mut autosave = Autosave::new(true);
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();

        state.todos.push(Todo::new("Test", "", None));
        state.mark_as_dirty();
        autosave.reset_timer();

        let widget = BottomBarWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&palette));

        assert!(text.contains("30s"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_waiting_status_during_debounce() {
        let mut state = ApplicationState::default();
        let mut autosave = Autosave::new(true);
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();

        state.todos.push(Todo::new("Test", "", None));
        state.mark_as_dirty();

        autosave.interval = Duration::from_millis(0);
        autosave.register_activity();

        let widget = BottomBarWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&palette));

        assert!(text.contains("(waiting...)"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_only_saved_status_when_no_changes() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut state = ApplicationState::default();
        let autosave = Autosave::new(true);
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();

        state.todos.clear();
        let _ = state.save(Some(&path), &StorageConfig::default());

        assert!(
            !state.any_unsaved_changes(),
            "State must be clean for this test"
        );

        let widget = BottomBarWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&palette));

        assert!(text.contains("✓ Saved"));
        assert!(!text.contains("s)"), "Should not contain timer when saved");
    }
}
