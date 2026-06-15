use crate::{
    core::Selectable,
    theme::ThemePalette,
    ui::{RenderContext, WidgetResponse, widgets::input::Input},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Style, Stylize},
};

/// Input widget made for enum types
#[derive(Debug, Default, Clone)]
pub struct EnumInput<T> {
    pub title: String,
    pub selected: Selectable<T>,
}

impl<T> EnumInput<T>
where
    T: strum::IntoEnumIterator + Copy + PartialEq + Default + 'static,
{
    pub fn from(value: T) -> Self {
        Self {
            selected: Selectable::new(value),
            title: String::default(),
        }
    }
}

impl<T> Input for EnumInput<T>
where
    T: strum::IntoEnumIterator + Copy + PartialEq + Default + std::fmt::Display + 'static,
{
    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Key event handling
    fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse {
        match key {
            KeyCode::Enter => return WidgetResponse::Submit,
            KeyCode::Esc => return WidgetResponse::Cancel,
            KeyCode::Left | KeyCode::Char('h') => self.selected.prev(),
            KeyCode::Right | KeyCode::Char('l') => self.selected.next(),
            _ => {}
        }

        WidgetResponse::Continue
    }

    /// Resetting input
    fn reset(&mut self) {
        self.selected = Default::default();
    }

    /// Enum input rendering
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

        let input = Paragraph::new(self.selected.to_string()).block(input_block);
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

/// Unit-tests for enum input
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;
    use ratatui::{crossterm::event::KeyCode, style::Color};

    #[derive(Debug, Clone, PartialEq, Default, Copy, strum::Display, strum::EnumIter)]
    enum MockEnum {
        #[default]
        A,
        B,
    }

    #[test]
    fn should_initialize_enum_input() {
        let input = EnumInput::from(MockEnum::B).title("Status");
        assert_eq!(input.selected.value, MockEnum::B);
        assert_eq!(input.title, "Status");
    }

    #[test]
    fn should_navigate_properly_through_enum_input() {
        let mut input = EnumInput::from(MockEnum::A);

        let resp = input.handle_key(&KeyCode::Right);
        assert_eq!(input.selected.value, MockEnum::B);
        assert_eq!(resp, WidgetResponse::Continue);

        input.handle_key(&KeyCode::Char('h'));
        assert_eq!(input.selected.value, MockEnum::A);
    }

    #[test]
    fn should_handle_cancel_key_for_enum_input() {
        let mut input = EnumInput::from(MockEnum::A);

        assert_eq!(input.handle_key(&KeyCode::Enter), WidgetResponse::Submit);
        assert_eq!(input.handle_key(&KeyCode::Esc), WidgetResponse::Cancel);
    }

    #[test]
    fn should_not_handle_other_keys_for_enum_input() {
        let mut input = EnumInput::from(MockEnum::A);
        let initial_val = input.selected;

        input.handle_key(&KeyCode::Char(' '));
        assert_eq!(input.selected, initial_val);
    }

    #[test]
    fn should_return_styles_if_focused() {
        let input = EnumInput::from(MockEnum::B);
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut styles: (Style, Style) = input.on_focused(false, &palette);
        assert_eq!(
            styles,
            (
                Style::default().fg(Color::Rgb(146, 131, 116)),
                Style::default().fg(Color::Rgb(146, 131, 116))
            )
        );

        styles = input.on_focused(true, &palette);
        assert_eq!(
            styles,
            (
                Style::default().fg(Color::Rgb(250, 189, 47)),
                Style::default().fg(Color::Rgb(235, 219, 178))
            )
        );
    }

    #[test]
    fn should_handle_reset_enum_input() {
        let mut input = EnumInput::from(MockEnum::B);
        let initial_val = input.selected;
        assert_eq!(input.selected.value, MockEnum::B);

        input.reset();
        assert_ne!(initial_val, input.selected);
    }
}
