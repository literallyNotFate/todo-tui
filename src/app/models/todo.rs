#[derive(Debug, Default)]
pub struct Todo {
    pub title: String,
    pub done: bool,
}

impl Todo {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
            done: false,
        }
    }

    pub fn toggle_done(&mut self) {
        self.done = !self.done;
    }

    pub fn rename<T: Into<String>>(&mut self, new_name: T) {
        self.title = new_name.into();
    }
}

// Unit-tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_todo_item() {
        let todo: Todo = Todo::new("Test task");

        assert_eq!(todo.title, "Test task");
        assert!(!todo.done);
    }

    #[test]
    fn should_toggle_complete() {
        let mut todo: Todo = Todo::new("Test task");

        todo.toggle_done();
        assert!(todo.done);

        todo.toggle_done();
        assert!(!todo.done);
    }

    #[test]
    fn should_rename_todo() {
        let mut todo: Todo = Todo::new("Test task");
        assert_eq!(todo.title, "Test task");

        todo.rename("Renamed task");
        assert_eq!(todo.title, "Renamed task");
    }
}
