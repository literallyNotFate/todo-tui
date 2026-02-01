use crate::{
    models::Priority,
    traits::Input,
    ui::{EnumInput, TextInput},
};
use tui_textarea::TextArea;

#[derive(Clone, Debug)]
pub enum FieldType<'a> {
    Text { input: TextInput },
    Enum { input: EnumInput<Priority> },
    Multiline { input: TextArea<'a> },
    Button,
}

impl<'a> FieldType<'a> {
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

    pub fn textarea(value: &str) -> Self {
        Self::Multiline {
            input: TextArea::new(vec![value.to_string()]),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Field<'a> {
    pub name: String,
    pub field_type: FieldType<'a>,
}

impl<'a> Field<'a> {
    pub fn new(field_name: impl Into<String>, field_type: FieldType<'a>) -> Self {
        Self {
            name: field_name.into(),
            field_type,
        }
    }
}
