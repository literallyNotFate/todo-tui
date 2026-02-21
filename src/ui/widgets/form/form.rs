use crate::{
    enums::WidgetResponse,
    models::{Priority, Todo},
    traits::Input,
    ui::{Field, FieldType, RenderContext},
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Paragraph},
};
use tui_textarea::TextArea;
use uuid::Uuid;

/// Form for appending/updating task
#[derive(Clone, Debug)]
pub struct Form {
    pub task_id: Option<Uuid>,
    pub focused: usize,
    pub fields: Vec<Field>,
}

impl Form {
    /// Creates new form (for append)
    pub fn new() -> Self {
        Self {
            fields: vec![
                Field::new("title", FieldType::text(" Title ", "")),
                Field::new("priority", FieldType::priority(Priority::Low)),
                Field::new("description", FieldType::textarea("")),
                Field::new("button", FieldType::Button),
            ],
            focused: 0,
            task_id: None,
        }
    }

    /// Creates new form (for update) with id
    pub fn from(task: &Todo) -> Self {
        Self {
            fields: vec![
                Field::new("title", FieldType::text(" Title ", &task.title)),
                Field::new("priority", FieldType::priority(task.priority)),
                Field::new("description", FieldType::textarea(&task.description)),
                Field::new("button", FieldType::Button),
            ],
            focused: 0,
            task_id: Some(task.id),
        }
    }

    /// Returs all values from form inputs to append/update
    pub fn data(&self) -> (Option<Uuid>, String, String, Priority) {
        let mut title: String = String::new();
        let mut description: String = String::new();
        let mut priority: Priority = Priority::Low;

        for field in &self.fields {
            match &field.field_type {
                FieldType::Text { input } => title = input.buffer.clone(),
                FieldType::Multiline { input } => description = input.lines().join("\n"),
                FieldType::Enum { input } => priority = input.selected,
                FieldType::Button => continue,
            }
        }

        (self.task_id, title, description, priority)
    }

    /// Key event handling
    pub fn handle_key(&mut self, event: &KeyEvent) -> WidgetResponse {
        let key: KeyCode = event.code;
        let modifiers: KeyModifiers = event.modifiers;

        match (key, modifiers) {
            (KeyCode::Enter, KeyModifiers::ALT) => {
                return WidgetResponse::Submit;
            }
            (KeyCode::Enter, KeyModifiers::NONE) if self.is_button_selected() => {
                return WidgetResponse::Submit;
            }
            (KeyCode::Esc, KeyModifiers::NONE) => return WidgetResponse::Cancel,
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
                FieldType::Enum { input } => {
                    input.handle_key(&key);
                }
                _ => {}
            }
        }

        WidgetResponse::Continue
    }

    /// Helper function to initialize field values (textbased only) using field key/name (for tests)
    pub fn set_value(&mut self, key: &str, value: &str) {
        if let Some(field) = self.fields.iter_mut().find(|f| f.name == key) {
            match &mut field.field_type {
                FieldType::Text { input } => {
                    input.buffer = value.to_string();
                }
                FieldType::Multiline { input } => {
                    let lines: Vec<String> = value.lines().map(|s| s.to_string()).collect();
                    *input = TextArea::new(lines);
                }
                _ => {}
            }
        }
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

    /// Main form layout method
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Priority
                Constraint::Min(10),   // Description
                Constraint::Length(3),
                Constraint::Length(3), // Button
            ])
            .split(area)
    }

    /// Form buttons layout method
    fn button_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(15), // Button 1
                Constraint::Min(0),
                Constraint::Length(15), // Button 2
            ])
            .split(area)
    }

    /// Form rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        let chunks: std::rc::Rc<[Rect]> = self.layout(area);
        let button_layout: std::rc::Rc<[Rect]> = self.button_layout(chunks[4]);
        let palette = ctx.palette();

        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = self.focused == i;
            let focused_style: Style;

            match &field.field_type {
                FieldType::Multiline { input } => {
                    let mut input = input.clone();

                    if is_focused {
                        input.set_cursor_style(
                            Style::default().bg(palette.accent).fg(palette.selection),
                        );
                        focused_style = Style::default().fg(palette.accent);
                    } else {
                        input.set_cursor_style(Style::default());
                        focused_style = Style::default().fg(palette.muted);
                    }

                    let block = Block::bordered()
                        .title(" Description ")
                        .border_style(focused_style);

                    input.set_block(block);
                    input.set_style(Style::default().fg(palette.fg));

                    ctx.render_widget(&input, chunks[i]);
                }
                FieldType::Text { input } => {
                    input.render(ctx, chunks[i], is_focused);
                }
                FieldType::Enum { input } => {
                    input.render(ctx, chunks[i], is_focused);
                }
                FieldType::Button => {
                    let (border_style, text_style) = if is_focused {
                        (
                            Style::default().fg(palette.accent),
                            Style::default().fg(palette.fg),
                        )
                    } else {
                        (
                            Style::default().fg(palette.muted),
                            Style::default().fg(palette.muted),
                        )
                    };

                    let button = Paragraph::new(" Save task ")
                        .block(
                            Block::bordered()
                                .border_style(border_style)
                                .style(text_style),
                        )
                        .centered();

                    ctx.render_widget(button, button_layout[2]);
                }
            }
        }
    }
}

