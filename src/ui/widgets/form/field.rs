use crate::{
    models::Priority,
    traits::Input,
    ui::{EnumInput, TextInput},
};

#[derive(Clone, Debug)]
pub enum FieldType {
    Text { input: TextInput },
    Enum { input: EnumInput<Priority> },
}

impl FieldType {
    pub fn text(title: &str, value: &str) -> Self {
        Self::Text {
            input: TextInput::from(value).title(title),
        }
    }

    pub fn priority(p: Priority) -> Self {
        Self::Enum {
            input: EnumInput::from(p).title(" Priority "),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

impl Field {
    pub fn new(field_name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: field_name.into(),
            field_type,
        }
    }

    pub fn get_text(&self) -> Option<String> {
        if let FieldType::Text { input } = &self.field_type {
            Some(input.buffer.clone())
        } else {
            None
        }
    }

    pub fn get_priority(&self) -> Option<Priority> {
        if let FieldType::Enum { input } = &self.field_type {
            Some(input.selected.clone())
        } else {
            None
        }
    }
}
