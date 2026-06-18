use crate::{
    models::TaskDetails,
    state::AdaptiveScroll,
    ui::{
        Popup, PopupComponent, RenderContext,
        widgets::modal::{ModalSize, popup::PopupKind},
    },
};
use ratatui::layout::Rect;

/// Component to render task details popup
pub struct DetailsComponent {
    pub details: TaskDetails,
    pub scroll: AdaptiveScroll,
}

impl DetailsComponent {
    pub fn new(details: TaskDetails) -> Self {
        Self {
            details,
            scroll: AdaptiveScroll::default(),
        }
    }
}

impl PopupComponent for DetailsComponent {
    fn is_scrollable(&self) -> bool {
        true
    }

    fn scroll_down(&self) {
        self.scroll.scroll_down();
    }

    fn scroll_up(&self) {
        self.scroll.scroll_up();
    }

    fn set_scroll(&mut self, scroll: AdaptiveScroll) {
        self.scroll = scroll;
    }

    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use crate::ui::scrollable;
        use ratatui::{
            layout::{Alignment, Constraint, Direction, Layout},
            style::{Style, Stylize},
            text::{Line, Span},
            widgets::{Block, Borders, Padding, Paragraph, Wrap},
        };

        let palette = ctx.palette();
        let (status_text, status_style) = if self.details.completed {
            (
                " DONE ",
                Style::default().bg(palette.success).fg(palette.bg).bold(),
            )
        } else {
            (
                " ACTIVE ",
                Style::default().bg(palette.accent).fg(palette.bg).bold(),
            )
        };

        let folder_display: String = if let Some(fid) = self.details.folder_id {
            format!(" Folder: {}... ", &fid.to_string()[..8])
        } else {
            " Inbox ".to_string()
        };

        let header_line = Line::from(vec![
            Span::styled(" ● ", Style::default().fg(palette.accent)),
            Span::styled(&self.details.title, Style::default().fg(palette.fg).bold()),
            Span::raw(" "),
            Span::styled(
                self.details.id_display.as_str(),
                Style::default().fg(palette.muted).italic(),
            ),
        ]);

        let meta_line = Line::from(vec![
            Span::styled(status_text, status_style),
            Span::raw("  "),
            Span::styled(
                folder_display,
                Style::default().fg(palette.secondary).italic(),
            ),
            Span::raw(" • "),
            Span::styled(
                format!(" {} Pinned ", ctx.config.ui.symbols.pinned),
                Style::default().fg(palette.warning).bold(),
            ),
            Span::raw(" • "),
            Span::styled("Created ", Style::default().fg(palette.muted)),
            Span::styled(
                &self.details.created_at,
                Style::default().fg(palette.success),
            ),
            Span::raw("  •  "),
            Span::styled("Updated ", Style::default().fg(palette.muted)),
            Span::styled(
                &self.details.updated_at,
                Style::default().fg(palette.accent),
            ),
        ]);

        let content_lines: Vec<Line> = self
            .details
            .description
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Title + ID
                Constraint::Length(1),
                Constraint::Length(1), // Status + Times
                Constraint::Length(1), // Separator
                Constraint::Fill(1),   // Description
            ])
            .margin(1)
            .split(area);

        ctx.render_widget(Paragraph::new(header_line), chunks[0]);
        ctx.render_widget(Paragraph::new(meta_line), chunks[2]);

        ctx.render_widget(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(palette.muted)
                .title_alignment(Alignment::Right)
                .title(Span::styled(
                    " info ",
                    Style::default().fg(palette.muted).italic(),
                )),
            chunks[3],
        );

        scrollable(
            ctx,
            chunks[4],
            Block::default()
                .title(Span::styled(
                    " Description ",
                    Style::default().fg(palette.muted),
                ))
                .padding(Padding::vertical(1)),
            &self.scroll,
            &content_lines,
            false,
            Style::default().fg(palette.accent),
            |ctx, inner_area| {
                let paragraph = Paragraph::new(content_lines.clone())
                    .wrap(Wrap { trim: true })
                    .scroll((self.scroll.current.get() as u16, 0));
                ctx.render_widget(paragraph, inner_area);
            },
        );
    }
}

impl Popup {
    /// Creating task details popup template
    pub fn details(title: impl Into<String>, details: TaskDetails) -> Self {
        Self::new(
            title,
            Box::new(DetailsComponent::new(details)),
            PopupKind::Info,
        )
        .with_size(ModalSize::Large)
    }
}

/// Unit-tests for details component
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::UIConfig,
        models::Task,
        ui::{WidgetResponse, widgets::modal::ModalResult},
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_details(title: &str, desc: &str, completed: bool) -> TaskDetails {
        let mut task = Task::new(title);
        task.description = desc.to_string();
        task.completed = completed;
        TaskDetails::from(&task, &UIConfig::default())
    }

    #[test]
    fn should_create_details_popup_via_factory() {
        let details = create_test_details("Fix Hydration", "Fix SSR mismatch errors", false);
        let popup = Popup::details(" Details ", details);

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Details ");
        assert_eq!(popup.content.to_modal_result(), ModalResult::Cancelled);
    }

    #[test]
    fn should_properly_report_scrollable_status() {
        let details = create_test_details("T", "D", false);
        let component = DetailsComponent::new(details);

        assert!(component.is_scrollable());
    }

    #[test]
    fn should_manage_internal_scroll_state() {
        let details = create_test_details("Long Description Task", "Line 1\nLine 2\nLine 3", false);
        let component = DetailsComponent::new(details);

        assert_eq!(component.scroll.current.get(), 0);

        component.scroll_down();
        assert_eq!(component.scroll.current.get(), 1);

        component.scroll_up();
        assert_eq!(component.scroll.current.get(), 0);
    }

    #[test]
    fn should_accept_external_scroll_state_via_setter() {
        let details = create_test_details("Shared State Task", "Description", true);
        let mut component = DetailsComponent::new(details);

        let external_scroll = AdaptiveScroll::default();
        external_scroll.current.set(42);
        component.set_scroll(external_scroll);

        assert_eq!(component.scroll.current.get(), 42);

        component.scroll_down();
        assert_eq!(component.scroll.current.get(), 43);
    }

    #[test]
    fn should_handle_key_as_continue_and_not_intercept_input() {
        let details = create_test_details("Read Only", "Desc", false);
        let mut component = DetailsComponent::new(details);
        let random_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let response = component.handle_key(&random_key);

        assert!(matches!(response, WidgetResponse::Continue));
    }
}
