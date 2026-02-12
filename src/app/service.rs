use crate::{
    core::TodoError,
    models::{Filter, Priority, Sort, Todo},
    state::ApplicationResult,
};
use chrono::Local;
use uuid::Uuid;

pub struct TodoService;

/// Main service methods (only for todos)
impl TodoService {
    /// Append new task to the end of list
    pub fn append_task(
        todos: &mut Vec<Todo>,
        task: Todo,
        sort: &Sort,
    ) -> ApplicationResult<String> {
        if task.title.trim().is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        let title: String = task.title.clone();
        todos.push(task);
        Self::sorting(todos, sort);

        Ok(title)
    }

    /// Update task by id
    pub fn update_task(
        todos: &mut [Todo],
        id: &Uuid,
        task: Todo,
        sort: &Sort,
    ) -> ApplicationResult<usize> {
        if task.title.trim().is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        let index: usize = todos
            .iter()
            .position(|t| t.id == *id)
            .ok_or(TodoError::TaskNotFound)?;

        todos[index].update(task);
        Self::sorting(todos, sort);

        let new_index: usize = todos.iter().position(|t| t.id == *id).unwrap();
        Ok(new_index)
    }

    /// Remove task by id
    pub fn remove_task(todos: &mut Vec<Todo>, id: &Uuid) -> ApplicationResult<String> {
        let index: usize = todos
            .iter()
            .position(|t| t.id == *id)
            .ok_or(TodoError::TaskNotFound)?;
        Ok(todos.remove(index).title)
    }

    /// Toggle completed/uncompleted by id
    pub fn toggle_task(todos: &mut [Todo], id: &Uuid) -> ApplicationResult<()> {
        let task: &mut Todo = todos
            .iter_mut()
            .find(|t| t.id == *id)
            .ok_or(TodoError::TaskNotFound)?;

        task.toggle_completed();
        Ok(())
    }

    /// Clear todos that being filtered by current filter
    pub fn clear(todos: &mut Vec<Todo>, filter: &Filter) -> usize {
        let initial_len: usize = todos.len();

        match filter {
            Filter::All => todos.clear(),
            Filter::Completed => todos.retain(|t| !t.completed),
            Filter::Active => todos.retain(|t| t.completed),
            Filter::HighPriority => todos.retain(|t| t.priority != Priority::High),
            Filter::Today => {
                let today = Local::now().date_naive();
                todos.retain(|t| t.created_at.with_timezone(&Local).date_naive() != today);
            }
        }

        initial_len - todos.len()
    }

    /// Automatic soritng by priority (considering updated_at also) after operation
    pub fn sorting(todos: &mut [Todo], sort: &Sort) {
        todos.sort_by(|a, b| sort.compare(a, b));
    }

    /// Move tasks by indices (change order of them)
    pub fn move_tasks(todos: &mut [Todo], a: usize, b: usize) -> ApplicationResult<()> {
        if a >= todos.len() || b >= todos.len() {
            return Err(TodoError::TaskNotFound.into());
        }

        if a == b {
            return Ok(());
        }

        if todos[a].priority != todos[b].priority {
            return Err(TodoError::MoveForbidden.into());
        }

        todos.swap(a, b);
        Ok(())
    }
}

