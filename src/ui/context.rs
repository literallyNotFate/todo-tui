use crate::{
    config::UIConfig,
    core::ApplicationMode,
    enums::FocusArea,
    models::Filter,
    state::UIState,
    theme::{Theme, ThemePalette},
    traits::InteractableEnum,
    ui::is_terminal_small,
};
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

/// A render context provider for widgets
pub struct RenderContext<'a, 'b> {
    pub frame: &'a mut Frame<'b>,
    pub theme: Theme,
    pub config: UIConfig,

    mode: ApplicationMode,
    focus: FocusArea,
    filter: Filter,
    is_small: bool,
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn new(frame: &'a mut Frame<'b>, ui: &UIState, mode: ApplicationMode) -> Self {
        let area: Rect = frame.area();

        Self {
            frame,
            theme: ui.theme,
            mode,
            config: ui.config.clone(),
            focus: ui.focus_area,
            filter: ui.current_filter,
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

    /// Render modal overlay (dimmed bg if popup/confirm is open)
    pub fn render_modal_overlay(&mut self) {
        let overlay_color: Color = if self.theme.is_dark() {
            Color::Rgb(10, 10, 10)
        } else {
            Color::Rgb(200, 200, 200)
        };

        let overlay_style: Style = Style::default().bg(overlay_color).dim();
        self.frame
            .render_widget(Block::default().style(overlay_style), self.area());
    }

    /// Returns an area rect of a current frame
    pub fn area(&self) -> Rect {
        self.frame.area()
    }

    /// Returns color palette for current theme
    pub fn palette(&self) -> ThemePalette {
        self.theme.palette()
    }

    /// Returns is_small value
    pub fn is_small(&self) -> bool {
        self.is_small
    }

    /// Returns mode value
    pub fn mode(&self) -> ApplicationMode {
        self.mode
    }

    /// Returns current filter name for todos
    pub fn filter(&self) -> &'static str {
        self.filter.to_string()
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

    /// Helper: what color to render on focused (based on FocusArea)
    pub fn focused_color(&self, on: Color, target: FocusArea) -> Color {
        let palette: ThemePalette = self.palette();
        if self.is_focused(target) {
            on
        } else {
            palette.muted
        }
    }

    /// Creates basic block with border focusing
    pub fn block(&self, title: impl Into<String>, focus: FocusArea) -> Block<'static> {
        let palette: ThemePalette = self.palette();
        Block::bordered()
            .title(format!(" {} ", title.into()))
            .border_type(self.config.border_type.into())
            .border_style(Style::default().fg(self.focused_color(palette.accent, focus)))
    }

    /// Creates block having constant color
    pub fn static_block(&self, title: impl Into<String>) -> Block<'static> {
        let palette: ThemePalette = self.palette();
        Block::bordered()
            .title(format!(" {} ", title.into()))
            .border_type(self.config.border_type.into())
            .border_style(Style::default().fg(palette.muted))
    }
}

/// Unit-tests for RenderContext
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_focus_logic_consistency() {
        let theme = Theme::default();
        let palette = theme.palette();
        let active_focus = FocusArea::LeftPanel;

        let is_focused = |current: FocusArea, target: FocusArea| current == target;
        let get_style = |current: FocusArea, target: FocusArea, palette: &ThemePalette| {
            if is_focused(current, target) {
                Style::default().fg(palette.accent)
            } else {
                Style::default().fg(palette.muted)
            }
        };

        assert!(is_focused(active_focus, FocusArea::LeftPanel));
        assert!(!is_focused(active_focus, FocusArea::MainContent));

        let style = get_style(active_focus, FocusArea::LeftPanel, &palette);
        assert_eq!(style.fg, Some(palette.accent));
    }
}
