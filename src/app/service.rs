use super::{TaskCreatedResult, TaskRemovedResult, TaskUpdatedResult};
use crate::{
    core::{Sort, TodoError},
    models::{Filter, Priority, Todo, todo::TodoEditor},
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
    ) -> ApplicationResult<TaskCreatedResult> {
        if task.title.trim().is_empty() {
            log::debug!("Validation error on append: Task title is empty");
            return Err(TodoError::EmptyTitle.into());
        }

        let title: String = task.title.clone();
        log::info!("Adding new task: '{}' (ID: {})", title, task.id);

        todos.push(task.clone());
        Self::sorting(todos, sort);
        let index = todos.iter().position(|t| t.id == task.id).unwrap();

        Ok(TaskCreatedResult { index, task })
    }

    /// Update task by id using TodoEditor model
    pub fn update_task(
        todos: &mut [Todo],
        id: &Uuid,
        editor: TodoEditor,
        sort: &Sort,
    ) -> ApplicationResult<TaskUpdatedResult> {
        if editor.title.trim().is_empty() {
            log::debug!("Validation error on update: Task title is empty");
            return Err(TodoError::EmptyTitle.into());
        }

        let original_index: usize = todos.iter().position(|t| t.id == *id).ok_or_else(|| {
            log::warn!(
                "Update failed: Task with ID {} not found in current list",
                id
            );
            TodoError::TaskNotFound
        })?;

        let old: Todo = todos[original_index].clone();
        todos[original_index].update_from_editor(editor);
        let new: Todo = todos[original_index].clone();

        log::info!(
            "Task updated successfully: '{}' (ID: {}). Changes: [Title: '{}' -> '{}', Priority: {:?} -> {:?}]",
            new.title,
            id,
            old.title,
            new.title,
            old.priority,
            new.priority
        );

        Self::sorting(todos, sort);
        let new_index: usize = todos
            .iter()
            .position(|t| t.id == *id)
            .expect("Task must exist");

        Ok(TaskUpdatedResult {
            index: new_index,
            old,
            new,
        })
    }

    /// Remove task by id
    pub fn remove_task(todos: &mut Vec<Todo>, id: &Uuid) -> ApplicationResult<TaskRemovedResult> {
        let index: usize = todos.iter().position(|t| t.id == *id).ok_or_else(|| {
            log::error!(
                "Remove failed: Attempted to remove non-existent task ID {}",
                id
            );
            TodoError::TaskNotFound
        })?;

        let task: Todo = todos.remove(index);
        log::info!("Task removed: '{}' (ID: {})", task.title, id);
        Ok(TaskRemovedResult { task })
    }

    /// Toggle completed/uncompleted by id
    pub fn toggle_task(todos: &mut [Todo], id: &Uuid) -> ApplicationResult<()> {
        let task: &mut Todo = todos.iter_mut().find(|t| t.id == *id).ok_or_else(|| {
            log::error!(
                "Toggle failed: Attempted to toggle completed the non-existent task ID {}",
                id
            );
            TodoError::TaskNotFound
        })?;

        task.toggle_completed();
        log::debug!(
            "Task {} toggled. New state: completed={}",
            id,
            task.completed
        );

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

        let removed: usize = initial_len - todos.len();
        log::info!(
            "Massive clear performed. Filter: {:?}, Removed: {} tasks",
            filter,
            removed
        );

        removed
    }

    /// Automatic soritng by priority (considering updated_at also) after operation
    pub fn sorting(todos: &mut [Todo], sort: &Sort) {
        todos.sort_by(|a, b| sort.compare(a, b));
    }

    /// Move tasks by indices (change order of them)
    pub fn move_tasks(todos: &mut [Todo], a: usize, b: usize) -> ApplicationResult<()> {
        if a >= todos.len() || b >= todos.len() {
            log::warn!(
                "Move failed: Indices out of bounds (a: {}, b: {}, total len: {})",
                a,
                b,
                todos.len()
            );
            return Err(TodoError::TaskNotFound.into());
        }

        if a == b {
            return Ok(());
        }

        if todos[a].priority != todos[b].priority {
            log::warn!(
                "Move forbidden: tasks have different priorities ({:?} vs {:?})",
                todos[a].priority,
                todos[b].priority
            );
            return Err(TodoError::MoveForbidden.into());
        }

        log::debug!("Swapping tasks at indices {} and {}", a, b);
        todos.swap(a, b);
        Ok(())
    }
}

