use crate::{
    config::BehaviorConfig,
    models::{Priority, Task},
    state::SidebarTab,
};
use chrono::{Local, NaiveDate};
use uuid::Uuid;

/// Struct to filter tasks for rendering (contains search query, filter, folder ID)
pub struct TaskFilter {
    pub tab: SidebarTab,
    pub folder_id: Option<Uuid>,
    pub search_query: String,
    pub raw_search_query: String,
}

impl TaskFilter {
    pub fn new(tab: SidebarTab, folder_id: Option<Uuid>, query: &str) -> Self {
        Self {
            tab,
            folder_id,
            search_query: query.to_lowercase().trim().to_string(),
            raw_search_query: query.trim().to_string(),
        }
    }

    /// Checks whether specific task matches current filter conditions (for filter)
    pub fn matches(&self, task: &Task, today: &NaiveDate, case_insensitive: bool) -> bool {
        if !self.raw_search_query.is_empty() {
            let title: &String = if case_insensitive {
                &task.title_lower
            } else {
                &task.title
            };

            let query: &String = if case_insensitive {
                &self.search_query
            } else {
                &self.raw_search_query
            };

            if !title.contains(query) {
                return false;
            }
        }

        if let Some(f_id) = self.folder_id {
            if task.folder_id != Some(f_id) {
                return false;
            }
        } else if self.tab == SidebarTab::Inbox {
            if task.folder_id.is_some() {
                return false;
            }
        }

        match self.tab {
            SidebarTab::Inbox => true,
            SidebarTab::Active => !task.completed,
            SidebarTab::Completed => task.completed,
            SidebarTab::HighPriority => task.priority == Priority::High,
            SidebarTab::Today => task.is_due_today(today),
        }
    }

    /// Applies filter to tasks vector
    pub fn apply<'a>(&self, tasks: &'a [Task], config: &BehaviorConfig) -> Vec<&'a Task> {
        let today: NaiveDate = Local::now().date_naive();
        tasks
            .iter()
            .filter(|task| self.matches(task, &today, config.case_insensitive_search))
            .collect()
    }
}

/// Unit-tests for task filter
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_task(
        title: &str,
        completed: bool,
        priority: Priority,
        folder: Option<Uuid>,
    ) -> Task {
        let mut t = Task::new(title).with_priority(priority);
        t.completed = completed;
        if let Some(f_id) = folder {
            t = t.with_folder(f_id);
        }
        t
    }

    #[test]
    fn should_handle_filtering_by_sidebar_tabs() {
        let tasks = vec![
            create_test_task("Active Task", false, Priority::Medium, None),
            create_test_task("Completed Task", true, Priority::Medium, None),
            create_test_task("High Priority Task", false, Priority::High, None),
        ];
        let config = BehaviorConfig::default();

        let filter_all = TaskFilter::new(SidebarTab::Inbox, None, "");
        assert_eq!(filter_all.apply(&tasks, &config).len(), 3);

        let filter_active = TaskFilter::new(SidebarTab::Active, None, "");
        let active_res = filter_active.apply(&tasks, &config);
        assert_eq!(active_res.len(), 2);
        assert!(active_res.iter().all(|t| !t.completed));

        let filter_completed = TaskFilter::new(SidebarTab::Completed, None, "");
        let completed_res = filter_completed.apply(&tasks, &config);
        assert_eq!(completed_res.len(), 1);
        assert_eq!(completed_res[0].title, "Completed Task");

        let filter_high = TaskFilter::new(SidebarTab::HighPriority, None, "");
        let high_res = filter_high.apply(&tasks, &config);
        assert_eq!(high_res.len(), 1);
        assert_eq!(high_res[0].title, "High Priority Task");
    }

    #[test]
    fn should_handle_filtering_by_folder() {
        let folder_a = Uuid::new_v4();
        let folder_b = Uuid::new_v4();
        let config = BehaviorConfig::default();

        let tasks = vec![
            create_test_task("Task in A 1", false, Priority::Medium, Some(folder_a)),
            create_test_task("Task in A 2", true, Priority::Medium, Some(folder_a)),
            create_test_task("Task in B", false, Priority::Medium, Some(folder_b)),
            create_test_task("Global Task", false, Priority::Medium, None),
        ];

        let filter_folder_a = TaskFilter::new(SidebarTab::Inbox, Some(folder_a), "");
        assert_eq!(filter_folder_a.apply(&tasks, &config).len(), 2);

        let filter_folder_b = TaskFilter::new(SidebarTab::Inbox, Some(folder_b), "");
        let res_b = filter_folder_b.apply(&tasks, &config);
        assert_eq!(res_b.len(), 1);
        assert_eq!(res_b[0].title, "Task in B");
    }

    #[test]
    fn should_test_filter_search_query_normalization() {
        let tasks = vec![
            create_test_task("Buy some milk", false, Priority::Medium, None),
            create_test_task("Clean the room", false, Priority::Medium, None),
        ];
        let config = BehaviorConfig::default();

        let filter = TaskFilter::new(SidebarTab::Inbox, None, "  mILk  ");
        let res = filter.apply(&tasks, &config);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "Buy some milk");
    }

    #[test]
    fn should_handle_filter_combining() {
        let folder_work = Uuid::new_v4();
        let config = BehaviorConfig::default();

        let tasks = vec![
            create_test_task("Fix bug", false, Priority::High, Some(folder_work)),
            create_test_task("Write docs", true, Priority::High, Some(folder_work)),
            create_test_task("Buy food", false, Priority::High, None),
            create_test_task("Fix chair", false, Priority::Low, Some(folder_work)),
        ];

        let filter = TaskFilter::new(SidebarTab::HighPriority, Some(folder_work), "Fix");
        let res = filter.apply(&tasks, &config);

        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "Fix bug");
    }

    #[test]
    fn should_handle_case_sensitivity_in_search() {
        let tasks = vec![create_test_task("Milk", false, Priority::Medium, None)];
        let mut config = BehaviorConfig::default();

        config.case_insensitive_search = true;
        let filter_ci = TaskFilter::new(SidebarTab::Inbox, None, "milk");
        assert_eq!(filter_ci.apply(&tasks, &config).len(), 1, "Should find 1");

        config.case_insensitive_search = false;
        let filter_cs = TaskFilter::new(SidebarTab::Inbox, None, "milk");
        assert_eq!(
            filter_cs.apply(&tasks, &config).len(),
            0,
            "Should not find anything"
        );

        let filter_cs_match = TaskFilter::new(SidebarTab::Inbox, None, "Milk");
        assert_eq!(
            filter_cs_match.apply(&tasks, &config).len(),
            1,
            "Should find 1"
        );
    }

    #[test]
    fn should_maintain_query_normalization() {
        let tasks = vec![create_test_task("Buy milk", false, Priority::Medium, None)];
        let config = BehaviorConfig::default();

        let filter = TaskFilter::new(SidebarTab::Inbox, None, "  MILK  ");
        let res = filter.apply(&tasks, &config);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].title, "Buy milk");
    }
}
