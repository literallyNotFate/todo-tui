use crate::{
    models::Priority,
    traits::Input,
    ui::{EnumInput, TextInput},
};
use tui_textarea::TextArea;

/// All possible field types for form (text input, enum input, textarea, button)
#[derive(Clone, Debug)]
pub enum FieldType {
    Text { input: TextInput },
    Enum { input: EnumInput<Priority> },
    Multiline { input: TextArea<'static> },
    Button,
}

impl FieldType {
    /// Create text input with buffer and title
    pub fn text(title: &str, value: &str) -> Self {
        Self::Text {
            input: TextInput::from(value).title(title),
        }
    }

    /// Create enum input with selected value and title
    pub fn priority(p: Priority) -> Self {
        Self::Enum {
            input: EnumInput::from(p).title(" Priority "),
        }
    }

    /// Create textarea input with buffer
    pub fn textarea(value: &str) -> Self {
        let lines: Vec<String> = value.lines().map(|s| s.to_string()).collect();

        let lines: Vec<String> = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Self::Multiline {
            input: TextArea::new(lines),
        }
    }
}

/// Field object in form
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
}
