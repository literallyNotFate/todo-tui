use crate::{
    models::{Folder, FolderColor},
    ui::{Field, FieldType, Form},
};
use uuid::Uuid;

/// Folder form
pub struct FolderFormContext {
    pub folder_id: Option<Uuid>,
    pub form: Form,
}

impl FolderFormContext {
    /// Append new folder form
    pub fn append_folder() -> Self {
        Self {
            folder_id: None,
            form: Form::new(vec![
                Field::new("name", FieldType::text(" Folder Name ", "")),
                Field::new("color", FieldType::color("Blue")),
                Field::new("save", FieldType::Button),
            ]),
        }
    }

    /// Create update form with folder values
    pub fn update_folder(folder: &Folder) -> Self {
        Self {
            folder_id: Some(folder.id),
            form: Form::new(vec![
                Field::new("name", FieldType::text(" Folder Name ", &folder.name)),
                Field::new("color", FieldType::color(&folder.color)),
                Field::new("save", FieldType::Button),
            ]),
        }
    }

    /// Specific for task form method to extract color from enum field
    pub fn get_color_value(&self) -> FolderColor {
        self.form
            .fields
            .iter()
            .find(|f| f.name == "color")
            .and_then(|f| match &f.field_type {
                FieldType::ColorEnum { input } => Some(input.selected.value),
                _ => None,
            })
            .unwrap_or(FolderColor::Blue)
    }

    /// Extract and data mapping from form fields
    pub fn parse_data(&self) -> (Option<Uuid>, String, String) {
        let color: FolderColor = self.get_color_value();
        (
            self.folder_id,
            self.form.get_text_value("name"),
            color.to_string(),
        )
    }
}

/// Unit-tests for folder form
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize_empty_folder_form_for_append() {
        let context = FolderFormContext::append_folder();

        assert!(context.folder_id.is_none());
        assert_eq!(context.form.fields.len(), 3);

        assert_eq!(context.form.fields[0].name, "name");
        assert_eq!(context.form.get_text_value("name"), "");

        assert_eq!(context.form.fields[1].name, "color");
        assert_eq!(context.get_color_value(), FolderColor::Blue);
    }

    #[test]
    fn should_populate_folder_form_fields_for_update() {
        let folder = Folder::new("Work", "Red");
        let context = FolderFormContext::update_folder(&folder);

        assert_eq!(context.folder_id, Some(folder.id));
        assert_eq!(context.form.get_text_value("name"), "Work");
        assert_eq!(context.get_color_value(), FolderColor::Red);
    }

    #[test]
    fn should_parse_and_extract_folder_data_with_color_serialization() {
        let mut context = FolderFormContext::append_folder();
        context.form.set_value("name", "Personal Projects");

        if let FieldType::ColorEnum { input } = &mut context.form.fields[1].field_type {
            input.selected.value = FolderColor::Green;
        }

        let (id, name, color_str) = context.parse_data();

        assert!(id.is_none());
        assert_eq!(name, "Personal Projects");
        assert_eq!(color_str, "Green");
    }
}
