use crate::{
    traits::{Input, InteractableEnum},
    ui::WidgetResponse,
    utils::constants::theme::TEXT_PRIMARY,
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style},
};

#[derive(Debug, Default, Clone)]
pub struct EnumInput<T> {
    pub title: String,
    pub selected: T,
    pub border_style: Style,
}

impl<T> EnumInput<T> {
    pub fn from(selected: T) -> Self {
        Self {
            selected,
            title: String::default(),
            border_style: Style::default(),
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

    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        use ratatui::{
            style::Style,
            widgets::{Block, Paragraph},
        };

        let focused_style: Style = if focused {
            Style::default().fg(Color::Green)
        } else {
            self.border_style
        };

        let input_block = Block::bordered()
            .border_style(focused_style)
            .title(self.title.clone())
            .title_style(Style::default().fg(TEXT_PRIMARY));

        let input = Paragraph::new(self.selected.to_string()).block(input_block);

        frame.render_widget(input, area);
    }
}

// Unit-tests for enum input
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;

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
        let initial_val = input.selected.clone();

        input.handle_key(&KeyCode::Char(' '));
        assert_eq!(input.selected, initial_val);
    }
}
