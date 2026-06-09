use crate::{
    models::{Priority, Task},
    ui::{Field, FieldType, Form},
};
use uuid::Uuid;

/// Task form
pub struct TaskFormContext {
    pub task_id: Option<Uuid>,
    pub form: Form,
}

impl TaskFormContext {
    /// Append new task form
    pub fn append_task() -> Self {
        Self {
            task_id: None,
            form: Form::new(vec![
                Field::new("title", FieldType::text(" Title ", "")),
                Field::new("priority", FieldType::priority(Priority::Low)),
                Field::new("description", FieldType::textarea("")),
                Field::new("save", FieldType::Button),
            ]),
        }
    }

    /// Create update form with task values
    pub fn update_task(task: &Task) -> Self {
        Self {
            task_id: Some(task.id),
            form: Form::new(vec![
                Field::new("title", FieldType::text(" Title ", &task.title)),
                Field::new("priority", FieldType::priority(task.priority)),
                Field::new("description", FieldType::textarea(&task.description)),
                Field::new("save", FieldType::Button),
            ]),
        }
    }

    /// Specific for task form method to extract priroity from enum field
    pub fn get_priority_value(&self) -> Priority {
        self.form
            .fields
            .iter()
            .find(|f| f.name == "priority")
            .and_then(|f| match &f.field_type {
                FieldType::PriorityEnum { input } => Some(*input.selected),
                _ => None,
            })
            .unwrap_or(Priority::Low)
    }

    /// Extract and data mapping from form fields
    pub fn parse_data(&self) -> (Option<Uuid>, String, String, Priority) {
        (
            self.task_id,
            self.form.get_text_value("title").trim().to_string(),
            self.form.get_text_value("description").trim().to_string(),
            self.get_priority_value(),
        )
    }
}

/// Unit-tests for task form
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize_empty_task_form_for_append() {
        let context = TaskFormContext::append_task();

        assert!(context.task_id.is_none());
        assert_eq!(context.form.fields.len(), 4);
        assert_eq!(context.form.fields[0].name, "title");
        assert_eq!(context.form.get_text_value("title"), "");
        assert_eq!(context.form.fields[1].name, "priority");
        assert_eq!(context.get_priority_value(), Priority::Low);
        assert_eq!(context.form.fields[2].name, "description");
        assert_eq!(context.form.get_text_value("description"), "");
    }

    #[test]
    fn should_populate_task_form_fields_for_update() {
        let task = Task::new("Buy milk")
            .with_description("2% fat preferred")
            .with_priority(Priority::High);
        let context = TaskFormContext::update_task(&task);

        assert_eq!(context.task_id, Some(task.id));
        assert_eq!(context.form.get_text_value("title"), "Buy milk");
        assert_eq!(
            context.form.get_text_value("description"),
            "2% fat preferred"
        );
        assert_eq!(context.get_priority_value(), Priority::High);
    }

    #[test]
    fn should_parse_and_extract_task_data_correctly() {
        let mut context = TaskFormContext::append_task();

        context.form.set_value("title", "Refactor Forms");
        context
            .form
            .set_value("description", "Separate task and folder contexts");

        if let FieldType::PriorityEnum { input } = &mut context.form.fields[1].field_type {
            input.selected.value = Priority::Medium;
        }

        let (id, title, description, priority) = context.parse_data();

        assert!(id.is_none());
        assert_eq!(title, "Refactor Forms");
        assert_eq!(description, "Separate task and folder contexts");
        assert_eq!(priority, Priority::Medium);
    }
}
