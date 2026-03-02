use super::{Filter, Priority};
use crate::config::UIConfig;
use chrono::{DateTime, Local, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Main todo entity with unique id
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: Priority,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip)]
    pub title_lower: String,
}

impl Todo {
    /// Create new todo object
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Option<Priority>,
    ) -> Self {
        let now = Utc::now();
        let title: String = title.into();
        let title_lower = title.to_lowercase();

        Self {
            id: Uuid::new_v4(),
            title,
            description: description.into(),
            completed: false,
            priority: priority.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            title_lower,
        }
    }

    /// Creating new todo based on existing id (for update)
    pub fn from_id(
        id: Uuid,
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Option<Priority>,
    ) -> Self {
        let mut todo = Self::new(title, description, priority);
        todo.id = id;
        todo
    }

    /// Update todo using other todo
    pub fn update(&mut self, other: Todo) {
        self.title = other.title;
        self.description = other.description;
        self.priority = other.priority;
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
        let time_passed: TimeDelta = now.signed_duration_since(self.created_at);

        let minutes: i64 = time_passed.num_minutes();
        let hours: i64 = time_passed.num_hours();
        let days: i64 = time_passed.num_days();

        let format_unit = |count: i64, unit: &str| -> String {
            if count == 1 {
                format!("{} {} ago", count, unit)
            } else {
                format!("{} {}s ago", count, unit)
            }
        };

        if days >= 365 {
            format_unit(days / 365, "year")
        } else if days >= 30 {
            format_unit(days / 30, "month")
        } else if days >= 7 {
            format_unit(days / 7, "week")
        } else if days > 0 {
            format_unit(days, "day")
        } else if hours > 0 {
            format_unit(hours, "hour")
        } else if minutes > 0 {
            format_unit(minutes, "minute")
        } else {
            "just now".to_string()
        }
    }

    /// Checks whether specific todo matches current filter conditions (for filter)
    pub fn matches_filter(&self, filter: &Filter, today: &NaiveDate) -> bool {
        match filter {
            Filter::All => true,
            Filter::Active => !self.completed,
            Filter::Completed => self.completed,
            Filter::HighPriority => self.priority == Priority::High,
            Filter::Today => self.created_at.with_timezone(&Local).date_naive() == *today,
        }
    }
}

/// Implementing hash for todo excluding time fields (created_at/updated_at) for performance
impl Hash for Todo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.description.hash(state);
        self.completed.hash(state);
        self.priority.hash(state);
    }
}

/// Todo details to be shown
#[derive(Debug, Clone)]
pub struct TodoDetails {
    pub id_short: String,
    pub title: String,
    pub completed: bool,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Implementation of initializing todo details from todo with UIConfig (date format)
impl TodoDetails {
    pub fn from(todo: &Todo, config: &UIConfig) -> Self {
        let time_fmt: &str = if config.use_24h { "%H:%M" } else { "%I:%M %p" };
        let full_fmt: String = format!("{}, {}", config.date_format, time_fmt);

        Self {
            title: todo.title.clone(),
            description: todo.description.clone(),
            created_at: todo
                .created_at
                .with_timezone(&Local)
                .format(&full_fmt)
                .to_string(),

            updated_at: todo
                .updated_at
                .with_timezone(&Local)
                .format(&full_fmt)
                .to_string(),

            id_short: todo.id.to_string()[..8].to_string(),
            completed: todo.completed,
        }
    }
}

/// Unit-tests for todo model (basic methods)
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Days, Duration, Months};

    #[test]
    fn should_create_todo_item() {
        let todo: Todo = Todo::new("Test", "Test", None);

        assert_eq!(todo.title, "Test");
        assert_eq!(todo.description, "Test");
        assert_eq!(todo.priority, Priority::Low);
        assert!(!todo.completed);
    }

    #[test]
    fn should_generate_unique_id_for_todos() {
        let title: &str = "Test Task";
        let desc: &str = "Description";

        let todo1 = Todo::new(title, desc, None);
        let todo2 = Todo::new(title, desc, None);

        assert!(!todo1.id.is_nil(), "UUID should not be nil");
        assert_ne!(todo1.id, todo2.id, "each todo must have a unique UUID");

        assert_eq!(
            todo1.id.get_version(),
            Some(uuid::Version::Random),
            "UUID should be version 4"
        );
    }

    #[test]
    fn should_return_todo_with_id() {
        let original_id: Uuid = Uuid::new_v4();
        let task: Todo =
            Todo::from_id(original_id, "Original", "Description", Some(Priority::High));

        assert_eq!(task.id, original_id);
        assert_eq!(task.title, "Original");
        assert!(!task.completed);
    }

    #[test]
    fn should_update_todo_fields() {
        let mut todo: Todo = Todo::new("Test", "Test", None);
        todo.completed = true;

        assert_eq!(todo.title, "Test");
        assert_eq!(todo.description, "Test");
        assert_eq!(todo.priority, Priority::Low);
        assert!(todo.completed);

        let new: Todo = Todo::new("Edit", "Edit", Some(Priority::High));
        todo.update(new);

        assert_eq!(todo.title, "Edit");
        assert_eq!(todo.description, "Edit");
        assert_eq!(todo.priority, Priority::High);
        assert!(todo.completed);
    }

    #[test]
    fn should_toggle_completed() {
        let mut todo: Todo = Todo::new("Test", "Test", Some(Priority::Medium));
        assert_eq!(todo.priority, Priority::Medium);

        todo.toggle_completed();
        assert!(todo.completed);

        todo.toggle_completed();
        assert!(!todo.completed);
    }

    #[test]
    fn should_return_created_at_string() {
        let mut todo: Todo = Todo::new("Test", "Test", None);
        assert_eq!(todo.time_ago(), "just now".to_string());

        todo.created_at = Utc::now().checked_sub_days(Days::new(2)).unwrap();
        assert_eq!(todo.time_ago(), "2 days ago".to_string());

        todo.created_at = Utc::now().checked_sub_days(Days::new(7)).unwrap();
        assert_eq!(todo.time_ago(), "1 week ago".to_string());

        todo.created_at = Utc::now().checked_sub_months(Months::new(3)).unwrap();
        assert_eq!(todo.time_ago(), "3 months ago");

        todo.created_at = Utc::now().checked_sub_days(Days::new(365)).unwrap();
        assert_eq!(todo.time_ago(), "1 year ago");
    }

    #[test]
    fn should_test_todo_filter_matching() {
        let today: NaiveDate = Local::now().date_naive();
        let mut todo = Todo::new("Test", "Desc", Some(Priority::High));

        assert!(todo.matches_filter(&Filter::HighPriority, &today));

        assert!(todo.matches_filter(&Filter::Active, &today));
        todo.completed = true;
        assert!(todo.matches_filter(&Filter::Completed, &today));
        assert!(!todo.matches_filter(&Filter::Active, &today));

        assert!(todo.matches_filter(&Filter::Today, &today));

        todo.created_at = Utc::now() - Duration::days(1);
        assert!(!todo.matches_filter(&Filter::Today, &today));
    }

    #[test]
    fn should_create_todo_details_from_todo() {
        let todo = Todo::new("Task 1", "Desc 1", None);
        let config = UIConfig::default();
        let details = TodoDetails::from(&todo, &config);

        assert_eq!(details.title, "Task 1");
        assert_eq!(details.description, "Desc 1");
        assert_eq!(details.id_short.len(), 8);
        assert!(details.updated_at.contains(":"));
    }
}
