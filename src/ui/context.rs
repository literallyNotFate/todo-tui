use crate::{
    config::{KeyMaps, UIConfig},
    core::{Action, ApplicationMode, FocusArea},
    models::Filter,
    state::UIState,
    theme::{Theme, ThemePalette},
    ui::is_terminal_small,
};
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, StatefulWidget, Widget},
};

/// A render context provider for widgets
pub struct RenderContext<'a, 'b> {
    pub frame: Option<&'a mut Frame<'b>>,
    pub theme: Theme,
    pub config: &'a UIConfig,
    pub keymaps: &'a KeyMaps,
    pub is_dimmed: bool,

    mode: ApplicationMode,
    focus: FocusArea,
    filter: Filter,
    is_small: bool,
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn new(
        frame: &'a mut Frame<'b>,
        ui: &'a UIState,
        keymaps: &'a KeyMaps,
        mode: ApplicationMode,
    ) -> Self {
        let area: Rect = frame.area();

        Self {
            frame: Some(frame),
            theme: ui.theme,
            mode,
            keymaps,
            config: &ui.config,
            focus: *ui.focused,
            filter: *ui.filter,
            is_small: is_terminal_small(area.width, area.height),
            is_dimmed: ui.modal.is_some(),
        }
    }

    /// Mock constructor for tests without frame
    #[cfg(test)]
    pub fn mock(ui: &'a UIState, keymaps: &'a KeyMaps) -> Self {
        Self {
            frame: None,
            theme: ui.theme,
            mode: ApplicationMode::Navigation,
            keymaps,
            config: &ui.config,
            focus: FocusArea::default(),
            filter: Filter::default(),
            is_small: false,
            is_dimmed: false,
        }
    }

    /// Render widget wrapper
    pub fn render_widget<W: Widget>(&mut self, widget: W, area: Rect) {
        if let Some(ref mut f) = self.frame {
            f.render_widget(widget, area);
        }
    }

    /// Render stateful widget wrapper
    pub fn render_stateful_widget<W, S>(&mut self, widget: W, area: Rect, state: &mut S)
    where
        W: StatefulWidget<State = S>,
    {
        if let Some(ref mut f) = self.frame {
            f.render_stateful_widget(widget, area, state);
        }
    }

    /// Render modal overlay (dimmed bg if popup/confirm is open)
    pub fn render_modal_overlay(&mut self) {
        let area: Rect = self.area();
        let is_dark: bool = self.theme.is_dark();

        if let Some(ref mut f) = self.frame {
            let overlay_color: Color = if is_dark {
                Color::Rgb(10, 10, 10)
            } else {
                Color::Rgb(200, 200, 200)
            };

            let overlay_style = Style::default().bg(overlay_color).dim();
            f.render_widget(Block::default().style(overlay_style), area);
        }
    }

    /// Sets cursor position wrapper for input
    pub fn set_cursor_position(&mut self, position: Position) {
        if let Some(ref mut f) = self.frame {
            f.set_cursor_position(position);
        }
    }

