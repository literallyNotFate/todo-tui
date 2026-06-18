use crate::{
    config::UIConfig,
    core::{Selectable, TaskError},
    state::ApplicationResult,
    theme::ThemePalette,
};
use chrono::{DateTime, Local, NaiveDate, TimeDelta, Utc};
use ratatui::style::Color;
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

    pub pinned: bool,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip)]
    pub title_lower: String,
    #[serde(skip)]
    pub id_formatted: String,
}

/// Task details to be shown
#[derive(Debug, Clone)]
pub struct TaskDetails {
    pub id_display: String,
    pub title: String,
    pub completed: bool,
    pub description: String,
    pub folder_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
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

/// Model for updating task
#[derive(Clone)]
pub struct TaskEditor {
    pub title: String,
    pub description: String,
    pub priority: Selectable<Priority>,
    pub folder_id: Option<Uuid>,
}

impl Task {
    /// Create new task object
    pub fn new(title: impl Into<String>) -> Self {
        let now: DateTime<Utc> = Utc::now();
        let id: Uuid = Uuid::new_v4();
        let title: String = title.into();
        let title_lower: String = title.to_lowercase();
        let id_formatted: String = format!("#{}", &id.to_string()[..8]);

        Self {
            id,
            title,
            description: String::new(),
            completed: false,
            priority: Priority::default(),
            folder_id: None,
            pinned: false,
            created_at: now,
            updated_at: now,
            title_lower,
            id_formatted,
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
    pub fn update_from(&mut self, editor: TaskEditor) {
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

    /// Toggle pin task
    pub fn toggle_pinned(&mut self) {
        self.pinned = !self.pinned;
    }

    /// Validate task
    pub fn validate(&self) -> ApplicationResult<()> {
        if self.title.trim().is_empty() {
            return Err(TaskError::EmptyTitle.into());
        }

        Ok(())
    }

    /// Return created at string for table
    pub fn time_ago(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        let diff: TimeDelta = now.signed_duration_since(self.created_at);
        let secs: i64 = diff.num_seconds();

        if secs <= 0 {
            return "just now".to_string();
        }

        let (count, unit) = match diff {
            d if d.num_days() >= 365 => (d.num_days() / 365, "year"),
            d if d.num_days() >= 30 => (d.num_days() / 30, "month"),
            d if d.num_days() >= 7 => (d.num_days() / 7, "week"),
            d if d.num_days() >= 1 => (d.num_days(), "day"),
            d if d.num_hours() >= 1 => (d.num_hours(), "hour"),
            d if d.num_minutes() >= 1 => (d.num_minutes(), "minute"),
            _ => return "just now".to_string(),
        };

        if count == 1 {
            format!("1 {} ago", unit)
        } else {
            format!("{} {}s ago", count, unit)
        }
    }

    /// Format date
    pub fn format_date(&self, use_24h: bool) -> String {
        let local_dt: DateTime<Local> = self.created_at.with_timezone(&Local);
        let today: NaiveDate = Local::now().date_naive();

        if self.is_due_today(&today) {
            let format = if use_24h { "%H:%M" } else { "%I:%M %p" };
            local_dt.format(format).to_string()
        } else {
            local_dt.format("%d %b").to_string()
        }
    }

    /// Get display info based on UI config
    pub fn get_display_info<'a>(
        &self,
        config: &'a UIConfig,
        palette: &'a ThemePalette,
    ) -> (&'a str, Color) {
        if self.pinned {
            (config.symbols.pinned.as_str(), palette.warning)
        } else if self.completed {
            (config.symbols.completed.as_str(), palette.success)
        } else {
            (config.symbols.pending.as_str(), palette.muted)
        }
    }

    /// Check whether task is created today
    pub fn is_due_today(&self, today: &NaiveDate) -> bool {
        self.created_at.with_timezone(&Local).date_naive() == *today
    }
}

/// Implementing hash for task excluding time fields and dynamically changeable folders for consistency
impl Hash for Task {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.description.hash(state);
        self.completed.hash(state);
        self.pinned.hash(state);
        self.priority.hash(state);
        self.folder_id.hash(state);
    }
}

impl TaskDetails {
    pub fn from(task: &Task, config: &UIConfig) -> Self {
        let time_fmt: &str = if config.use_24h { "%H:%M" } else { "%I:%M %p" };
        let full_fmt: String = format!("{}, {}", config.date_format, time_fmt);
        let fmt = |dt: DateTime<Utc>| dt.with_timezone(&Local).format(&full_fmt).to_string();

        Self {
            id_display: task.id_formatted.clone(),
            title: task.title.clone(),
            completed: task.completed,
            description: task.description.clone(),
            folder_id: task.folder_id,
            created_at: fmt(task.created_at),
            updated_at: fmt(task.updated_at),
        }
    }
}

impl Priority {
    pub fn palette(&self, palette: &ThemePalette) -> ratatui::style::Color {
        match self {
            Priority::High => palette.error,
            Priority::Medium => palette.warning,
            Priority::Low => palette.success,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        }
    }
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
    use crate::{
        core::ApplicationError,
        theme::{ThemeName, ThemePalette},
    };
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
        task.update_from(editor);

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
    fn should_toggle_pinned() {
        let mut task: Task = Task::new("Test");
        assert!(!task.pinned);

        task.toggle_pinned();
        assert!(task.pinned);

        task.toggle_pinned();
        assert!(!task.pinned);
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
    fn should_test_task_due_today() {
        let today: NaiveDate = Local::now().date_naive();
        let mut task = Task::new("Test")
            .with_description("Desc")
            .with_priority(Priority::High);

        assert!(task.is_due_today(&today));

        task.created_at = Utc::now() - Duration::days(1);
        assert!(!task.is_due_today(&today));
    }

    #[test]
    fn should_properly_validate_task() {
        let task = Task::new("Title").with_description("Desc");
        let result = task.validate();
        assert!(result.is_ok());

        let task = Task::new("").with_description("Desc");
        let result = task.validate();
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Task(TaskError::EmptyTitle))
        ));
    }

    #[test]
    fn should_create_task_details_from_task() {
        let task = Task::new("Task 1").with_description("Desc 1");
        let config = UIConfig::default();
        let details = TaskDetails::from(&task, &config);

        assert_eq!(details.title, "Task 1");
        assert_eq!(details.description, "Desc 1");
        assert_eq!(details.id_display.len(), 9);
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

    #[test]
    fn should_test_task_date_formatting() {
        let now: DateTime<Utc> = Utc::now();
        let task: Task = Task {
            created_at: now,
            ..Default::default()
        };
        assert!(task.format_date(true).contains(':'));

        let past_task: Task = Task {
            created_at: now - Duration::days(2),
            ..Default::default()
        };
        assert!(!past_task.format_date(true).contains(':'));
    }
}
