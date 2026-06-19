use crate::{core::Selectable, models::Task};
use clap::ValueEnum;
use std::cmp::Ordering;
use strum::{Display, EnumIter};

/// Sorting tasks by parameter with orders
#[derive(Default, Debug, Copy, Clone)]
pub struct Sort {
    pub parameter: Selectable<SortBy>,
    pub order: Selectable<SortOrder>,
}

/// Sorting by specific parameter-field
#[derive(Default, Debug, Clone, Copy, PartialEq, EnumIter, Display, ValueEnum)]
#[strum(serialize_all = "title_case")]
pub enum SortBy {
    #[default]
    Priority,
    Title,
    #[strum(to_string = "Created")]
    CreatedAt,
}

/// Sort order (descending by default)
#[derive(Default, Debug, Copy, Clone, PartialEq, EnumIter, Display, ValueEnum)]
pub enum SortOrder {
    #[default]
    #[strum(to_string = "▼")]
    Desc,
    #[strum(to_string = "▲")]
    Asc,
}

impl Sort {
    pub fn new(parameter: SortBy, order: SortOrder) -> Self {
        Self {
            parameter: Selectable::new(parameter),
            order: Selectable::new(order),
        }
    }

    pub fn compare(&self, a: &Task, b: &Task) -> Ordering {
        let pinned_cmp: Ordering = b.pinned.cmp(&a.pinned);
        if pinned_cmp != Ordering::Equal {
            return pinned_cmp;
        }

        let cmp: Ordering = match *self.parameter {
            SortBy::Title => a.title_lower.cmp(&b.title_lower),
            SortBy::Priority => {
                let result: Ordering = a.priority.cmp(&b.priority);
                if result == Ordering::Equal {
                    a.created_at.cmp(&b.created_at)
                } else {
                    result
                }
            }
            SortBy::CreatedAt => a.created_at.cmp(&b.created_at),
        };

        match *self.order {
            SortOrder::Asc => cmp,
            SortOrder::Desc => cmp.reverse(),
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.parameter, self.order)
    }
}

/// Unit-tests for sort
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Priority;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn task(title: &str, priority: Priority, seconds_ago: i64) -> Task {
        Task {
            id: Uuid::new_v4(),
            title: title.to_string(),
            title_lower: title.to_lowercase(),
            description: String::new(),
            priority,
            completed: false,
            pinned: false,
            created_at: Utc::now() - Duration::seconds(seconds_ago),
            updated_at: Utc::now(),
            folder_id: None,
            id_formatted: String::new(),
        }
    }

    #[test]
    fn should_cycle_through_sort_by_and_order() {
        let mut sort: Sort = Sort::default();

        sort.parameter.next();
        assert_eq!(sort.parameter, SortBy::Title);
        sort.parameter.next();
        assert_eq!(sort.parameter, SortBy::CreatedAt);
        sort.parameter.next();
        assert_eq!(sort.parameter, SortBy::Priority);

        sort.order.next();
        assert_eq!(sort.order, SortOrder::Asc);
        sort.order.next();
        assert_eq!(sort.order, SortOrder::Desc);
    }

    #[test]
    fn should_compare_priority_desc() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Desc);
        let high = task("High", Priority::High, 0);
        let low = task("Low", Priority::Low, 0);

        assert_eq!(sort.compare(&high, &low), Ordering::Less);
    }

    #[test]
    fn should_compare_title_case_insensitive() {
        let sort = Sort::new(SortBy::Title, SortOrder::Asc);
        let a = task("apple", Priority::Low, 0);
        let b = task("Banana", Priority::Low, 0);

        assert_eq!(sort.compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn should_stabilize_priority_sorting() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Asc);

        let old_task = task("Old", Priority::Medium, 10);
        let new_task = task("New", Priority::Medium, 0);

        assert_eq!(sort.compare(&old_task, &new_task), Ordering::Less);
    }

    #[test]
    fn should_always_place_pinned_tasks_on_top() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Asc);
        let mut pinned_low = task("Pinned Low", Priority::Low, 100);
        pinned_low.pinned = true;

        let normal_high = task("Normal High", Priority::High, 0);
        assert_eq!(sort.compare(&pinned_low, &normal_high), Ordering::Less);
        assert_eq!(sort.compare(&normal_high, &pinned_low), Ordering::Greater);
    }

    #[test]
    fn should_sort_between_two_pinned_tasks() {
        let sort = Sort::new(SortBy::Priority, SortOrder::Asc);
        let mut pinned_low = task("Low", Priority::Low, 0);
        pinned_low.pinned = true;

        let mut pinned_high = task("High", Priority::High, 0);
        pinned_high.pinned = true;

        let expected = pinned_high.priority.cmp(&pinned_low.priority);
        assert_eq!(sort.compare(&pinned_high, &pinned_low), expected);
    }

    #[test]
    fn should_keep_pinned_tasks_grouped_at_top() {
        let sort = Sort::new(SortBy::Title, SortOrder::Asc);

        let mut pinned1 = task("Z Pinned", Priority::Low, 0);
        pinned1.pinned = true;

        let mut normal1 = task("A Normal", Priority::High, 0);
        normal1.pinned = false;

        let mut normal2 = task("B Normal", Priority::High, 0);
        normal2.pinned = false;

        let tasks = vec![normal1, pinned1.clone(), normal2];
        let mut sorted = tasks.clone();

        sorted.sort_by(|a, b| sort.compare(a, b));

        assert_eq!(sorted[0].id, pinned1.id);
        assert_eq!(sorted[1].title, "A Normal");
        assert_eq!(sorted[2].title, "B Normal");
    }
}
