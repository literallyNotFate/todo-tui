#[derive(Debug, Clone, Default)]
pub struct Todo {
    pub title: String,
    pub done: bool,
}

impl Todo {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            done: false,
        }
    }

    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }

    pub fn rename(&mut self, new_name: impl Into<String>) {
        self.title = new_name.into();
    }
}
