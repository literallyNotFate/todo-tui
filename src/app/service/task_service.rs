use crate::{
    app::OperationResult,
    core::{Sort, TaskError},
    models::{Priority, Task, task::TaskEditor},
    state::{ApplicationResult, SidebarTab},
};
use chrono::Local;
use uuid::Uuid;

/// Main service methods (only for tasks)
pub struct TaskService;

impl TaskService {
    /// Append new task to the end of list
    pub fn append_task(tasks: &mut Vec<Task>, task: Task) -> ApplicationResult<OperationResult> {
        task.validate()?;
        log::info!("Adding new task: '{}' (ID: {})", task.title, task.id);
        tasks.push(task.clone());

        Ok(OperationResult::TaskCreated { task })
    }

    /// Update task by id using TaskEditor model
    pub fn update_task(
        tasks: &mut [Task],
        id: &Uuid,
        editor: TaskEditor,
    ) -> ApplicationResult<OperationResult> {
        let index: usize = Self::find_index(tasks, id)?;

        let mut temp: Task = tasks[index].clone();
        temp.update_from(editor.clone());
        temp.validate()?;

        let old: Task = tasks[index].clone();
        tasks[index] = temp;
        let new: Task = tasks[index].clone();

        log::info!(
            "Task updated successfully: '{}' (ID: {}). Changes: [Title: '{}' -> '{}', Priority: {:?} -> {:?}]",
            new.title,
            id,
            old.title,
            new.title,
            old.priority,
            new.priority
        );

        Ok(OperationResult::TaskUpdated { old, new })
    }

    /// Remove task by id
    pub fn remove_task(tasks: &mut Vec<Task>, id: &Uuid) -> ApplicationResult<OperationResult> {
        let index: usize = Self::find_index(tasks, id)?;
        let task: Task = tasks.remove(index);

        log::info!("Task removed: '{}' (ID: {})", task.title, id);
        Ok(OperationResult::TaskRemoved { task })
    }

    /// Toggle completed/uncompleted by id
    pub fn toggle_completed(tasks: &mut [Task], id: &Uuid) -> ApplicationResult<()> {
        let task: &mut Task = Self::find_task_mut(tasks, id)?;
        task.toggle_completed();
        log::debug!("Task {} status toggled (completed={})", id, task.completed);
        Ok(())
    }

    /// Toggle pinned/unpinned by id
    pub fn toggle_pinned(tasks: &mut [Task], id: &Uuid) -> ApplicationResult<()> {
        let task: &mut Task = Self::find_task_mut(tasks, id)?;
        task.toggle_pinned();
        log::debug!("Task {} pin toggled (pinned={})", id, task.pinned);
        Ok(())
    }

    /// Clear tasks that are being filtered by current sidebar tab or folder selection.
    /// Returns the number of removed tasks.
    pub fn clear_tasks(tasks: &mut Vec<Task>, tab: SidebarTab, folder_id: Option<Uuid>) -> usize {
        let initial_len: usize = tasks.len();
        let today = Local::now().date_naive();

        tasks.retain(|t| {
            if let Some(target_folder_id) = folder_id {
                return t.folder_id != Some(target_folder_id);
            }

            match tab {
                SidebarTab::Inbox => false,
                SidebarTab::Active => t.completed,
                SidebarTab::Completed => !t.completed,
                SidebarTab::HighPriority => t.priority != Priority::High,
                SidebarTab::Today => !t.is_due_today(&today),
            }
        });

        let removed: usize = initial_len - tasks.len();
        log::info!(
            "Massive clear performed. Tab: {:?}, Folder: {:?}, Removed: {} tasks",
            tab,
            folder_id,
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
            log::warn!("Move failed: indices out of bounds");
            return Err(TaskError::TaskNotFound.into());
        }

        if a == b {
            return Ok(());
        }

        if tasks[a].pinned || tasks[b].pinned {
            log::warn!("Move forbidden: cannot move pinned tasks");
            return Err(TaskError::MoveForbidden.into());
        }

        if tasks[a].priority != tasks[b].priority {
            log::warn!("Move forbidden: tasks have different priorities");
            return Err(TaskError::MoveForbidden.into());
        }

        log::debug!("Swapping tasks at indices {} and {}", a, b);
        tasks.swap(a, b);
        Ok(())
    }

