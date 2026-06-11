use crate::{config::UIConfig, core::Selectable, models::Filter, theme::ThemePalette};
use chrono::{DateTime, Local, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Main task entity with unique id
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: Priority,
    pub folder_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip)]
    pub title_lower: String,
}

impl Task {
    /// Create new task object
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        let title: String = title.into();
        let title_lower: String = title.to_lowercase();

        Self {
            id: Uuid::new_v4(),
            title,
            description: String::new(),
            completed: false,
            priority: Priority::default(),
            folder_id: None,
            created_at: now,
            updated_at: now,
            title_lower,
        }
    }

    /// Add description to task
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add priority to task
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Add folder to task
    pub fn with_folder(mut self, folder_id: Uuid) -> Self {
        self.folder_id = Some(folder_id);
        self
    }

    /// Update task using editor model from form
    pub fn update_from_editor(&mut self, editor: TaskEditor) {
        self.title = editor.title;
        self.description = editor.description;
        self.priority = *editor.priority;
        self.folder_id = editor.folder_id;
        self.title_lower = self.title.to_lowercase();
        self.updated_at = Utc::now();
    }

    /// Toggle completed on Enter
    pub fn toggle_completed(&mut self) {
        self.completed = !self.completed;
    }

    /// Return created at string for table
    pub fn time_ago(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        let diff: TimeDelta = now.signed_duration_since(self.created_at);

        if diff.num_seconds() <= 0 {
            return "just now".into();
        }

        let (count, unit) = match diff {
            d if d.num_days() >= 365 => (d.num_days() / 365, "year"),
            d if d.num_days() >= 30 => (d.num_days() / 30, "month"),
            d if d.num_days() >= 7 => (d.num_days() / 7, "week"),
            d if d.num_days() >= 1 => (d.num_days(), "day"),
            d if d.num_hours() >= 1 => (d.num_hours(), "hour"),
            d if d.num_minutes() >= 1 => (d.num_minutes(), "minute"),
            _ => return "just now".into(),
        };

        let s = if count == 1 { "" } else { "s" };
        format!("{} {}{} ago", count, unit, s)
    }

    /// Checks whether specific task matches current filter conditions (for filter)
    pub fn matches_filter(&self, filter: &Filter, today: &chrono::NaiveDate) -> bool {
        match filter {
            Filter::All => true,
            Filter::Active => !self.completed,
            Filter::Completed => self.completed,
            Filter::HighPriority => self.priority == Priority::High,
            Filter::Today => self.created_at.with_timezone(&Local).date_naive() == *today,
            Filter::InFolder(folder_id) => self.folder_id == Some(*folder_id),
        }
    }
}

/// Implementing hash for task excluding time fields and dynamically changeable folders for consistency
impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.description.hash(state);
        self.completed.hash(state);
        self.priority.hash(state);
        self.folder_id.hash(state);
    }
}

/// Task details to be shown
#[derive(Debug, Clone)]
pub struct TaskDetails {
    pub id_short: String,
    pub title: String,
    pub completed: bool,
    pub description: String,
    pub folder_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

/// Implementation of initializing task details from task with UIConfig (date format)
impl TaskDetails {
    pub fn from(task: &Task, config: &UIConfig) -> Self {
        let time_fmt: &str = if config.use_24h { "%H:%M" } else { "%I:%M %p" };
        let full_fmt: String = format!("{}, {}", config.date_format, time_fmt);
        let fmt = |dt: DateTime<Utc>| dt.with_timezone(&Local).format(&full_fmt).to_string();

        Self {
            id_short: task.id.to_string()[..8].to_string(),
            title: task.title.clone(),
            completed: task.completed,
            description: task.description.clone(),
            folder_id: task.folder_id,
            created_at: fmt(task.created_at),
            updated_at: fmt(task.updated_at),
        }
    }
}

/// Task priority
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Default,
    Hash,
    Eq,
    PartialEq,
    Copy,
    PartialOrd,
    Ord,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "PascalCase")]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn palette(&self, palette: &ThemePalette) -> ratatui::style::Color {
        match self {
            Priority::High => palette.error,
            Priority::Medium => palette.warning,
            Priority::Low => palette.success,
        }
    }
}

/// Model for updating task
pub struct TaskEditor {
    pub title: String,
    pub description: String,
    pub priority: Selectable<Priority>,
    pub folder_id: Option<Uuid>,
}

impl TaskEditor {
    pub fn from_task(task: &Task) -> Self {
        Self {
            title: task.title.clone(),
            description: task.description.clone(),
            priority: Selectable::new(task.priority),
            folder_id: task.folder_id,
        }
    }

    pub fn save(self, task: &mut Task) {
        task.title = self.title;
        task.description = self.description;
        task.priority = *self.priority;
        task.folder_id = self.folder_id;
    }
}

