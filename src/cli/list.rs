use crate::{
    cli::FilterMode,
    core::Sort,
    models::{Priority, Task},
};
use comfy_table::{Cell, Color, ContentArrangement, Table, presets};

/// `list` command implementation
pub fn list_tasks(
    all_tasks: &[Task],
    filter: FilterMode,
    limit: Option<usize>,
    query: Option<String>,
    sort: Sort,
) {
    let mut table: Table = Table::new();

    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("Status").fg(Color::White),
            Cell::new("ID").fg(Color::Grey),
            Cell::new("Title").fg(Color::Magenta),
            Cell::new("Priority").fg(Color::Yellow),
            Cell::new("Created").fg(Color::Blue),
        ]);

    let display_tasks: Vec<&Task> = get_filtered_tasks(all_tasks, filter, query, limit, sort);
    if display_tasks.is_empty() {
        println!("No tasks to be shown!");
        return;
    }

    for task in display_tasks {
        table.add_row(vec![
            get_status(task),
            Cell::new(&task.id_formatted).fg(Color::DarkGrey),
            Cell::new(&task.title),
            Cell::new(task.priority.as_str()).fg(match task.priority {
                Priority::High => Color::Red,
                Priority::Medium => Color::Yellow,
                _ => Color::Green,
            }),
            Cell::new(task.time_ago()),
        ]);
    }

    println!("{table}");
}

/// Helper to filter tasks (w/query, limit)
pub fn get_filtered_tasks<'a>(
    tasks: &'a [Task],
    filter: FilterMode,
    query: Option<String>,
    limit: Option<usize>,
    sort: Sort,
) -> Vec<&'a Task> {
    use crate::state::{ApplicationState, SidebarTab};

    let tab: SidebarTab = filter.into();
    let mut filtered: Vec<&Task> =
        ApplicationState::filter(tasks, tab, None, &query.unwrap_or_default()).collect();

    filtered.sort_by(|a, b| sort.compare(a, b));
    if let Some(n) = limit {
        filtered.into_iter().take(n).collect()
    } else {
        filtered
    }
}

/// Helper function to return table cell of status based on task
fn get_status(task: &Task) -> Cell {
    if task.pinned {
        Cell::new("PINNED").fg(Color::Yellow)
    } else if task.completed {
        Cell::new("DONE").fg(Color::Green)
    } else {
        Cell::new("TO DO").fg(Color::Red)
    }
}

/// Unit-tests for list CLI command
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SortBy, SortOrder};
    use chrono::Utc;
    use uuid::Uuid;

    fn setup_tasks() -> Vec<Task> {
        vec![
            Task {
                id: Uuid::new_v4(),
                title: "Task 1".into(),
                title_lower: "task 1".into(),
                completed: false,
                priority: Priority::High,
                created_at: Utc::now(),
                ..Default::default()
            },
            Task {
                id: Uuid::new_v4(),
                title: "Task 2".into(),
                title_lower: "task 2".into(),
                completed: true,
                priority: Priority::Low,
                created_at: Utc::now(),
                ..Default::default()
            },
        ]
    }

    #[test]
    fn should_handle_list_command_filter_active() {
        let tasks = setup_tasks();
        let sort = Sort::default();
        let result = get_filtered_tasks(&tasks, FilterMode::Active, None, None, sort);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "Task 1");
    }

    #[test]
    fn should_handle_list_command_limit() {
        let tasks = setup_tasks();
        let sort = Sort::default();
        let result = get_filtered_tasks(&tasks, FilterMode::All, None, Some(1), sort);

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_handle_list_command_query_search() {
        let tasks = setup_tasks();
        let sort = Sort::default();
        let result = get_filtered_tasks(&tasks, FilterMode::All, Some("Task 2".into()), None, sort);

        assert_eq!(result.len(), 1);
        assert!(result[0].completed);
    }

    #[test]
    fn should_handle_list_command_sorting_priority() {
        let tasks = setup_tasks();
        let sort = Sort::new(SortBy::Priority, SortOrder::Desc);
        let result = get_filtered_tasks(&tasks, FilterMode::All, None, None, sort);

        assert_eq!(result[0].title, "Task 1");
        assert_eq!(result[1].title, "Task 2");
    }
}