    /// Private method to return mutable task by id
    fn find_task_mut<'a>(tasks: &'a mut [Task], id: &Uuid) -> ApplicationResult<&'a mut Task> {
        tasks.iter_mut().find(|t| t.id == *id).ok_or_else(|| {
            log::error!("Task with ID {} not found", id);
            TaskError::TaskNotFound.into()
        })
    }

    /// Private method to return task index by id
    fn find_index(tasks: &[Task], id: &Uuid) -> ApplicationResult<usize> {
        tasks
            .iter()
            .position(|t| t.id == *id)
            .ok_or(TaskError::TaskNotFound.into())
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

        let result: ApplicationResult<OperationResult> =
            TaskService::append_task(&mut tasks, task_to_add);
        let added_task: &Task = &tasks[0];

        assert!(result.is_ok());
        assert_eq!(tasks.len(), 1);
        assert_eq!(result.unwrap().entity_title(), added_task.title);
    }

    #[test]
    fn should_fail_append_task_service_on_empty_title() {
        let mut tasks: Vec<Task> = Vec::new();
        let task_to_add: Task = Task::new("");
        let result: ApplicationResult<OperationResult> =
            TaskService::append_task(&mut tasks, task_to_add);

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
            folder_id: None,
        };

        let result = TaskService::update_task(&mut tasks, &id, editor);
        assert!(result.is_ok());

        let (old, new) = result.unwrap().unwrap_task_updated();

        assert_eq!(old.title, "Old title");
        assert_eq!(new.title, "New Title");
        assert_eq!(new.priority, Priority::High);
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
            folder_id: None,
        };

        let result: ApplicationResult<OperationResult> =
            TaskService::update_task(&mut tasks, &id, editor);

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
            folder_id: None,
        };

        let result: ApplicationResult<OperationResult> =
            TaskService::update_task(&mut tasks, &fake_id, editor);

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

        let result: ApplicationResult<OperationResult> =
            TaskService::remove_task(&mut tasks, &id_to_remove);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().entity_title(), "Task 1");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task 2");
    }

    #[test]
    fn should_fail_remove_task_service_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Task")];
        let fake_id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<OperationResult> =
            TaskService::remove_task(&mut tasks, &fake_id);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::TaskNotFound))
        ));
    }

    #[test]
    fn should_toggle_completed_and_pinned_task_service() {
        let mut tasks: Vec<Task> = vec![Task::new("Toggle Me")];
        let id: Uuid = tasks[0].id;
        assert!(!tasks[0].completed);
        assert!(!tasks[0].pinned);

        TaskService::toggle_completed(&mut tasks, &id).unwrap();
        assert!(tasks[0].completed);

        TaskService::toggle_pinned(&mut tasks, &id).unwrap();
        assert!(tasks[0].pinned);

        TaskService::toggle_completed(&mut tasks, &id).unwrap();
        assert!(!tasks[0].completed);

        TaskService::toggle_pinned(&mut tasks, &id).unwrap();
        assert!(!tasks[0].pinned);
    }

    #[test]
    fn should_fail_task_toggling_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Toggle Me")];
        let fake_id: Uuid = Uuid::new_v4();
        assert!(!tasks[0].completed);
        assert!(!tasks[0].pinned);

        let result: ApplicationResult<()> = TaskService::toggle_completed(&mut tasks, &fake_id);

        assert!(!tasks[0].completed);
        assert_eq!(result, Err(ApplicationError::Task(TaskError::TaskNotFound)));

        let result: ApplicationResult<()> = TaskService::toggle_pinned(&mut tasks, &fake_id);
        assert!(!tasks[0].pinned);
        assert_eq!(result, Err(ApplicationError::Task(TaskError::TaskNotFound)))
    }

    #[test]
    fn should_fail_toggle_task_service_with_wrong_id() {
        let mut tasks: Vec<Task> = vec![Task::new("Toggle Me")];
        let fake_id: Uuid = Uuid::new_v4();
        assert!(!tasks[0].completed);

        let result: ApplicationResult<()> = TaskService::toggle_completed(&mut tasks, &fake_id);

        assert!(!tasks[0].completed);
        assert_eq!(result, Err(ApplicationError::Task(TaskError::TaskNotFound)))
    }

    #[test]
    fn should_clear_tasks_service_completed() {
        let mut tasks: Vec<Task> = vec![Task::new("Active"), Task::new("Done")];
        tasks[1].completed = true;

        let removed_count: usize =
            TaskService::clear_tasks(&mut tasks, SidebarTab::Completed, None);

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
    fn should_not_move_pinned_task() {
        let mut tasks: Vec<Task> = vec![
            Task::new("High Task").with_priority(Priority::High),
            Task::new("Medium Task").with_priority(Priority::Medium),
        ];

        let mut pinned = Task::new("Pinned").with_priority(Priority::High);
        pinned.toggle_pinned();
        tasks.push(pinned);

        let result: ApplicationResult<()> = TaskService::move_tasks(&mut tasks, 1, 2);
        assert_eq!(
            result,
            Err(ApplicationError::Task(TaskError::MoveForbidden))
        );
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
