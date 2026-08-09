use crate::{
    theme::ThemePalette,
    ui::{Field, FieldType, RenderContext, WidgetResponse, widgets::input::Input},
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::{Style, Stylize},
    text::Line,
};

/// Main form interface
#[derive(Clone, Debug)]
pub struct Form {
    pub fields: Vec<Field>,
    pub focused: usize,
}

impl Form {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields, focused: 0 }
    }

    /// Form rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::{prelude::Margin, style::Style, widgets::Block};

        let palette = ctx.palette();
        ctx.render_widget(
            Block::default()
                .title_bottom(self.hotkeys(&palette))
                .style(Style::default().bg(palette.bg)),
            area,
        );

        let inner_area = area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });
        let chunks = self.layout(inner_area);

        for (i, field) in self.fields.iter().enumerate() {
            if i < chunks.len() {
                field.field_type.render(ctx, chunks[i], self.focused == i);
            }
        }
    }

    /// Key event handling
    pub fn handle_key(&mut self, event: &KeyEvent) -> WidgetResponse {
        let key: KeyCode = event.code;
        let modifiers: KeyModifiers = event.modifiers;

        match (key, modifiers) {
            (KeyCode::Enter, KeyModifiers::ALT) => return WidgetResponse::Submit,
            (KeyCode::Enter, KeyModifiers::NONE) if self.is_button_selected() => {
                return WidgetResponse::Submit;
            }
            (KeyCode::Esc, KeyModifiers::NONE) => return WidgetResponse::Cancel,
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.next_focus();
                return WidgetResponse::Continue;
            }
            (KeyCode::BackTab, KeyModifiers::NONE) => {
                self.prev_focus();
                return WidgetResponse::Continue;
            }
            (KeyCode::Down, KeyModifiers::NONE) => {
                if !self.is_textarea_focused() || self.is_cursor_at_bottom() {
                    self.next_focus();
                    return WidgetResponse::Continue;
                }
            }
            (KeyCode::Up, KeyModifiers::NONE) => {
                if !self.is_textarea_focused() || self.is_cursor_at_top() {
                    self.prev_focus();
                    return WidgetResponse::Continue;
                }
            }
            _ => {}
        }

        if let Some(field) = self.fields.get_mut(self.focused) {
            match &mut field.field_type {
                FieldType::Multiline { input } => {
                    input.input(*event);
                }
                FieldType::Text { input } => {
                    input.handle_key(&key);
                }
                FieldType::PriorityEnum { input } => {
                    input.handle_key(&key);
                }
                FieldType::ColorEnum { input } => {
                    input.handle_key(&key);
                }
                FieldType::Select { input } => {
                    input.handle_key(&key);
                }
                _ => {}
            }
        }

        WidgetResponse::Continue
    }

    /// Get value from the text field by its name
    pub fn get_text_value(&self, name: &str) -> String {
        self.fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| match &f.field_type {
                FieldType::Text { input } => input.buffer.clone(),
                FieldType::Multiline { input } => input.lines().join("\n"),
                _ => String::new(),
            })
            .unwrap_or_default()
    }

    /// Focus on the next field
    pub fn next_focus(&mut self) {
        if !self.fields.is_empty() {
            self.focused = (self.focused + 1) % self.fields.len();
        }
    }

    /// Focus on the prev field
    pub fn prev_focus(&mut self) {
        if !self.fields.is_empty() {
            self.focused = (self.focused + self.fields.len() - 1) % self.fields.len();
        }
    }

    /// Checks whether button is selected
    pub fn is_button_selected(&self) -> bool {
        matches!(
            self.fields.get(self.focused).map(|f| &f.field_type),
            Some(FieldType::Button)
        )
    }

    /// Checks whether textarea is selected
    fn is_textarea_focused(&self) -> bool {
        if let Some(f) = self.fields.get(self.focused) {
            matches!(f.field_type, FieldType::Multiline { .. })
        } else {
            false
        }
    }

    /// Checks whether enum input/select input is selected
    fn is_enum_or_select_focused(&self) -> bool {
        matches!(
            self.fields.get(self.focused).map(|f| &f.field_type),
            Some(FieldType::PriorityEnum { .. })
                | Some(FieldType::ColorEnum { .. })
                | Some(FieldType::Select { .. })
        )
    }

    /// Check if cursor of textarea is on the top (to focus to prev field)
    fn is_cursor_at_top(&self) -> bool {
        if let Some(field) = self.fields.get(self.focused) {
            if let FieldType::Multiline { input } = &field.field_type {
                let (row, _) = input.cursor();
                return row == 0;
            }
        }

        false
    }

    /// Check if cursor of textarea is on the bottom (to focus to next field)
    fn is_cursor_at_bottom(&self) -> bool {
        if let Some(field) = self.fields.get(self.focused) {
            if let FieldType::Multiline { input } = &field.field_type {
                let (row, _) = input.cursor();
                return row == input.lines().len() - 1;
            }
        }

        false
    }

    /// Checks whether the form contains a submit button field
    pub fn has_button(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f.field_type, FieldType::Button))
    }

    /// Generates hotkeys for form
    fn hotkeys(&self, palette: &ThemePalette) -> Line<'static> {
        use ratatui::text::{Line, Span};

        let mut spans = Vec::new();
        if self.has_button() {
            spans.extend(vec![
                Span::styled("<alt+enter>", Style::default().fg(palette.success).bold()),
                Span::styled(":submit ", Style::default().fg(palette.muted)),
            ]);
        }

        if self.is_enum_or_select_focused() {
            spans.push(Span::styled(
                " ◄/►",
                Style::default().fg(palette.accent).bold(),
            ));
            spans.push(Span::styled(":select ", Style::default().fg(palette.muted)));
        }

        if self.fields.len() > 1 {
            spans.extend(vec![
                Span::styled(" ▲/▼", Style::default().fg(palette.secondary).bold()),
                Span::styled(":move ", Style::default().fg(palette.muted)),
            ]);
        }

        Line::from(spans).left_aligned()
    }

    /// Main form layout method
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        use ratatui::layout::{Constraint, Direction, Layout};
        let constraints: Vec<Constraint> = self
            .fields
            .iter()
            .map(|f| match &f.field_type {
                FieldType::Text { .. }
                | FieldType::PriorityEnum { .. }
                | FieldType::ColorEnum { .. }
                | FieldType::Select { .. }
                | FieldType::Button => Constraint::Length(3),
                FieldType::Multiline { .. } => Constraint::Min(6),
            })
            .collect();
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area)
    }

    /// Helper function to initialize field values using field key/name (primarily for tests)
    #[cfg(test)]
    pub fn set_value(&mut self, key: &str, value: &str) {
        use crate::models::{FolderColor, Priority};
        use std::str::FromStr;
        use tui_textarea::TextArea;

        if let Some(field) = self.fields.iter_mut().find(|f| f.name == key) {
            match &mut field.field_type {
                FieldType::Text { input } => {
                    input.buffer = value.to_string();
                }
                FieldType::Multiline { input } => {
                    let lines: Vec<String> = value.lines().map(|s| s.to_string()).collect();
                    *input = TextArea::new(lines);
                }
                FieldType::PriorityEnum { input } => {
                    if let Ok(priority) = Priority::from_str(value) {
                        input.selected.value = priority;
                    }
                }
                FieldType::ColorEnum { input } => {
                    if let Ok(color) = FolderColor::from_str(value) {
                        input.selected.value = color;
                    }
                }
                FieldType::Select { input } => {
                    use crate::theme::ThemeId;

                    if let Ok(theme_id) = ThemeId::from_str(value) {
                        if let Some(pos) = input.items.iter().position(|item| item == &theme_id) {
                            input.selected_index = pos;
                        }
                    }
                }
                FieldType::Button => {}
            }
        }
    }
}