/// Unit-tests for todo service
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ApplicationError;

    #[test]
    fn should_append_task_service() {
        let mut todos: Vec<Todo> = Vec::new();
        let task_to_add: Todo = Todo::new("Buy stuff", "Just buy stuff", Some(Priority::High));

        let result: ApplicationResult<String> =
            TodoService::append_task(&mut todos, task_to_add, &Sort::default());
        let added_task: &Todo = &todos[0];

        assert!(result.is_ok());
        assert_eq!(todos.len(), 1);
        assert_eq!(result.unwrap(), added_task.title);
    }

    #[test]
    fn should_fail_append_task_service_on_empty_title() {
        let mut todos: Vec<Todo> = Vec::new();
        let task_to_add: Todo = Todo::new("", "Just buy stuff", Some(Priority::High));
        let result: ApplicationResult<String> =
            TodoService::append_task(&mut todos, task_to_add, &Sort::default());

        assert!(matches!(
            result,
            Err(ApplicationError::Todo(TodoError::EmptyTitle))
        ));
        assert!(todos.is_empty());
    }

    #[test]
    fn should_update_task_service() {
        let mut todos: Vec<Todo> = vec![Todo::new(
            "Old title",
            "Description",
            Some(Priority::Medium),
        )];
        let id: Uuid = todos[0].id;
        let updated_data: Todo = Todo::new("New Title", "Description", Some(Priority::High));

        let result: ApplicationResult<usize> =
            TodoService::update_task(&mut todos, &id, updated_data, &Sort::default());

        assert!(result.is_ok());
        assert_eq!(result, Ok(0));
        assert_eq!(todos[0].title, "New Title");
        assert_eq!(todos[0].priority, Priority::High);
    }

    #[test]
    fn should_fail_update_task_service_with_empty_title() {
        let mut todos: Vec<Todo> = vec![Todo::new("Valid", "Desc", None)];
        let id: Uuid = todos[0].id;
        let invalid_data: Todo = Todo::new("", "Desc", None);

        let result: ApplicationResult<usize> =
            TodoService::update_task(&mut todos, &id, invalid_data, &Sort::default());

        assert!(result.is_err());
        assert_eq!(result, Err(ApplicationError::Todo(TodoError::EmptyTitle)));
        assert_eq!(todos[0].title, "Valid");
    }

    #[test]
    fn should_fail_update_task_service_with_wrong_id() {
        let mut todos: Vec<Todo> = vec![Todo::new("Task", "", None)];
        let fake_id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<usize> = TodoService::update_task(
            &mut todos,
            &fake_id,
            Todo::new("X", "", None),
            &Sort::default(),
        );

        assert!(result.is_err());
        assert_eq!(result, Err(ApplicationError::Todo(TodoError::TaskNotFound)));
    }

    #[test]
    fn should_remove_task_service() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("Task 1", "Desc", None),
            Todo::new("Task 2", "Desc", None),
        ];
        let id_to_remove: Uuid = todos[0].id;

        let result: ApplicationResult<String> = TodoService::remove_task(&mut todos, &id_to_remove);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Task 1");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Task 2");
    }

    #[test]
    fn should_fail_remove_task_service_with_wrong_id() {
        let mut todos: Vec<Todo> = vec![Todo::new("Task", "", None)];
        let fake_id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<String> = TodoService::remove_task(&mut todos, &fake_id);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotFound))
        ));
    }

    #[test]
    fn should_toggle_task_service() {
        let mut todos: Vec<Todo> = vec![Todo::new("Toggle Me", "", None)];
        let id: Uuid = todos[0].id;
        assert!(!todos[0].completed);

        TodoService::toggle_task(&mut todos, &id).unwrap();
        assert!(todos[0].completed);

        TodoService::toggle_task(&mut todos, &id).unwrap();
        assert!(!todos[0].completed);
    }

    #[test]
    fn should_fail_toggle_task_service_with_wrong_id() {
        let mut todos: Vec<Todo> = vec![Todo::new("Toggle Me", "", None)];
        let fake_id: Uuid = Uuid::new_v4();
        assert!(!todos[0].completed);

        let result: ApplicationResult<()> = TodoService::toggle_task(&mut todos, &fake_id);

        assert!(!todos[0].completed);
        assert_eq!(result, Err(ApplicationError::Todo(TodoError::TaskNotFound)))
    }

    #[test]
    fn should_clear_tasks_service_completed() {
        let mut todos: Vec<Todo> = vec![Todo::new("Active", "", None), Todo::new("Done", "", None)];
        todos[1].completed = true;

        let removed_count: usize = TodoService::clear(&mut todos, &Filter::Completed);

        assert_eq!(removed_count, 1);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Active");
    }

    #[test]
    fn should_sort_by_priority_automatically() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("Low Task", "", None),
            Todo::new("High Task", "", Some(Priority::High)),
        ];

        TodoService::sorting(&mut todos, &Sort::default());

        assert_eq!(todos[0].title, "High Task");
        assert_eq!(todos[1].title, "Low Task");
    }

    #[test]
    fn should_move_tasks_successully_with_same_priority() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("Task 1", "", Some(Priority::High)),
            Todo::new("Task 2", "", Some(Priority::High)),
        ];

        let result: ApplicationResult<()> = TodoService::move_tasks(&mut todos, 0, 1);

        assert!(result.is_ok());
        assert_eq!(todos[0].title, "Task 2");
        assert_eq!(todos[1].title, "Task 1");
    }

    #[test]
    fn should_not_move_tasks_with_different_priorities() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("High Task", "", Some(Priority::High)),
            Todo::new("Medium Task", "", Some(Priority::Medium)),
        ];

        let result: ApplicationResult<()> = TodoService::move_tasks(&mut todos, 0, 1);

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::MoveForbidden.into()))
        );
        assert_eq!(todos[0].title, "High Task");
        assert_eq!(todos[1].title, "Medium Task");
    }

    #[test]
    fn should_test_boundaries_on_move_tasks_service() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("Task 1", "", Some(Priority::Low)),
            Todo::new("Task 2", "", Some(Priority::High)),
        ];

        let res_same = TodoService::move_tasks(&mut todos, 0, 0);
        assert!(
            res_same.is_ok(),
            "Moving to the same index should be ignored"
        );

        let res_diff = TodoService::move_tasks(&mut todos, 0, 1);
        assert!(
            res_diff.is_err(),
            "Should be MoveForbidden due to priority difference"
        );
    }
}
