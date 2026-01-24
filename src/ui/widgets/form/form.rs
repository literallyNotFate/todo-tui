use crate::{
    models::{Priority, Todo},
    state::{ApplicationResult, ApplicationState},
    traits::Input,
    ui::{Field, FieldType, WidgetResponse},
};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

#[derive(Clone, Debug)]
pub struct Form<'a> {
    pub task_id: Option<uuid::Uuid>,
    pub focused: usize,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Form<'a> {
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

    // Focus on the next field
    pub fn next_focus(&mut self) {
        if !self.fields.is_empty() {
            self.focused = (self.focused + 1) % self.fields.len();
        }
    }

    // Focus on the prev field
    pub fn prev_focus(&mut self) {
        if !self.fields.is_empty() {
            self.focused = (self.focused + self.fields.len() - 1) % self.fields.len();
        }
    }

    // Checks whether button is selected
    pub fn is_button_selected(&self) -> bool {
        matches!(
            self.fields.get(self.focused).map(|f| &f.field_type),
            Some(FieldType::Button)
        )
    }

    // Checks whether textarea is selected
    fn is_textarea_focused(&self) -> bool {
        if let Some(f) = self.fields.get(self.focused) {
            matches!(f.field_type, FieldType::Multiline { .. })
        } else {
            false
        }
    }

    // Check if cursor of textarea is on the top (to focus to prev field)
    fn is_cursor_at_top(&self) -> bool {
        if let Some(field) = self.fields.get(self.focused) {
            if let FieldType::Multiline { input } = &field.field_type {
                let (row, _) = input.cursor();
                return row == 0;
            }
        }

        false
    }

    // Check if cursor of textarea is on the bottom (to focus to next field)
    fn is_cursor_at_bottom(&self) -> bool {
        if let Some(field) = self.fields.get(self.focused) {
            if let FieldType::Multiline { input } = &field.field_type {
                let (row, _) = input.cursor();
                return row == input.lines().len() - 1;
            }
        }

        false
    }

    // Submit form
    pub fn apply(&self, state: &mut ApplicationState) -> ApplicationResult<String> {
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

        let todo: Todo = Todo::new(title, description, Some(priority));
        if let Some(id) = self.task_id {
            state.update(&id, todo)
        } else {
            state.append(todo)
        }
    }

    pub fn handle_key(&mut self, event: &KeyEvent) -> WidgetResponse {
        let key: KeyCode = event.code;

        match key {
            KeyCode::Enter if self.is_button_selected() => return WidgetResponse::Submit,
            KeyCode::Esc => return WidgetResponse::Cancel,
            KeyCode::Down => {
                if !self.is_textarea_focused() || self.is_cursor_at_bottom() {
                    self.next_focus();
                    return WidgetResponse::Continue;
                }
            }
            KeyCode::Up => {
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

    // Main layout method
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

    // Buttons layout method
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

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks: std::rc::Rc<[Rect]> = self.layout(area);
        let button_layout: std::rc::Rc<[Rect]> = self.button_layout(chunks[4]);

        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = self.focused == i;

            match &field.field_type {
                FieldType::Multiline { input } => {
                    let mut input = input.clone();
                    let focused_style: Style;

                    if is_focused {
                        input.set_cursor_style(Style::default().bg(Color::White));
                        focused_style = Style::default().fg(Color::Green);
                    } else {
                        input.set_cursor_style(Style::default());
                        focused_style = Style::default();
                    }

                    let block = Block::bordered()
                        .title(" Description ")
                        .border_style(focused_style);

                    input.set_block(block);
                    frame.render_widget(&input, chunks[i]);
                }
                FieldType::Text { input } => {
                    input.render(frame, chunks[i], is_focused);
                }
                FieldType::Enum { input } => {
                    input.render(frame, chunks[i], is_focused);
                }
                FieldType::Button => {
                    let focused_style: Style = if is_focused {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    };

                    let button = Paragraph::new(" Save task ")
                        .block(Block::bordered().border_style(focused_style))
                        .centered();

                    frame.render_widget(button, button_layout[2]);
                }
            }
        }
    }
}

// Unit-tests for form
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyEventState, KeyModifiers};

    // Helper function to create key events
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
    fn should_apply_creating_task_with_textarea_data() {
        let mut state = ApplicationState::default();
        let mut form = Form::new();

        if let FieldType::Text { input } = &mut form.fields[0].field_type {
            input.buffer = "Task".to_string();
        }

        if let FieldType::Multiline { input } = &mut form.fields[2].field_type {
            input.insert_str("Multiline content");
        }

        let result = form.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.todos[0].title, "Task");
        assert_eq!(state.todos[0].description, "Multiline content");
    }

    #[test]
    fn should_apply_editing_existing_task() {
        let mut state = ApplicationState::default();
        let task = Todo::new("Old Title", "", None);
        let task_id = task.id;
        state.append(task).unwrap();

        let mut form = Form::from(&state.todos[0]);

        if let FieldType::Text { input } = &mut form.fields[0].field_type {
            input.buffer = "Updated Title".to_string();
        }

        let result = form.apply(&mut state);
        assert!(result.is_ok());

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Updated Title");
        assert_eq!(state.todos[0].id, task_id);
    }
}