/// Unit-tests for form
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn should_create_new_form() {
        let form = Form::new();
        assert_eq!(form.focused, 0);
        assert!(form.task_id.is_none());
        assert_eq!(form.fields.len(), 4);

        assert_eq!(form.fields[0].name, "title");
        assert_eq!(form.fields[1].name, "priority");
        assert_eq!(form.fields[2].name, "description");
        assert!(matches!(form.fields[3].field_type, FieldType::Button));
    }

    #[test]
    fn should_create_form_with_task() {
        let task = Todo::new("Buy milk", "At the store", Some(Priority::High));
        let form = Form::from(&task);

        assert_eq!(form.task_id, Some(task.id));

        if let FieldType::Text { input } = &form.fields[0].field_type {
            assert_eq!(input.buffer, "Buy milk");
        }
        if let FieldType::Enum { input } = &form.fields[1].field_type {
            assert_eq!(input.selected, Priority::High);
        }
        if let FieldType::Multiline { input } = &form.fields[2].field_type {
            assert_eq!(input.lines().join("\n"), "At the store");
        }
    }

    #[test]
    fn should_navigate_through_form_with_button() {
        let mut form = Form::new();
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
    fn should_handle_multiline_input() {
        let mut form = Form::new();
        form.focused = 2;

        form.handle_key(&key(KeyCode::Char('L')));
        form.handle_key(&key(KeyCode::Char('i')));
        form.handle_key(&key(KeyCode::Char('n')));
        form.handle_key(&key(KeyCode::Char('e')));

        if let FieldType::Multiline { input } = &form.fields[2].field_type {
            assert_eq!(input.lines()[0], "Line");
        }
    }

    #[test]
    fn should_return_submit_on_button_enter() {
        let mut form = Form::new();
        form.focused = 3;

        let response = form.handle_key(&key(KeyCode::Enter));
        assert!(matches!(response, WidgetResponse::Submit));
    }

    #[test]
    fn should_prevent_navigation_if_textarea_not_at_edge() {
        let mut form = Form::new();
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
    fn should_handle_key_for_form() {
        let mut form = Form::new();

        form.handle_key(&key(KeyCode::Char('R')));
        form.handle_key(&key(KeyCode::Char('u')));
        form.handle_key(&key(KeyCode::Char('s')));
        form.handle_key(&key(KeyCode::Char('t')));

        if let FieldType::Text { input } = &form.fields[0].field_type {
            assert_eq!(input.buffer, "Rust");
        }

        form.next_focus();
        form.handle_key(&key(KeyCode::Right));

        if let FieldType::Enum { input } = &form.fields[1].field_type {
            assert_ne!(input.selected, Priority::Low);
        }
    }

    #[test]
    fn should_return_all_data_on_append() {
        let mut form = Form::new();

        if let FieldType::Text { input } = &mut form.fields[0].field_type {
            input.buffer = "Task".to_string();
        }

        if let FieldType::Multiline { input } = &mut form.fields[2].field_type {
            input.insert_str("Multiline content");
        }

        let (_, title, desc, priority) = form.data();
        assert_eq!(title, "Task");
        assert_eq!(desc, "Multiline content");
        assert_eq!(priority, Priority::Low);
    }

    #[test]
    fn should_return_all_data_on_update() {
        let task = Todo::new("Old Title", "", None);
        let task_id: Uuid = task.id;
        let mut form = Form::from(&task);

        if let FieldType::Text { input } = &mut form.fields[0].field_type {
            input.buffer = "Updated Title".to_string();
        }

        let (id, title, desc, priority) = form.data();
        assert_eq!(id, Some(task_id));
        assert_eq!(title, "Updated Title");
        assert_eq!(desc, "");
        assert_eq!(priority, Priority::Low);
    }

    #[test]
    fn should_properly_set_value_for_text_input() {
        let mut form = Form::new();

        form.set_value("title", "Title test");
        form.set_value("description", "Desc test");

        let (_, title, desc, _) = form.data();
        assert_eq!(title, "Title test");
        assert_eq!(desc, "Desc test");
    }
}
