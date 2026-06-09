use super::{TaskCreatedResult, TaskRemovedResult, TaskUpdatedResult};
use crate::{
    core::{Sort, TaskError},
    models::{Filter, Priority, Task, task::TaskEditor},
    state::ApplicationResult,
};
use chrono::Local;
use uuid::Uuid;

/// Main service methods (only for tasks)
pub struct TaskService;

impl TaskService {
    /// Append new task to the end of list
    pub fn append_task(
        tasks: &mut Vec<Task>,
        task: Task,
        sort: &Sort,
    ) -> ApplicationResult<TaskCreatedResult> {
        if task.title.trim().is_empty() {
            log::debug!("Validation error on append: Task title is empty");
            return Err(TaskError::EmptyTitle.into());
        }

        let title: String = task.title.clone();
        log::info!("Adding new task: '{}' (ID: {})", title, task.id);

        tasks.push(task.clone());
        Self::sorting(tasks, sort);
        let index = tasks.iter().position(|t| t.id == task.id).unwrap();

        Ok(TaskCreatedResult { index, task })
    }

    /// Update task by id using TaskEditor model
    pub fn update_task(
        tasks: &mut [Task],
        id: &Uuid,
        editor: TaskEditor,
        sort: &Sort,
    ) -> ApplicationResult<TaskUpdatedResult> {
        if editor.title.trim().is_empty() {
            log::debug!("Validation error on update: Task title is empty");
            return Err(TaskError::EmptyTitle.into());
        }

        let original_index: usize = tasks.iter().position(|t| t.id == *id).ok_or_else(|| {
            log::warn!(
                "Update failed: Task with ID {} not found in current list",
                id
            );
            TaskError::TaskNotFound
        })?;

        let old: Task = tasks[original_index].clone();
        tasks[original_index].update_from_editor(editor);
        let new: Task = tasks[original_index].clone();

        log::info!(
            "Task updated successfully: '{}' (ID: {}). Changes: [Title: '{}' -> '{}', Priority: {:?} -> {:?}]",
            new.title,
            id,
            old.title,
            new.title,
            old.priority,
            new.priority
        );

        Self::sorting(tasks, sort);
        let new_index: usize = tasks
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
    pub fn remove_task(tasks: &mut Vec<Task>, id: &Uuid) -> ApplicationResult<TaskRemovedResult> {
        let index: usize = tasks.iter().position(|t| t.id == *id).ok_or_else(|| {
            log::error!(
                "Remove failed: Attempted to remove non-existent task ID {}",
                id
            );
            TaskError::TaskNotFound
        })?;

        let task: Task = tasks.remove(index);
        log::info!("Task removed: '{}' (ID: {})", task.title, id);
        Ok(TaskRemovedResult { task })
    }

    /// Toggle completed/uncompleted by id
    pub fn toggle_task(tasks: &mut [Task], id: &Uuid) -> ApplicationResult<()> {
        let task: &mut Task = tasks.iter_mut().find(|t| t.id == *id).ok_or_else(|| {
            log::error!(
                "Toggle failed: Attempted to toggle completed the non-existent task ID {}",
                id
            );
            TaskError::TaskNotFound
        })?;

        task.toggle_completed();
        log::debug!(
            "Task {} toggled. New state: completed={}",
            id,
            task.completed
        );

        Ok(())
    }

    /// Clear tasks that being filtered by current filter
    pub fn clear(tasks: &mut Vec<Task>, filter: &Filter) -> usize {
        let initial_len: usize = tasks.len();

        match filter {
            Filter::All => tasks.clear(),
            Filter::Completed => tasks.retain(|t| !t.completed),
            Filter::Active => tasks.retain(|t| t.completed),
            Filter::HighPriority => tasks.retain(|t| t.priority != Priority::High),
            Filter::Today => {
                let today = Local::now().date_naive();
                tasks.retain(|t| t.created_at.with_timezone(&Local).date_naive() != today);
            }
        }

        let removed: usize = initial_len - tasks.len();
        log::info!(
            "Massive clear performed. Filter: {:?}, Removed: {} tasks",
            filter,
            removed
        );

        removed
    }

    /// Automatic soritng by priority (considering updated_at also) after operation
    pub fn sorting(tasks: &mut [Task], sort: &Sort) {
        tasks.sort_by(|a, b| sort.compare(a, b));
    }

    /// Move tasks by indices (change order of them)
    pub fn move_tasks(tasks: &mut [Task], a: usize, b: usize) -> ApplicationResult<()> {
        if a >= tasks.len() || b >= tasks.len() {
            log::warn!(
                "Move failed: Indices out of bounds (a: {}, b: {}, total len: {})",
                a,
                b,
                tasks.len()
            );
            return Err(TaskError::TaskNotFound.into());
        }

        if a == b {
            return Ok(());
        }

        if tasks[a].priority != tasks[b].priority {
            log::warn!(
                "Move forbidden: tasks have different priorities ({:?} vs {:?})",
                tasks[a].priority,
                tasks[b].priority
            );
            return Err(TaskError::MoveForbidden.into());
        }

        log::debug!("Swapping tasks at indices {} and {}", a, b);
        tasks.swap(a, b);
        Ok(())
    }
}

/// Unit-tests for task service
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ApplicationError, Selectable};

    #[test]
    fn should_append_task_service() {
        let mut tasks: Vec<Task> = Vec::new();
        let task_to_add: Task = Task::new("Buy stuff")
            .with_description("Just buy stuff")
            .with_priority(Priority::High);

        let result: ApplicationResult<TaskCreatedResult> =
            TaskService::append_task(&mut tasks, task_to_add, &Sort::default());
        let added_task: &Task = &tasks[0];

        assert!(result.is_ok());
        assert_eq!(tasks.len(), 1);
        assert_eq!(result.unwrap().task.title, added_task.title);
    }

    #[test]
    fn should_fail_append_task_service_on_empty_title() {
        let mut tasks: Vec<Task> = Vec::new();
        let task_to_add: Task = Task::new("");
        let result: ApplicationResult<TaskCreatedResult> =
            TaskService::append_task(&mut tasks, task_to_add, &Sort::default());

        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::EmptyTitle))
        ));
        assert!(tasks.is_empty());
    }

    #[test]
    fn should_update_task_service() {
        let mut tasks: Vec<Task> = vec![
            Task::new("Old title")
                .with_description("Description")
                .with_priority(Priority::Medium),
        ];
        let id: Uuid = tasks[0].id;
        let editor: TaskEditor = TaskEditor {
            title: "New Title".into(),
            description: "Description".into(),
            priority: Selectable::new(Priority::High),
        };

        let result = TaskService::update_task(&mut tasks, &id, editor, &Sort::default());
        assert!(result.is_ok());

        let res = result.unwrap();

        assert_eq!(res.index, 0);
        assert_eq!(res.old.title, "Old title");
        assert_eq!(res.new.title, "New Title");
        assert_eq!(res.new.priority, Priority::High);
        assert_eq!(tasks[0].title, "New Title");
    }

    #[test]
    fn should_fail_update_task_service_with_empty_title() {
        let mut tasks: Vec<Task> = vec![Task::new("Valid").with_description("Desc")];
        let id: Uuid = tasks[0].id;
        let editor: TaskEditor = TaskEditor {
            title: "".into(),
            description: "Desc".into(),
            priority: Selectable::new(Priority::Low),
        };

        let result: ApplicationResult<TaskUpdatedResult> =
            TaskService::update_task(&mut tasks, &id, editor, &Sort::default());

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::EmptyTitle))
        ));
        assert_eq!(tasks[0].title, "Valid");
    }

    #[test]
    fn should_fail_update_task_service_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Task")];
        let fake_id: Uuid = Uuid::new_v4();
        let editor: TaskEditor = TaskEditor {
            title: "X".into(),
            description: "".into(),
            priority: Selectable::new(Priority::Low),
        };

        let result: ApplicationResult<TaskUpdatedResult> =
            TaskService::update_task(&mut tasks, &fake_id, editor, &Sort::default());

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::TaskNotFound))
        ));
    }

    #[test]
    fn should_remove_task_service() {
        let mut tasks: Vec<Task> = vec![Task::new("Task 1"), Task::new("Task 2")];
        let id_to_remove: Uuid = tasks[0].id;

        let result: ApplicationResult<TaskRemovedResult> =
            TaskService::remove_task(&mut tasks, &id_to_remove);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().task.title, "Task 1");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task 2");
    }

    #[test]
    fn should_fail_remove_task_service_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Task")];
        let fake_id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<TaskRemovedResult> =
            TaskService::remove_task(&mut tasks, &fake_id);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::TaskNotFound))
        ));
    }

    #[test]
    fn should_toggle_task_service() {
        let mut tasks: Vec<Task> = vec![Task::new("Toggle Me")];
        let id: Uuid = tasks[0].id;
        assert!(!tasks[0].completed);

        TaskService::toggle_task(&mut tasks, &id).unwrap();
        assert!(tasks[0].completed);

        TaskService::toggle_task(&mut tasks, &id).unwrap();
        assert!(!tasks[0].completed);
    }

    #[test]
    fn should_fail_toggle_task_service_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Toggle Me")];
        let fake_id: Uuid = Uuid::new_v4();
        assert!(!tasks[0].completed);

        let result: ApplicationResult<()> = TaskService::toggle_task(&mut tasks, &fake_id);

        assert!(!tasks[0].completed);
        assert_eq!(result, Err(ApplicationError::Task(TaskError::TaskNotFound)))
    }

    #[test]
    fn should_clear_tasks_service_completed() {
        let mut tasks: Vec<Task> = vec![Task::new("Active"), Task::new("Done")];
        tasks[1].completed = true;

        let removed_count: usize = TaskService::clear(&mut tasks, &Filter::Completed);

        assert_eq!(removed_count, 1);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Active");
    }

    #[test]
    fn should_sort_by_priority_automatically() {
        let mut tasks: Vec<Task> = vec![
            Task::new("Low Task"),
            Task::new("High Task").with_priority(Priority::High),
        ];

        TaskService::sorting(&mut tasks, &Sort::default());

        assert_eq!(tasks[0].title, "High Task");
        assert_eq!(tasks[1].title, "Low Task");
    }

    #[test]
    fn should_move_tasks_successully_with_same_priority() {
        let mut tasks: Vec<Task> = vec![
            Task::new("Task 1").with_priority(Priority::High),
            Task::new("Task 2").with_priority(Priority::High),
        ];

        let result: ApplicationResult<()> = TaskService::move_tasks(&mut tasks, 0, 1);

        assert!(result.is_ok());
        assert_eq!(tasks[0].title, "Task 2");
        assert_eq!(tasks[1].title, "Task 1");
    }

    #[test]
    fn should_not_move_tasks_with_different_priorities() {
        let mut tasks: Vec<Task> = vec![
            Task::new("High Task").with_priority(Priority::High),
            Task::new("Medium Task").with_priority(Priority::Medium),
        ];

        let result: ApplicationResult<()> = TaskService::move_tasks(&mut tasks, 0, 1);

        assert_eq!(
            result,
            Err(ApplicationError::Task(TaskError::MoveForbidden))
        );
        assert_eq!(tasks[0].title, "High Task");
        assert_eq!(tasks[1].title, "Medium Task");
    }

    #[test]
    fn should_test_boundaries_on_move_tasks_service() {
        let mut tasks: Vec<Task> = vec![
            Task::new("Task 1"),
            Task::new("Task 2").with_priority(Priority::High),
        ];

        let res_same = TaskService::move_tasks(&mut tasks, 0, 0);
        assert!(
            res_same.is_ok(),
            "Moving to the same index should be ignored"
        );

        let res_diff = TaskService::move_tasks(&mut tasks, 0, 1);
        assert!(
            res_diff.is_err(),
            "Should be MoveForbidden due to priority difference"
        );
    }
}