/// Unit-tests for task model + priority (basic methods)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeName, ThemePalette};
    use chrono::{Days, Duration, Months, NaiveDate};

    #[test]
    fn should_create_task_item() {
        let task: Task = Task::new("Test");

        assert_eq!(task.title, "Test");
        assert_eq!(task.description, "");
        assert_eq!(task.priority, Priority::Low);
        assert_eq!(task.folder_id, None);
        assert!(!task.completed);
    }

    #[test]
    fn should_create_task_linked_to_folder() {
        let folder_id = Uuid::new_v4();
        let task = Task::new("Folder Task")
            .with_description("Inside folder")
            .with_priority(Priority::Medium)
            .with_folder(folder_id);

        assert_eq!(task.folder_id, Some(folder_id));
        assert_eq!(task.priority, Priority::Medium);
        assert_eq!(task.description, "Inside folder");
    }

    #[test]
    fn should_generate_unique_id_for_tasks() {
        let title: &str = "Test Task";

        let task1 = Task::new(title);
        let task2 = Task::new(title);

        assert!(!task1.id.is_nil(), "UUID should not be nil");
        assert_ne!(task1.id, task2.id, "each task must have a unique UUID");

        assert_eq!(
            task1.id.get_version(),
            Some(uuid::Version::Random),
            "UUID should be version 4"
        );
    }

    #[test]
    fn should_update_task_fields_including_folder_using_editor() {
        let mut task: Task = Task::new("Test");
        let new_folder_id = Uuid::new_v4();

        let editor: TaskEditor = TaskEditor {
            title: "Edit".into(),
            description: "Edit".into(),
            priority: Selectable::new(Priority::High),
            folder_id: Some(new_folder_id),
        };
        task.update_from_editor(editor);

        assert_eq!(task.title, "Edit");
        assert_eq!(task.description, "Edit");
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.folder_id, Some(new_folder_id));
    }

    #[test]
    fn should_toggle_completed() {
        let mut task: Task = Task::new("Test").with_priority(Priority::Medium);
        assert_eq!(task.priority, Priority::Medium);

        task.toggle_completed();
        assert!(task.completed);

        task.toggle_completed();
        assert!(!task.completed);
    }

    #[test]
    fn should_return_created_at_string() {
        let mut task: Task = Task::new("Test");
        assert_eq!(task.time_ago(), "just now".to_string());

        task.created_at = Utc::now().checked_sub_days(Days::new(2)).unwrap();
        assert_eq!(task.time_ago(), "2 days ago".to_string());

        task.created_at = Utc::now().checked_sub_days(Days::new(7)).unwrap();
        assert_eq!(task.time_ago(), "1 week ago".to_string());

        task.created_at = Utc::now().checked_sub_months(Months::new(3)).unwrap();
        assert_eq!(task.time_ago(), "3 months ago");

        task.created_at = Utc::now().checked_sub_days(Days::new(365)).unwrap();
        assert_eq!(task.time_ago(), "1 year ago");
    }

    #[test]
    fn should_test_task_filter_matching() {
        let today: NaiveDate = Local::now().date_naive();
        let mut task = Task::new("Test")
            .with_description("Desc")
            .with_priority(Priority::High);

        assert!(task.matches_filter(&Filter::HighPriority, &today));

        assert!(task.matches_filter(&Filter::Active, &today));
        task.completed = true;
        assert!(task.matches_filter(&Filter::Completed, &today));
        assert!(!task.matches_filter(&Filter::Active, &today));

        assert!(task.matches_filter(&Filter::Today, &today));

        task.created_at = Utc::now() - Duration::days(1);
        assert!(!task.matches_filter(&Filter::Today, &today));
    }

    #[test]
    fn should_create_task_details_from_task() {
        let task = Task::new("Task 1").with_description("Desc 1");
        let config = UIConfig::default();
        let details = TaskDetails::from(&task, &config);

        assert_eq!(details.title, "Task 1");
        assert_eq!(details.description, "Desc 1");
        assert_eq!(details.id_short.len(), 8);
        assert_eq!(details.folder_id, None);
        assert!(details.updated_at.contains(":"));
    }

    #[test]
    fn should_return_string_from_enum() {
        assert_eq!(Priority::Low.to_string(), "Low");
        assert_eq!(Priority::Medium.to_string(), "Medium");
        assert_eq!(Priority::High.to_string(), "High");
    }

    #[test]
    fn should_compare_priorities() {
        assert!(Priority::High > Priority::Low);
        assert!(Priority::Medium > Priority::Low);
        assert!(Priority::High > Priority::Medium);
    }

    #[test]
    fn should_return_right_color_of_priority_with_theme() {
        let palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut priority: Priority = Priority::Low;
        assert_eq!(priority.palette(&palette), palette.success);

        priority = Priority::Medium;
        assert_eq!(priority.palette(&palette), palette.warning);

        priority = Priority::High;
        assert_eq!(priority.palette(&palette), palette.error);
    }
}
