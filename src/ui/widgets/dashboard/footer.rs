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

/// Footer widget
pub struct FooterWidget<'a> {
    state: &'a ApplicationState,
    autosave: &'a Autosave,
}

impl<'a> FooterWidget<'a> {
    pub fn new(state: &'a ApplicationState, autosave: &'a Autosave) -> Self {
        Self { state, autosave }
    }

    /// Footer rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::{Block, Paragraph};

        let theme: &Theme = &ctx.theme;
        let palette: ThemePalette = theme.palette();
        let chunks: std::rc::Rc<[Rect]> = self.layout(area);

        ctx.render_widget(
            Block::default().style(Style::default().bg(palette.bg)),
            area,
        );

        let status_layout: std::rc::Rc<[Rect]> = self.status_layout(chunks[1]);

        ctx.render_widget(self.left_info(&ctx), status_layout[0]);

        if let Some(n) = &self.state.notification {
            if !n.is_expired() {
                n.render(ctx, status_layout[1]);
            }
        }

        ctx.render_widget(
            Paragraph::new(self.right_status(&ctx).right_aligned()),
            status_layout[2],
        );
    }

    /// Layout of whole footer bar
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
    fn left_info(&self, ctx: &RenderContext) -> Line<'_> {
        let palette: ThemePalette = ctx.palette();

        let as_status = if self.autosave.enabled {
            Span::styled(
                " Autosave: [ON]",
                Style::default().fg(ctx.color(palette.success)),
            )
        } else {
            Span::styled(" Autosave: [OFF]", Style::default().fg(palette.muted))
        };

        Line::from(vec![
            Span::styled(
                format!("[{}] ", ctx.theme),
                Style::default().fg(ctx.color(palette.warning)),
            ),
            Span::styled("|", Style::default().fg(ctx.color(palette.fg))),
            as_status,
        ])
    }

    /// Right part: (15s) ● Unsaved
    fn right_status(&self, ctx: &RenderContext) -> Line<'_> {
        let mut spans = Vec::new();
        let has_changes = self.state.any_unsaved_changes();
        let palette: ThemePalette = ctx.palette();

        if self.autosave.enabled && has_changes {
            if self.autosave.is_debouncing(has_changes) {
                spans.push(Span::styled("(waiting...) ", ctx.color(palette.warning)));
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

        spans.push(Span::styled(msg, Style::default().fg(ctx.color(color))));
        Line::from(spans)
    }
}

/// Unit-tests for footer bar
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{KeyMaps, StorageConfig},
        core::Storage,
        models::Task,
        state::{Session, UIState},
    };
    use std::time::Duration;
    use tempdir::TempDir;

    // Helper to extract text from line
    fn line_to_string(line: Line) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    #[test]
    fn should_render_left_info_with_theme_name_and_autosave() {
        let state = ApplicationState::default();
        let ui = UIState::default();
        let keymaps = KeyMaps::default();
        let mut autosave = Autosave::new(true);
        let theme_name = "Gruvbox Dark";

        let ctx = RenderContext::mock(&ui, &keymaps);

        let widget = FooterWidget::new(&state, &autosave);
        let text = line_to_string(widget.left_info(&ctx));
        assert!(text.contains("Autosave: [ON]"));
        assert!(text.contains(theme_name));

        autosave.enabled = false;
        let widget_off = FooterWidget::new(&state, &autosave);
        let text_off = line_to_string(widget_off.left_info(&ctx));
        assert!(text_off.contains("Autosave: [OFF]"));
    }

    #[test]
    fn should_render_right_status_with_timer_and_unsaved() {
        let mut state = ApplicationState::default();
        let ui = UIState::default();
        let keymaps = KeyMaps::default();
        let mut autosave = Autosave::new(true);

        state.tasks.push(Task::new("Test", "", None));
        state.mark_as_dirty();
        autosave.reset_timer();

        let ctx = RenderContext::mock(&ui, &keymaps);
        let widget = FooterWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&ctx));

        assert!(text.contains("30s"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_waiting_status_during_debounce() {
        let mut state = ApplicationState::default();
        let ui = UIState::default();
        let keymaps = KeyMaps::default();
        let mut autosave = Autosave::new(true);

        state.tasks.push(Task::new("Test", "", None));
        state.mark_as_dirty();

        autosave.interval = Duration::from_millis(0);
        autosave.register_activity();

        let ctx = RenderContext::mock(&ui, &keymaps);
        let widget = FooterWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&ctx));

        assert!(text.contains("(waiting...)"));
        assert!(text.contains("● Unsaved"));
    }

    #[test]
    fn should_render_only_saved_status_when_no_changes() {
        let temp_dir = TempDir::new("task_test").unwrap();
        let path = temp_dir.path().join("test_tasks.db");

        let mut state = ApplicationState::default();
        let ui = UIState::default();
        let keymaps = KeyMaps::default();
        let autosave = Autosave::new(true);

        state.tasks.clear();
        let session = Session::from_state(&ui, None);

        let mut storage: Storage = Storage::init(Some(&path), &StorageConfig::default()).unwrap();
        let _ = storage.save(&state.tasks, session);
        state.mark_saved();

        let ctx = RenderContext::mock(&ui, &keymaps);
        let widget = FooterWidget::new(&state, &autosave);
        let text = line_to_string(widget.right_status(&ctx));

        assert!(text.contains("✓ Saved"));
        assert!(!text.contains("s)"), "Should not contain timer when saved");
    }
}
