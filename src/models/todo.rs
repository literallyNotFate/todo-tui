use super::Filter;
use crate::{config::UIConfig, core::Selectable, theme::ThemePalette};
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

    /// Update task using editor model from form
    pub fn update_from_editor(&mut self, editor: TodoEditor) {
        self.title = editor.title;
        self.description = editor.description;
        self.priority = *editor.priority;
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
        let fmt = |dt: DateTime<Utc>| dt.with_timezone(&Local).format(&full_fmt).to_string();

        Self {
            id_short: todo.id.to_string()[..8].to_string(),
            title: todo.title.clone(),
            completed: todo.completed,
            description: todo.description.clone(),
            created_at: fmt(todo.created_at),
            updated_at: fmt(todo.updated_at),
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
pub struct TodoEditor {
    pub title: String,
    pub description: String,
    pub priority: Selectable<Priority>,
}

impl TodoEditor {
    pub fn from_todo(todo: &Todo) -> Self {
        Self {
            title: todo.title.clone(),
            description: todo.description.clone(),
            priority: Selectable::new(todo.priority),
        }
    }

    pub fn save(self, todo: &mut Todo) {
        todo.title = self.title;
        todo.description = self.description;
        todo.priority = *self.priority;
    }
}

/// Unit-tests for todo model + priority (basic methods)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ThemeName, ThemePalette};
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
    fn should_update_todo_fields_using_editor() {
        let mut todo: Todo = Todo::new("Test", "Test", None);
        todo.completed = true;

        assert_eq!(todo.title, "Test");
        assert_eq!(todo.description, "Test");
        assert_eq!(todo.priority, Priority::Low);
        assert!(todo.completed);

        let editor: TodoEditor = TodoEditor {
            title: "Edit".into(),
            description: "Edit".into(),
            priority: Selectable::new(Priority::High),
        };
        todo.update_from_editor(editor);

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
