use crate::{
    models::{FolderColor, Priority},
    ui::{EnumInput, RenderContext, TextInput, widgets::input::Input},
};
use ratatui::layout::Rect;
use tui_textarea::TextArea;

/// All possible field types for form (text input, enum input, textarea, button)
#[derive(Clone, Debug)]
pub enum FieldType {
    Text { input: TextInput },
    PriorityEnum { input: EnumInput<Priority> },
    ColorEnum { input: EnumInput<FolderColor> },
    Multiline { input: TextArea<'static> },
    Button,
}

impl FieldType {
    /// Create text input with buffer and title
    pub fn text(title: &str, value: &str) -> Self {
        Self::Text {
            input: TextInput::from(value).title(title),
        }
    }

    /// Create enum input with selected value and title
    pub fn priority(p: Priority) -> Self {
        Self::PriorityEnum {
            input: EnumInput::from(p).title(" Priority "),
        }
    }

    /// Create color enum input
    pub fn color(color: FolderColor) -> Self {
        Self::ColorEnum {
            input: EnumInput::from(color).title(" Folder Color "),
        }
    }

    /// Create textarea input with buffer
    pub fn textarea(value: &str) -> Self {
        let lines: Vec<String> = value.lines().map(|s| s.to_string()).collect();

        let lines: Vec<String> = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };

        Self::Multiline {
            input: TextArea::new(lines),
        }
    }

    /// Render field to the terminal
    pub fn render(&self, ctx: &mut RenderContext, area: Rect, is_focused: bool) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::Style,
            widgets::{Block, Paragraph},
        };

        let palette = ctx.palette();
        let border_type = ctx.config.ui.border_type.into();

        let focused_style = if is_focused {
            Style::default().fg(palette.accent)
        } else {
            Style::default().fg(palette.muted)
        };

        match self {
            FieldType::Text { input } => {
                input.render(ctx, area, is_focused);
            }
            FieldType::PriorityEnum { input } => {
                input.render(ctx, area, is_focused);
            }
            FieldType::ColorEnum { input } => {
                input.render(ctx, area, is_focused);

                let swatch_area: Rect = Rect::new(area.x + area.width - 2, area.y + 1, 1, 1);
                let swatch =
                    Paragraph::new("█").style(Style::default().fg(input.selected.value.into()));

                ctx.render_widget(swatch, swatch_area);
            }
            FieldType::Multiline { input } => {
                let mut input = input.clone();

                if is_focused {
                    input.set_cursor_style(
                        Style::default().bg(palette.accent).fg(palette.selection),
                    );
                } else {
                    input.set_cursor_style(Style::default());
                }

                let block: Block = Block::bordered()
                    .title(" Description ")
                    .border_type(border_type)
                    .border_style(focused_style);

                input.set_block(block);
                input.set_style(Style::default().fg(palette.fg));
                ctx.render_widget(&input, area);
            }
            FieldType::Button => {
                let button_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(15),
                        Constraint::Min(0),
                        Constraint::Length(15),
                    ])
                    .split(area);

                let text_style = if is_focused {
                    Style::default().fg(palette.fg)
                } else {
                    Style::default().fg(palette.muted)
                };

                let button: Paragraph = Paragraph::new(" Save ")
                    .block(
                        Block::bordered()
                            .border_type(border_type)
                            .border_style(focused_style)
                            .style(text_style),
                    )
                    .centered();

                ctx.render_widget(button, button_layout[2]);
            }
        }
    }
}

/// Field object in form
#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

impl Field {
    pub fn new(field_name: impl Into<String>, field_type: FieldType) -> Self {
        Self {
            name: field_name.into(),
            field_type,
        }
    }
}
