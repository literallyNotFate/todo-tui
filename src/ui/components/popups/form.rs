use crate::{
    models::{Folder, Task},
    ui::{
        FolderFormContext, Popup, PopupComponent, RenderContext, TaskFormContext, WidgetResponse,
        widgets::modal::{ModalResult, ModalSize, popup::PopupKind},
    },
};
use ratatui::{crossterm::event::KeyEvent, layout::Rect};

/// Component to render task form
impl PopupComponent for TaskFormContext {
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        self.form.render(ctx, area);
    }

    fn handle_key(&mut self, event: &KeyEvent) -> WidgetResponse {
        self.form.handle_key(event)
    }

    fn to_modal_result(&self) -> ModalResult {
        let (id, title, description, priority) = self.parse_data();
        ModalResult::TaskSubmitted {
            id,
            title,
            description,
            priority,
        }
    }
}

/// Component to render folder form
impl PopupComponent for FolderFormContext {
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        self.form.render(ctx, area);
    }

    fn handle_key(&mut self, event: &KeyEvent) -> WidgetResponse {
        self.form.handle_key(event)
    }

    fn to_modal_result(&self) -> ModalResult {
        let (id, name, color) = self.parse_data();
        ModalResult::FolderSubmitted { id, name, color }
    }
}

impl Popup {
    /// Popup to create new task form
    pub fn append_task() -> Self {
        Self::new(
            " Create Task ",
            Box::new(TaskFormContext::append_task()),
            PopupKind::Info,
        )
        .with_size(ModalSize::Large)
    }

    /// Popup to create update existing task form
    pub fn update_task(task: &Task) -> Self {
        Self::new(
            " Update Task ",
            Box::new(TaskFormContext::update_task(task)),
            PopupKind::Info,
        )
        .with_size(ModalSize::Large)
    }

    /// Popup to create new folder form
    pub fn append_folder() -> Self {
        Self::new(
            " Create Folder ",
            Box::new(FolderFormContext::append_folder()),
            PopupKind::Info,
        )
        .with_size(ModalSize::Medium)
    }

    /// Popup to create update existing task form
    pub fn update_folder(folder: &Folder) -> Self {
        Self::new(
            " Update Folder ",
            Box::new(FolderFormContext::update_folder(folder)),
            PopupKind::Info,
        )
        .with_size(ModalSize::Medium)
    }
}

/// Unit-tests for form popup components
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::{FolderColor, Priority},
        ui::FieldType,
    };
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
    fn should_create_append_task_popup_via_factory() {
        let popup = Popup::append_task();

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Create Task ");
    }

    #[test]
    fn should_create_update_task_popup_via_factory() {
        let mut task = Task::new("Fix bugs");
        task.description = "Production hotfix".to_string();
        task.priority = Priority::High;

        let popup = Popup::update_task(&task);

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Update Task ");
    }

    #[test]
    fn should_task_form_context_intercept_keys_and_convert_to_result() {
        let mut context = TaskFormContext::append_task();
        let response = context.handle_key(&key(KeyCode::Char('A')));
        assert!(matches!(response, WidgetResponse::Continue));

        context.form.set_value("title", "Review PR");
        context
            .form
            .set_value("description", "Check architecture changes");

        if let FieldType::PriorityEnum { input } = &mut context.form.fields[1].field_type {
            input.selected.value = Priority::Medium;
        }

        let modal_result = context.to_modal_result();
        assert_eq!(
            modal_result,
            ModalResult::TaskSubmitted {
                id: None,
                title: "Review PR".to_string(),
                description: "Check architecture changes".to_string(),
                priority: Priority::Medium,
            }
        );
    }

    #[test]
    fn should_create_append_folder_popup_via_factory() {
        let popup = Popup::append_folder();

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Create Folder ");
    }

    #[test]
    fn should_create_update_folder_popup_via_factory() {
        let folder = Folder::new("Personal", FolderColor::Blue);
        let popup = Popup::update_folder(&folder);

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(folder.name, "Personal");
    }

    #[test]
    fn should_folder_form_context_convert_to_result() {
        let folder = Folder::new("Archive", FolderColor::Lavender);
        let folder_id = folder.id;
        let context = FolderFormContext::update_folder(&folder);

        let modal_result = context.to_modal_result();
        assert_eq!(
            modal_result,
            ModalResult::FolderSubmitted {
                id: Some(folder_id),
                name: "Archive".to_string(),
                color: "Lavender".to_string(),
            }
        );
    }
}
