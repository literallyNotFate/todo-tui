use crate::{models::Task, ui::RenderContext};
use ratatui::text::{Line, Span};
use serde::{Deserialize, Serialize};

/// All tasks related configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct TaskConfig {
    pub auto_sort: bool,
    pub max_title_length: usize,
    pub display_format: String,

    #[serde(skip)]
    pub compiled_format: Vec<TaskTemplate>,
}

/// Starship-like format template
#[derive(Debug, Clone, PartialEq)]
pub enum TaskTemplate {
    Folder,
    Title,
    ID,
    Text(String),
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            auto_sort: true,
            max_title_length: 50,
            display_format: "{folder} {title}".to_string(),
            compiled_format: Vec::new(),
        }
    }
}

impl TaskConfig {
    /// Compiles string format to token list
    pub fn compile(&mut self) {
        self.compiled_format = parse_template(&self.display_format);
    }

    /// Validates task config
    pub fn validate(&mut self) {
        self.max_title_length = self.max_title_length.clamp(10, 200);
    }

    /// Build line
    pub fn build(
        &self,
        task: &Task,
        title_spans: Vec<Span<'static>>,
        folder_span: Span<'static>,
    ) -> Line<'static> {
        let mut final_spans = Vec::new();

        for token in &self.compiled_format {
            match token {
                TaskTemplate::Folder => final_spans.push(folder_span.clone()),
                TaskTemplate::Title => {
                    let full: String = title_spans.iter().map(|s| s.content.as_ref()).collect();
                    let truncated = RenderContext::truncate(&full, self.max_title_length);
                    let style = title_spans.first().map(|s| s.style).unwrap_or_default();
                    final_spans.push(Span::styled(truncated, style));
                }
                TaskTemplate::ID => final_spans.push(Span::raw(format!("#{} ", task.id))),
                TaskTemplate::Text(text) => final_spans.push(Span::raw(text.clone())),
            }
        }

        Line::from(final_spans)
    }
}

/// Template parser with tag support: {folder}, {title}, {id}
fn parse_template(format: &str) -> Vec<TaskTemplate> {
    let mut tokens = Vec::new();
    let mut current_text = String::new();
    let mut chars = format.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if !current_text.is_empty() {
                tokens.push(TaskTemplate::Text(current_text.split_off(0)));
            }

            let mut tag = String::new();
            while let Some(next) = chars.next() {
                if next == '}' {
                    break;
                }
                tag.push(next);
            }

            match tag.to_lowercase().as_str() {
                "folder" => tokens.push(TaskTemplate::Folder),
                "title" => tokens.push(TaskTemplate::Title),
                "id" => tokens.push(TaskTemplate::ID),
                other => tokens.push(TaskTemplate::Text(format!("{{{}}}", other))),
            }
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        tokens.push(TaskTemplate::Text(current_text));
    }

    tokens
}

/// Unit-tests for task config
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_basic_values_in_parser() {
        let tokens = parse_template("{folder} {title}");
        assert_eq!(
            tokens,
            vec![
                TaskTemplate::Folder,
                TaskTemplate::Text(" ".to_string()),
                TaskTemplate::Title
            ]
        );
    }

    #[test]
    fn should_handle_id_and_extra_text_in_parser() {
        let tokens = parse_template("ID: {id} | {title}");
        assert_eq!(
            tokens,
            vec![
                TaskTemplate::Text("ID: ".to_string()),
                TaskTemplate::ID,
                TaskTemplate::Text(" | ".to_string()),
                TaskTemplate::Title
            ]
        );
    }

    #[test]
    fn should_handle_unknown_tag_in_parser() {
        let tokens = parse_template("{unknown}");
        assert_eq!(tokens, vec![TaskTemplate::Text("{unknown}".to_string())]);
    }

    #[test]
    fn should_handle_task_config_validation() {
        let mut config = TaskConfig::default();
        config.max_title_length = 5;
        config.validate();
        assert_eq!(config.max_title_length, 10);

        config.max_title_length = 500;
        config.validate();
        assert_eq!(config.max_title_length, 200);
    }

    #[test]
    fn should_test_build_format() {
        let mut config = TaskConfig::default();
        config.display_format = "{id} | {title}".to_string();
        config.compile();

        let task: Task = Task::new("Test");
        let title_spans = vec![Span::raw("Test")];
        let folder_span = Span::raw("");

        let line = config.build(&task, title_spans, folder_span);
        assert_eq!(line.spans.len(), 3);
    }

    #[test]
    fn should_handle_max_title_length_clamping() {
        let mut config = TaskConfig::default();
        config.max_title_length = 2;
        config.compile();

        let task: Task = Task::new("Long Title");
        let title_spans = vec![Span::raw("Long Title")];
        let folder_span = Span::raw("");

        let line = config.build(&task, title_spans, folder_span);
        assert_eq!(line.spans.last().unwrap().content, "L…");
    }
}
