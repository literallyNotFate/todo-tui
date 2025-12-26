// Unit-tests for todo model (basic methods)
#[cfg(test)]
mod tests {
    use crate::app::models::todo::Todo;

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
