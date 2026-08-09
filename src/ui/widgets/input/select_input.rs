use crate::{
    theme::ThemePalette,
    ui::{RenderContext, WidgetResponse, widgets::input::Input},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Style, Stylize},
};

/// Select widget made for array of values
#[derive(Clone, Debug)]
pub struct SelectInput<T> {
    pub items: Vec<T>,
    pub selected_index: usize,
    pub title: String,
}

impl<T> SelectInput<T>
where
    T: PartialEq + Clone + 'static,
{
    pub fn from(items: Vec<T>, initial: &T) -> Self {
        let selected_index = items.iter().position(|item| item == initial).unwrap_or(0);
        Self {
            items,
            selected_index,
            title: String::default(),
        }
    }
}

impl<T> Input for SelectInput<T>
where
    T: Clone + std::fmt::Display + PartialEq + 'static,
{
    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Key event handling for cycling through items
    fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse {
        match key {
            KeyCode::Enter => return WidgetResponse::Submit,
            KeyCode::Esc => return WidgetResponse::Cancel,
            KeyCode::Left | KeyCode::Char('h') => {
                if !self.items.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.items.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }

                    return WidgetResponse::Changed;
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                    return WidgetResponse::Changed;
                }
            }
            _ => {}
        }

        WidgetResponse::Continue
    }

    /// Resetting input to the first item
    fn reset(&mut self) {
        self.selected_index = 0;
    }

    /// Select input rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect, focused: bool) {
        use ratatui::widgets::{Block, Paragraph};

        let palette = ctx.palette();
        let (border_style, text_style) = self.on_focused(focused, &palette);

        let adapted_border_style =
            Style::default().fg(ctx.color(border_style.fg.unwrap_or(palette.muted)));
        let adapted_text_style =
            Style::default().fg(ctx.color(text_style.fg.unwrap_or(palette.fg)));

        let input_block = Block::bordered()
            .border_style(adapted_border_style)
            .border_type(ctx.config.ui.border_type.into())
            .style(adapted_text_style)
            .title(self.title.as_str())
            .bg(palette.bg)
            .fg(ctx.color(palette.fg));

        let display_text = self
            .items
            .get(self.selected_index)
            .map(|item| item.to_string())
            .unwrap_or_default();

        let input = Paragraph::new(display_text).block(input_block);
        ctx.render_widget(input, area);
    }

    /// Returns styles if input is being focused
    fn on_focused(&self, focused: bool, palette: &ThemePalette) -> (Style, Style) {
        if focused {
            (
                Style::default().fg(palette.accent),
                Style::default().fg(palette.fg),
            )
        } else {
            (
                Style::default().fg(palette.muted),
                Style::default().fg(palette.muted),
            )
        }
    }
}

/// Unit-tests for select input
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::BuiltinTheme;
    use ratatui::crossterm::event::KeyCode;

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::EnumIter)]
    enum MockSelectEnum {
        #[default]
        First,
        Second,
        Third,
    }

    #[test]
    fn should_initialize_select_input() {
        let items = vec![
            MockSelectEnum::First,
            MockSelectEnum::Second,
            MockSelectEnum::Third,
        ];
        let input = SelectInput::from(items, &MockSelectEnum::Second).title("Choice");

        assert_eq!(input.selected_index, 1);
        assert_eq!(input.title, "Choice");
        assert_eq!(input.items.len(), 3);
    }

    #[test]
    fn should_navigate_properly_through_select_input() {
        let items = vec![
            MockSelectEnum::First,
            MockSelectEnum::Second,
            MockSelectEnum::Third,
        ];
        let mut input = SelectInput::from(items, &MockSelectEnum::First);

        let resp = input.handle_key(&KeyCode::Right);
        assert_eq!(input.selected_index, 1);
        assert_eq!(resp, WidgetResponse::Changed);

        input.handle_key(&KeyCode::Char('l'));
        assert_eq!(input.selected_index, 2);

        input.handle_key(&KeyCode::Char('l'));
        input.handle_key(&KeyCode::Char('h'));
        assert_eq!(input.selected_index, 2);
    }

    #[test]
    fn should_handle_cancel_and_submit_for_select_input() {
        let items = vec![MockSelectEnum::First, MockSelectEnum::Second];
        let mut input = SelectInput::from(items, &MockSelectEnum::First);

        assert_eq!(input.handle_key(&KeyCode::Enter), WidgetResponse::Submit);
        assert_eq!(input.handle_key(&KeyCode::Esc), WidgetResponse::Cancel);
    }

    #[test]
    fn should_handle_reset_select_input() {
        let items = vec![MockSelectEnum::First, MockSelectEnum::Second];
        let mut input = SelectInput::from(items, &MockSelectEnum::Second);
        assert_eq!(input.selected_index, 1);

        input.reset();
        assert_eq!(input.selected_index, 0);
    }

    #[test]
    fn should_return_styles_if_focused_select() {
        let items = vec![MockSelectEnum::First];
        let input = SelectInput::from(items, &MockSelectEnum::First);
        let palette: ThemePalette = BuiltinTheme::GruvboxDark.palette();

        let styles_unfocused = input.on_focused(false, &palette);
        let styles_focused = input.on_focused(true, &palette);

        assert_ne!(styles_unfocused, styles_focused);
    }
}
