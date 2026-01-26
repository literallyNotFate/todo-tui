use super::Priority;
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Hash)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
}

impl Todo {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Option<Priority>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            completed: false,
            priority: priority.unwrap_or_default(),
            created_at: Utc::now(),
        }
    }

    pub fn toggle_completed(&mut self) {
        self.completed = !self.completed;
    }

    pub fn time_ago(&self) -> String {
        let now: DateTime<Utc> = Utc::now();
        let time_passed: TimeDelta = now.signed_duration_since(self.created_at);
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
        } else if time_passed.num_weeks() > 0 {
            format_unit(time_passed.num_weeks(), "week")
        } else if days > 0 {
            format_unit(days, "day")
        } else if time_passed.num_hours() > 0 {
            format_unit(time_passed.num_hours(), "hour")
        } else if time_passed.num_minutes() > 0 {
            format_unit(time_passed.num_minutes(), "minute")
        } else if time_passed.num_seconds() > 0 {
            format_unit(time_passed.num_seconds(), "second")
        } else {
            "just now".to_string()
        }
    }
}

// Unit-tests for todo model (basic methods)
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Days, Months};

    #[test]
    fn should_create_todo_item() {
        let todo: Todo = Todo::new("Test", "Test", None);

        assert_eq!(todo.title, "Test");
        assert_eq!(todo.title, "Test");
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
}
