use super::Todo;
use std::cmp::Ordering;

/// Sorting tasks by parameter with orders
#[derive(Default, Debug, Copy, Clone)]
pub struct Sort {
    pub parameter: SortBy,
    pub order: SortOrder,
}

/// Sorting by specific parameter-field
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    #[default]
    Priority,
    Title,
    CreatedAt,
}

/// Sort order (descending by default)
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum SortOrder {
    #[default]
    Desc,
    Asc,
}

impl Sort {
    pub fn new(parameter: SortBy, order: SortOrder) -> Self {
        Self { parameter, order }
    }

    pub fn compare(&self, a: &Todo, b: &Todo) -> Ordering {
        let cmp: Ordering = match self.parameter {
            SortBy::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
            SortBy::Priority => {
                let result: Ordering = (a.priority as u8).cmp(&(b.priority as u8));
                if result == Ordering::Equal {
                    a.created_at.cmp(&b.created_at)
                } else {
                    result
                }
            }
            SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
        };

        match self.order {
            SortOrder::Asc => cmp,
            SortOrder::Desc => cmp.reverse(),
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.parameter.label(), self.order.icon())
    }
}

impl SortBy {
    pub fn next(&self) -> Self {
        match self {
            Self::Priority => Self::Title,
            Self::Title => Self::CreatedAt,
            Self::CreatedAt => Self::Priority,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Priority => "Priority",
            Self::CreatedAt => "Created",
        }
    }
}

impl SortOrder {
    pub fn next(&self) -> Self {
        match self {
            Self::Desc => Self::Asc,
            Self::Asc => Self::Desc,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SortOrder::Asc => "▲",
            SortOrder::Desc => "▼",
        }
    }
}

/// Unit-tests for sort
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn todo(title: &str, priority: Priority, seconds_ago: i64) -> Todo {
        Todo {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: String::new(),
            priority,
            completed: false,
            created_at: Utc::now() - Duration::seconds(seconds_ago),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn should_cycle_through_sort_by_and_order() {
        let mut by: SortBy = SortBy::Priority;
        by = by.next();
        assert_eq!(by, SortBy::Title);
        by = by.next();
        assert_eq!(by, SortBy::CreatedAt);
        by = by.next();
        assert_eq!(by, SortBy::Priority);

        let mut order: SortOrder = SortOrder::Desc;
        order = order.next();
        assert_eq!(order, SortOrder::Asc);
        order = order.next();
        assert_eq!(order, SortOrder::Desc);
    }

    #[test]
    fn should_compare_priority_desc() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Desc);
        let high = todo("High", Priority::High, 0);
        let low = todo("Low", Priority::Low, 0);

        assert_eq!(sort.compare(&high, &low), Ordering::Less);
    }

    #[test]
    fn should_compare_title_case_insensitive() {
        let sort = Sort::new(SortBy::Title, SortOrder::Asc);
        let a = todo("apple", Priority::Low, 0);
        let b = todo("Banana", Priority::Low, 0);

        assert_eq!(sort.compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn should_stabilize_priority_sorting() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Asc);

        let old_task = todo("Old", Priority::Medium, 10);
        let new_task = todo("New", Priority::Medium, 0);

        assert_eq!(sort.compare(&old_task, &new_task), Ordering::Less);
    }
}