    /// Returns an area rect of a current frame
    pub fn area(&self) -> Rect {
        match &self.frame {
            Some(f) => f.area(),
            None => Rect::new(0, 0, 80, 24),
        }
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

    /// Returns focus value
    pub fn focus(&self) -> FocusArea {
        self.focus
    }

    /// Returns current filter name for todos
    pub fn filter(&self) -> String {
        self.filter.to_string()
    }

    /// Helper: is that widget being foucused on?
    pub fn is_focused(&self, area: FocusArea) -> bool {
        self.focus == area
    }

    /// Helper: what color to render on focused (based on FocusArea)
    pub fn focused_color(&self, on: Color, target: FocusArea) -> Color {
        let palette: ThemePalette = self.palette();
        if self.is_focused(target) {
            self.color(on)
        } else {
            palette.muted
        }
    }

    /// Struct builder method with dimmed flag on/off
    pub fn with_dimmed(mut self, dimmed: bool) -> Self {
        self.is_dimmed = dimmed;
        self
    }

    /// Dimmed flag setter
    pub fn set_dimmed(&mut self, dimmed: bool) {
        self.is_dimmed = dimmed;
    }

    /// Return color from palette considering overlay
    pub fn color(&self, normal: Color) -> Color {
        if self.is_dimmed {
            self.palette().muted
        } else {
            normal
        }
    }

    /// Creates basic block with border focusing
    pub fn block(&self, title: impl Into<String>, focus: Option<FocusArea>) -> Block<'static> {
        let palette = self.palette();
        let border_color = match focus {
            Some(area) if self.is_focused(area) => self.color(palette.accent),
            _ => palette.muted,
        };

        Block::bordered()
            .title(format!(" {} ", title.into()))
            .border_type(self.config.border_type.into())
            .border_style(Style::default().fg(border_color))
    }

    /// Wrapper to get first assigned key for dynamic hotkeys
    pub fn get_key(&self, action: Action) -> String {
        self.keymaps.first_assigned(action)
    }

    /// Helper to form dynamic hotkey text with action: <key>:label
    pub fn key_hint(&self, action: Action, label: &str, color: Color) -> Vec<Span<'static>> {
        let key: String = self.get_key(action);
        let palette: ThemePalette = self.palette();

        if key.is_empty() {
            return vec![];
        }

        vec![
            Span::styled(
                format!(" {}", key),
                Style::default().fg(self.color(color)).bold(),
            ),
            Span::styled(format!(":{} ", label), Style::default().fg(palette.muted)),
        ]
    }

    /// Truncates text if too long (for notifications/summary)
    pub fn truncate(text: &str, max_width: usize) -> String {
        let char_count = text.chars().count();
        if char_count > max_width && max_width > 0 {
            let truncated: String = text.chars().take(max_width.saturating_sub(1)).collect();
            format!("{}…", truncated)
        } else {
            text.to_string()
        }
    }

    /// Truncates text based on area width
    pub fn truncate_to_area(&self, text: &str) -> String {
        Self::truncate(text, self.area().width as usize)
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
        let active_focus = FocusArea::Sidebar;

        let is_focused = |current: FocusArea, target: FocusArea| current == target;
        let get_style = |current: FocusArea, target: FocusArea, palette: &ThemePalette| {
            if is_focused(current, target) {
                Style::default().fg(palette.accent)
            } else {
                Style::default().fg(palette.muted)
            }
        };

        assert!(is_focused(active_focus, FocusArea::Sidebar));
        assert!(!is_focused(active_focus, FocusArea::Main));

        let style = get_style(active_focus, FocusArea::Sidebar, &palette);
        assert_eq!(style.fg, Some(palette.accent));
    }

    #[test]
    fn should_truncate_text_shorter_than_limit() {
        let text = "Hello";
        assert_eq!(RenderContext::truncate(text, 10), "Hello");
    }

    #[test]
    fn should_truncate_text_longer_than_limit() {
        let text = "Long task description";
        let result = RenderContext::truncate(text, 10);

        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('…'));
        assert_eq!(result, "Long task…");
    }

    #[test]
    fn should_truncate_text_exact_limit() {
        let text = "Exact";
        assert_eq!(RenderContext::truncate(text, 5), "Exact");
    }

    #[test]
    fn should_truncate_text_unicode_support() {
        let text = "🦀🦀🦀🦀🦀";
        let result = RenderContext::truncate(text, 3);

        assert_eq!(result, "🦀🦀…");
        assert_eq!(result.chars().count(), 3);
    }

    #[test]
    fn should_handle_zero_width() {
        let text = "Anything";
        assert_eq!(RenderContext::truncate(text, 0), "");
    }

    #[test]
    fn should_handle_single_char_width() {
        let text = "Long";
        assert_eq!(RenderContext::truncate(text, 1), "…");
    }
}
