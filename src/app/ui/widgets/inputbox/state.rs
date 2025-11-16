#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Insert,
    Edit,
}

pub enum InputResult {
    Continue,
    Submit(String),
    Cancel,
}
