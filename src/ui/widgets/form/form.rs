use crate::{
    models::{Priority, Todo},
    state::{ApplicationResult, ApplicationState},
    traits::Input,
    ui::{Field, FieldType, WidgetResponse},
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Paragraph,
};

#[derive(Clone, Debug)]
pub struct Form {
    pub task_id: Option<uuid::Uuid>,
    pub focused: usize,
    pub fields: Vec<Field>,
}

impl Form {
    pub fn new() -> Self {
        Self {
            fields: vec![
                Field::new("title", FieldType::text(" Title ", "")),
                Field::new("priority", FieldType::priority(Priority::Low)),
                Field::new("description", FieldType::text(" Description ", "")),
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
                Field::new(
                    "description",
                    FieldType::text(" Description ", &task.description),
                ),
            ],
            focused: 0,
            task_id: Some(task.id),
        }
    }

    // Focus on the next field
    pub fn next_focus(&mut self) {
        self.focused = (self.focused + 1) % 3;
    }

    // Focus on the prev field
    pub fn prev_focus(&mut self) {
        self.focused = (self.focused + 2) % 3;
    }

    // Submit form
    pub fn apply(&self, state: &mut ApplicationState) -> ApplicationResult<String> {
        let title: String = self
            .fields
            .iter()
            .find(|f| f.name == "title")
            .and_then(|f| f.get_text())
            .unwrap_or_default();

        let description: String = self
            .fields
            .iter()
            .find(|f| f.name == "description")
            .and_then(|f| f.get_text())
            .unwrap_or_default();

        let priority: Priority = self
            .fields
            .iter()
            .find(|f| f.name == "priority")
            .and_then(|f| f.get_priority())
            .unwrap_or(Priority::Low);

        let todo: Todo = Todo::new(title, description, Some(priority));
        if let Some(id) = self.task_id {
            state.update(&id, todo)
        } else {
            state.append(todo)
        }
    }

    pub fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse {
        match key {
            KeyCode::Enter => return WidgetResponse::Submit,
            KeyCode::Esc => return WidgetResponse::Cancel,
            KeyCode::Down => self.next_focus(),
            KeyCode::Up => self.prev_focus(),
            KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Left | KeyCode::Right => {
                if let Some(field) = self.fields.get_mut(self.focused) {
                    match &mut field.field_type {
                        FieldType::Text { input } => {
                            input.handle_key(key);
                        }
                        FieldType::Enum { input } => {
                            input.handle_key(key);
                        }
                    }
                }
            }
            _ => {}
        }

        WidgetResponse::Continue
    }

    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Priority
                Constraint::Min(10),   // Description
                Constraint::Length(3),
                Constraint::Length(1), // Buttons
            ])
            .split(area)
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks: std::rc::Rc<[Rect]> = self.layout(area);

        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = self.focused == i;

            match &field.field_type {
                FieldType::Text { input } => {
                    input.render(frame, chunks[i], is_focused);
                }
                FieldType::Enum { input } => {
                    input.render(frame, chunks[i], is_focused);
                }
            }
        }

        let buttons = Paragraph::new("[Esc: Cancel] [Enter: Save]").right_aligned();
        frame.render_widget(buttons, chunks[4]);
    }
}

// Unit-tests for form
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyCode;

    #[test]
    fn should_create_new_form() {
        let form = Form::new();
        assert_eq!(form.focused, 0);
        assert!(form.task_id.is_none());
        assert_eq!(form.fields.len(), 3);

        assert_eq!(form.fields[0].name, "title");
        assert_eq!(form.fields[1].name, "priority");
        assert_eq!(form.fields[2].name, "description");
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
    }

    #[test]
    fn should_navigate_through_form() {
        let mut form = Form::new();
        assert_eq!(form.focused, 0);

        form.handle_key(&KeyCode::Down);
        assert_eq!(form.focused, 1);

        form.handle_key(&KeyCode::Down);
        assert_eq!(form.focused, 2);

        form.handle_key(&KeyCode::Down);
        assert_eq!(form.focused, 0);

        form.handle_key(&KeyCode::Up);
        assert_eq!(form.focused, 2);
    }

    #[test]
    fn should_handle_key_for_form() {
        let mut form = Form::new();

        form.handle_key(&KeyCode::Char('R'));
        form.handle_key(&KeyCode::Char('u'));
        form.handle_key(&KeyCode::Char('s'));
        form.handle_key(&KeyCode::Char('t'));

        if let FieldType::Text { input } = &form.fields[0].field_type {
            assert_eq!(input.buffer, "Rust");
        }

        form.next_focus();
        form.handle_key(&KeyCode::Right);

        if let FieldType::Enum { input } = &form.fields[1].field_type {
            assert_ne!(input.selected, Priority::Low);
        }
    }

    #[test]
    fn should_apply_creating_new_task() {
        let mut state = ApplicationState::default();
        let mut form = Form::new();

        if let FieldType::Text { input } = &mut form.fields[0].field_type {
            input.buffer = "New Task".to_string();
        }

        let result = form.apply(&mut state);
        assert!(result.is_ok());
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "New Task");
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
