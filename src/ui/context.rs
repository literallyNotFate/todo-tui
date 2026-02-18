use crate::{
    core::ApplicationMode, enums::FocusArea, state::UIState, theme::ThemeColors,
    ui::is_terminal_small,
};
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::Style,
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

/// A render context provider for widgets
pub struct RenderContext<'a, 'b> {
    pub frame: &'a mut Frame<'b>,
    pub theme: ThemeColors,

    mode: ApplicationMode,
    focus: FocusArea,
    is_small: bool,
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn new(frame: &'a mut Frame<'b>, ui: &UIState, mode: ApplicationMode) -> Self {
        let area = frame.area();

        let theme = ui.theme.colors();
        let focus = ui.focus_area;

        Self {
            frame,
            theme,
            mode,
            focus,
            is_small: is_terminal_small(area.width, area.height),
        }
    }

    /// Render widget wrapper
    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        self.frame.render_widget(widget, area);
    }

    /// Render stateful widget wrapper
    pub fn render_stateful_widget<W, S>(&mut self, widget: W, area: Rect, state: &mut S)
    where
        W: StatefulWidget<State = S>,
    {
        self.frame.render_stateful_widget(widget, area, state);
    }

    /// Returns is_small value
    pub fn is_small(&self) -> bool {
        self.is_small
    }

    /// Returns mode value
    pub fn mode(&self) -> ApplicationMode {
        self.mode
    }

    /// Mode hotkeys wrapper
    pub fn hotkeys(&self) -> Vec<Line<'static>> {
        self.mode.hotkeys(&self.theme, &self.focus)
    }

    /// Set cursor position wrapper for input
    pub fn set_cursor_position(&mut self, position: Position) {
        self.frame.set_cursor_position(position);
    }

    /// Helper: is that widget being foucused on?
    pub fn is_focused(&self, area: FocusArea) -> bool {
        self.focus == area
    }

    /// Helper: what border style to have (based on FocusArea)?
    pub fn focused_style(&self, target: FocusArea) -> Style {
        if self.is_focused(target) {
            Style::default().fg(self.theme.accent)
        } else {
            Style::default().fg(self.theme.border)
        }
    }

    /// Creates basic block with border focusing
    pub fn block(&self, title: impl Into<String>, focus: FocusArea) -> Block<'static> {
        Block::bordered()
            .title(format!(" {} ", title.into()))
            .border_style(self.focused_style(focus))
    }

    /// Creates block having constant color
    pub fn static_block(&self, title: impl Into<String>) -> Block<'static> {
        Block::bordered()
            .title(format!(" {} ", title.into()))
            .border_style(Style::default().fg(self.theme.border))
    }
}

/// Unit-tests for RenderContext
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_focus_logic_consistency() {
        let theme = ThemeColors::default();
        let active_focus = FocusArea::LeftPanel;

        let is_focused = |current: FocusArea, target: FocusArea| current == target;
        let get_style = |current: FocusArea, target: FocusArea, theme: &ThemeColors| {
            if is_focused(current, target) {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.border)
            }
        };

        assert!(is_focused(active_focus, FocusArea::LeftPanel));
        assert!(!is_focused(active_focus, FocusArea::MainContent));

        let style = get_style(active_focus, FocusArea::LeftPanel, &theme);
        assert_eq!(style.fg, Some(theme.accent));
    }
}
