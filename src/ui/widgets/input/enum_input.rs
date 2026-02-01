use crate::{
    theme::ThemeColors,
    traits::{Input, InteractableEnum},
    ui::WidgetResponse,
};
use ratatui::{Frame, crossterm::event::KeyCode, layout::Rect, style::Style};

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
    T: InteractableEnum + Clone + 'static,
{
    fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

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

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &ThemeColors) {
        use ratatui::widgets::{Block, Paragraph};

        let focused_style: Style = self.on_focused(focused, theme);

        let input_block: Block = Block::bordered()
            .border_style(focused_style)
            .title(self.title.as_str())
            .title_style(Style::default().fg(theme.text_primary));

        let input = Paragraph::new(self.selected.to_string()).block(input_block);

        frame.render_widget(input, area);
    }

    fn on_focused(&self, focused: bool, theme: &ThemeColors) -> Style {
        if focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(theme.border)
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
        let mut style: Style = input.on_focused(false, &ThemeColors::GRUVBOX);
        assert_eq!(style, Style::default().fg(Color::Rgb(102, 92, 84)));

        style = input.on_focused(true, &ThemeColors::GRUVBOX);
        assert_eq!(style, Style::default().fg(Color::Rgb(250, 189, 47)));
    }
}