/// Unit-tests for generic form widget and its contexts
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use ratatui::crossterm::event::{KeyEventKind, KeyEventState};

    fn create_test_form() -> Form {
        Form::new(vec![
            Field::new("title", FieldType::text("Title", "")),
            Field::new("priority", FieldType::priority(Priority::Low)),
            Field::new("description", FieldType::textarea("")),
            Field::new("save", FieldType::Button),
        ])
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn should_create_generic_form_with_fields() {
        let form = create_test_form();
        assert_eq!(form.focused, 0);
        assert_eq!(form.fields.len(), 4);

        assert_eq!(form.fields[0].name, "title");
        assert_eq!(form.fields[1].name, "priority");
        assert_eq!(form.fields[2].name, "description");
        assert!(matches!(form.fields[3].field_type, FieldType::Button));
    }

    #[test]
    fn should_navigate_through_fields_using_down_key() {
        let mut form = create_test_form();
        assert_eq!(form.focused, 0);

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 1);

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 2);

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 3);

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 0);
    }

    #[test]
    fn should_handle_text_and_textarea_inputs() {
        let mut form = create_test_form();

        form.handle_key(&key(KeyCode::Char('R')));
        form.handle_key(&key(KeyCode::Char('u')));
        form.handle_key(&key(KeyCode::Char('s')));
        form.handle_key(&key(KeyCode::Char('t')));
        assert_eq!(form.get_text_value("title"), "Rust");

        form.focused = 2;
        form.handle_key(&key(KeyCode::Char('T')));
        form.handle_key(&key(KeyCode::Char('U')));
        form.handle_key(&key(KeyCode::Char('I')));
        assert_eq!(form.get_text_value("description"), "TUI");
    }

    #[test]
    fn should_return_submit_on_button_enter() {
        let mut form = create_test_form();
        form.focused = 3;

        let response = form.handle_key(&key(KeyCode::Enter));
        assert!(matches!(response, WidgetResponse::Submit));
    }

    #[test]
    fn should_prevent_navigation_if_textarea_cursor_not_at_edge() {
        let mut form = create_test_form();
        form.focused = 2;

        if let FieldType::Multiline { input } = &mut form.fields[2].field_type {
            input.insert_str("Line 1");
            input.insert_newline();
            input.insert_str("Line 2");
            input.move_cursor(tui_textarea::CursorMove::Up);
        }

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 2);

        if let FieldType::Multiline { input } = &mut form.fields[2].field_type {
            input.move_cursor(tui_textarea::CursorMove::Bottom);
        }

        form.handle_key(&key(KeyCode::Down));
        assert_eq!(form.focused, 3);
    }

    #[test]
    fn should_properly_set_text_values_via_helper() {
        let mut form = create_test_form();

        form.set_value("title", "New Title");
        form.set_value("description", "Line 1\nLine 2");

        assert_eq!(form.get_text_value("title"), "New Title");
        assert_eq!(form.get_text_value("description"), "Line 1\nLine 2");
    }
}