/// Unit-tests for todo service
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ApplicationError, Selectable};

    #[test]
    fn should_append_task_service() {
        let mut todos: Vec<Todo> = Vec::new();
        let task_to_add: Todo = Todo::new("Buy stuff", "Just buy stuff", Some(Priority::High));

        let result: ApplicationResult<TaskCreatedResult> =
            TodoService::append_task(&mut todos, task_to_add, &Sort::default());
        let added_task: &Todo = &todos[0];

        assert!(result.is_ok());
        assert_eq!(todos.len(), 1);
        assert_eq!(result.unwrap().task.title, added_task.title);
    }

    #[test]
    fn should_fail_append_task_service_on_empty_title() {
        let mut todos: Vec<Todo> = Vec::new();
        let task_to_add: Todo = Todo::new("", "Just buy stuff", Some(Priority::High));
        let result: ApplicationResult<TaskCreatedResult> =
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
        let editor: TodoEditor = TodoEditor {
            title: "New Title".into(),
            description: "Description".into(),
            priority: Selectable::new(Priority::High),
        };

        let result = TodoService::update_task(&mut todos, &id, editor, &Sort::default());
        assert!(result.is_ok());

        let res = result.unwrap();

        assert_eq!(res.index, 0);
        assert_eq!(res.old.title, "Old title");
        assert_eq!(res.new.title, "New Title");
        assert_eq!(res.new.priority, Priority::High);
        assert_eq!(todos[0].title, "New Title");
    }

    #[test]
    fn should_fail_update_task_service_with_empty_title() {
        let mut todos: Vec<Todo> = vec![Todo::new("Valid", "Desc", None)];
        let id: Uuid = todos[0].id;
        let editor: TodoEditor = TodoEditor {
            title: "".into(),
            description: "Desc".into(),
            priority: Selectable::new(Priority::Low),
        };

        let result: ApplicationResult<TaskUpdatedResult> =
            TodoService::update_task(&mut todos, &id, editor, &Sort::default());

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Todo(TodoError::EmptyTitle))
        ));
        assert_eq!(todos[0].title, "Valid");
    }

    #[test]
    fn should_fail_update_task_service_with_wrong_id() {
        let mut todos: Vec<Todo> = vec![Todo::new("Task", "", None)];
        let fake_id: Uuid = Uuid::new_v4();
        let editor: TodoEditor = TodoEditor {
            title: "X".into(),
            description: "".into(),
            priority: Selectable::new(Priority::Low),
        };

        let result: ApplicationResult<TaskUpdatedResult> =
            TodoService::update_task(&mut todos, &fake_id, editor, &Sort::default());

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotFound))
        ));
    }

    #[test]
    fn should_remove_task_service() {
        let mut todos: Vec<Todo> = vec![
            Todo::new("Task 1", "Desc", None),
            Todo::new("Task 2", "Desc", None),
        ];
        let id_to_remove: Uuid = todos[0].id;

        let result: ApplicationResult<TaskRemovedResult> =
            TodoService::remove_task(&mut todos, &id_to_remove);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().task.title, "Task 1");
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "Task 2");
    }

    #[test]
    fn should_fail_remove_task_service_with_wrong_id() {
        let mut todos: Vec<Todo> = vec![Todo::new("Task", "", None)];
        let fake_id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<TaskRemovedResult> =
            TodoService::remove_task(&mut todos, &fake_id);

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
