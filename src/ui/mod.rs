pub mod components;
pub mod layout;
pub mod renderer;
pub mod widgets;

#[derive(Debug, PartialEq)]
pub enum WidgetResponse {
    Continue,
    Submit,
    Cancel,
}

pub use components::{Fallback, Menu};
pub use layout::*;
pub use renderer::Renderer;
pub use widgets::{Confirm, EnumInput, Field, FieldType, Form, Notification, Popup, TextInput};
