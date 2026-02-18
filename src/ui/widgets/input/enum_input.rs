use crate::{
    enums::WidgetResponse,
    theme::ThemeColors,
    traits::{Input, InteractableEnum},
    ui::RenderContext,
};
use ratatui::{crossterm::event::KeyCode, layout::Rect, style::Style};

/// Input widget made for enum types
#[derive(Debug, Default, Clone)]
pub struct EnumInput<T> {
    pub title: String,
    pub selected: T,
}

impl<T> EnumInput<T> {
    pub fn from(selected: T) -> Self {
        Self {
            selected,
            title: String::default(),
        }
    }
}

impl<T> Input for EnumInput<T>
where
    T: InteractableEnum + Clone + Default + 'static,
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
            KeyCode::Left | KeyCode::Char('h') => self.selected = self.selected.prev(),
            KeyCode::Right | KeyCode::Char('l') => self.selected = self.selected.next(),
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

        let theme = &ctx.theme;
        let (border_style, text_style): (Style, Style) = self.on_focused(focused, theme);

        let input_block: Block = Block::bordered()
            .border_style(border_style)
            .style(text_style)
            .title(self.title.as_str());

        let input = Paragraph::new(self.selected.to_string()).block(input_block);

        ctx.render_widget(input, area);
    }

    /// Returns styles if input is being focused
    fn on_focused(&self, focused: bool, theme: &ThemeColors) -> (Style, Style) {
        if focused {
            (
                Style::default().fg(theme.accent),
                Style::default().fg(theme.text_primary),
            )
        } else {
            (
                Style::default().fg(theme.border),
                Style::default().fg(theme.text_dim),
            )
        }
    }
}

// Unit-tests for enum input
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{crossterm::event::KeyCode, style::Color};

    #[derive(Debug, Clone, PartialEq, Default, Copy)]
    enum MockEnum {
        #[default]
        A,
        B,
    }

    impl InteractableEnum for MockEnum {
        fn all_variants() -> &'static [Self] {
            &[Self::A, Self::B]
        }

        fn to_string(&self) -> &'static str {
            match self {
                Self::A => "A",
                Self::B => "B",
            }
        }

        fn next(&self) -> Self {
            match self {
                MockEnum::A => MockEnum::B,
                MockEnum::B => MockEnum::A,
            }
        }
        fn prev(&self) -> Self {
            match self {
                MockEnum::A => MockEnum::B,
                MockEnum::B => MockEnum::A,
            }
        }
    }

    #[test]
    fn should_initialize_enum_input() {
        let input = EnumInput::from(MockEnum::B).title("Status");
        assert_eq!(input.selected, MockEnum::B);
        assert_eq!(input.title, "Status");
    }

    #[test]
    fn should_navigate_properly_through_enum_input() {
        let mut input = EnumInput::from(MockEnum::A);

        let resp = input.handle_key(&KeyCode::Right);
        assert_eq!(input.selected, MockEnum::B);
        assert_eq!(resp, WidgetResponse::Continue);

        input.handle_key(&KeyCode::Char('h'));
        assert_eq!(input.selected, MockEnum::A);
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
        let mut styles: (Style, Style) = input.on_focused(false, &ThemeColors::GRUVBOX);
        assert_eq!(
            styles,
            (
                Style::default().fg(Color::Rgb(102, 92, 84)),
                Style::default().fg(Color::Rgb(168, 153, 132))
            )
        );

        styles = input.on_focused(true, &ThemeColors::GRUVBOX);
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
        assert_eq!(input.selected, MockEnum::B);

        input.reset();
        assert_ne!(initial_val, input.selected);
    }
}
