use crate::{
    models::TodoDetails,
    state::AdaptiveScroll,
    theme::ThemePalette,
    traits::{Modal, ModalResult, ModalSize},
    ui::{RenderContext, center, scrollable},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
};

/// What is going to be shown
#[derive(Debug, Clone)]
pub enum PopupContent {
    Message(String),
    Task(TodoDetails),
    Help(Vec<Line<'static>>),
}

/// Defines how popup is getting closed (on any key or on specific)
#[derive(Debug, Clone, PartialEq)]
pub enum PopupCloseBehavior {
    AnyKey,
    Specific(KeyCode),
}

/// Type of a popup
#[derive(Debug, Clone, PartialEq)]
pub enum PopupKind {
    Info,
    Error,
    Success,
}

/// Popup modal widget
#[derive(Debug)]
pub struct Popup {
    pub title: String,
    pub content: PopupContent,
    pub kind: PopupKind,
    pub close_behavior: PopupCloseBehavior,
    pub scroll: AdaptiveScroll,
    pub size: ModalSize,
}

impl Popup {
    /// Creating info popup template
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Info,
            title: String::from(" Info "),
            content: PopupContent::Message(message.into()),
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
            scroll: AdaptiveScroll::default(),
            size: ModalSize::Medium,
        }
    }

    /// Creating success popup template
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Success,
            title: String::from(" Success "),
            ..Self::info(message)
        }
    }

    /// Creating error popup template
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Error,
            title: String::from(" Error "),
            ..Self::info(message)
        }
    }

    /// Creating task details popup template
    pub fn details(title: String, details: TodoDetails) -> Self {
        Self {
            title,
            content: PopupContent::Task(details),
            kind: PopupKind::Info,
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
            scroll: AdaptiveScroll::default(),
            size: ModalSize::Large,
        }
    }

    /// Creating hotkeys popup templete
    pub fn help(lines: Vec<Line<'static>>) -> Self {
        Self {
            kind: PopupKind::Info,
            title: " Keyboard Shortcuts ".into(),
            content: PopupContent::Help(lines),
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Char('?')),
            scroll: AdaptiveScroll::default(),
            size: ModalSize::Medium,
        }
    }

    /// With specific title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Closes on any key
    pub fn close_on_any_key(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::AnyKey;
        self
    }

    /// Closes on specific key
    pub fn close_on(mut self, key: KeyCode) -> Self {
        self.close_behavior = PopupCloseBehavior::Specific(key);
        self
    }

    /// With modal size
    pub fn with_size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    /// With adaptive scroll
    pub fn with_scroll(mut self, scroll: AdaptiveScroll) -> Self {
        self.scroll = scroll;
        self
    }

    /// Vertical layout for inner content
    fn vertical_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
            .split(area)
    }

    /// Horizontal layout for inner content
    fn horizontal_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10), // Left
                Constraint::Percentage(80),
                Constraint::Percentage(10), // Right
            ])
            .split(area)
    }

    /// Generate bottom title based on close behavior
    fn bottom_keys(&self, palette: &ThemePalette) -> Line<'static> {
        let mut spans = Vec::new();

        let close_key = match self.close_behavior {
            PopupCloseBehavior::AnyKey => "any".to_string(),
            PopupCloseBehavior::Specific(code) => format!("{}", code),
        };

        spans.push(Span::styled(
            format!(" <{}>", close_key),
            Style::default().fg(palette.success).bold(),
        ));
        spans.push(Span::styled(":close ", Style::default().fg(palette.muted)));

        if matches!(self.content, PopupContent::Task(_))
            || matches!(self.content, PopupContent::Help(_))
        {
            spans.push(Span::styled(
                " <j/k>",
                Style::default().fg(palette.accent).bold(),
            ));
            spans.push(Span::styled(":scroll ", Style::default().fg(palette.muted)));
        }

        Line::from(spans).centered()
    }

    /// Return color based on kind
    fn color_on_kind(&self, palette: &ThemePalette) -> Color {
        match self.kind {
            PopupKind::Info => palette.accent,
            PopupKind::Success => palette.success,
            PopupKind::Error => palette.error,
        }
    }
}

impl Modal for Popup {
    /// Calculate area for popup
    fn area(&self, frame_area: Rect) -> Rect {
        let (width, height) = self.size.percentages();
        center(frame_area, width, height)
    }

    /// Popup rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::{Block, Borders, Padding, Paragraph, Wrap};

        let palette: ThemePalette = ctx.palette();
        let popup_block: Block = Block::bordered()
            .border_type(ctx.config.border_type.into())
            .title_alignment(Alignment::Center)
            .title(self.title.as_str())
            .title_bottom(self.bottom_keys(&palette))
            .border_style(self.color_on_kind(&palette))
            .fg(palette.fg)
            .bg(palette.bg);

        let inner_area = popup_block.inner(area);
        ctx.render_widget(popup_block, area);

        match &self.content {
            PopupContent::Message(msg) => {
                let vertical_chunks: std::rc::Rc<[Rect]> = self.vertical_layout(inner_area);
                let message_area: Rect = self.horizontal_layout(vertical_chunks[1])[1];
                let message: Paragraph = Paragraph::new(msg.as_str())
                    .centered()
                    .wrap(Wrap { trim: true });
                ctx.render_widget(message, message_area);
            }
            PopupContent::Task(details) => {
                let (status_text, status_style) = if details.completed {
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

                let header_line = Line::from(vec![
                    Span::styled(" ● ", Style::default().fg(palette.accent)),
                    Span::styled(&details.title, Style::default().fg(palette.fg).bold()),
                    Span::raw(" "),
                    Span::styled(
                        format!("#{}", details.id_short),
                        Style::default().fg(palette.muted).italic(),
                    ),
                ]);

                let meta_line = Line::from(vec![
                    Span::styled(status_text, status_style),
                    Span::raw("  "),
                    Span::styled("Created ", Style::default().fg(palette.muted)),
                    Span::styled(&details.created_at, Style::default().fg(palette.success)),
                    Span::raw("  •  "),
                    Span::styled("Updated ", Style::default().fg(palette.muted)),
                    Span::styled(&details.updated_at, Style::default().fg(palette.accent)),
                ]);

                let content_lines: Vec<Line> = details
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
                    .split(inner_area);

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
                    |ctx, area| {
                        let paragraph = Paragraph::new(content_lines.clone())
                            .wrap(Wrap { trim: true })
                            .scroll((self.scroll.current.get() as u16, 0));
                        ctx.render_widget(paragraph, area);
                    },
                );
            }
            PopupContent::Help(lines) => {
                let mid: usize = (lines.len() + 1) / 2;
                let dummy_lines = vec![Line::from(""); mid];

                scrollable(
                    ctx,
                    inner_area,
                    Block::default().title(""),
                    &self.scroll,
                    &dummy_lines,
                    false,
                    Style::default().fg(palette.accent),
                    |ctx, area| {
                        let chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints([
                                Constraint::Percentage(48),
                                Constraint::Percentage(4),
                                Constraint::Percentage(48),
                            ])
                            .split(area);

                        let (left, right) = lines.split_at(mid);
                        let scroll_val = self.scroll.current.get() as u16;

                        ctx.render_widget(
                            Paragraph::new(left.to_vec()).scroll((scroll_val, 0)),
                            chunks[0],
                        );
                        ctx.render_widget(
                            Paragraph::new(right.to_vec()).scroll((scroll_val, 0)),
                            chunks[2],
                        );
                    },
                );
            }
        }
    }

    /// Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<ModalResult> {
        if matches!(self.content, PopupContent::Task(_))
            || matches!(self.content, PopupContent::Help(_))
        {
            match key {
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll.scroll_down();
                    return None;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll.scroll_up();
                    return None;
                }
                _ => {}
            }
        }

        match self.close_behavior {
            PopupCloseBehavior::AnyKey => Some(ModalResult::Cancelled),
            PopupCloseBehavior::Specific(k) if k == key => Some(ModalResult::Cancelled),
            _ => None,
        }
    }
}

/// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::UIConfig, models::Todo, theme::ThemeName};

    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_message_popup() {
        let mut popup: Popup = Popup::success("Success");

        assert_eq!(popup.kind, PopupKind::Success);
        assert!(matches!(popup.content, PopupContent::Message(_)));
        assert_eq!(popup.title, " Success ");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );

        popup = Popup::info("Info");
        assert_eq!(popup.kind, PopupKind::Info);
        assert!(matches!(popup.content, PopupContent::Message(_)));

        popup = Popup::error("Error");
        assert_eq!(popup.kind, PopupKind::Error);
        assert!(matches!(popup.content, PopupContent::Message(_)));
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup: Popup = Popup::success("Task completed!")
            .title("Some title")
            .close_on_any_key();

        assert_eq!(popup.kind, PopupKind::Success);
        assert!(matches!(popup.content, PopupContent::Message(_)));
        assert_eq!(popup.title, "Some title");
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
    }

    #[test]
    fn should_create_task_details_popup() {
        let todo = Todo::new("Task 1", "Desc 1", None);
        let details = TodoDetails::from(&todo, &UIConfig::default());
        let popup: Popup = Popup::details("Test".to_string(), details);

        assert_eq!(popup.kind, PopupKind::Info);
        assert!(matches!(popup.content, PopupContent::Task(_)));
        assert_eq!(popup.title, "Test");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup: Popup = Popup::info("Test").close_on_any_key();

        assert_eq!(
            popup.handle_key(KeyCode::Char('q')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Enter),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Esc), Some(ModalResult::Cancelled));
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup: Popup = Popup::error("Test").close_on(KeyCode::Char('y'));

        assert_eq!(
            popup.handle_key(KeyCode::Char('y')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    #[test]
    fn should_handle_scroll_input() {
        let todo = Todo::new("T", "D", None);
        let config = UIConfig::default();
        let details = TodoDetails::from(&todo, &config);
        let mut popup = Popup::details("Test".to_string(), details);

        assert_eq!(popup.scroll.current.get(), 0);

        popup.handle_key(KeyCode::Char('j'));
        assert_eq!(popup.scroll.current.get(), 1);

        popup.handle_key(KeyCode::Char('k'));
        assert_eq!(popup.scroll.current.get(), 0);

        popup.handle_key(KeyCode::Char('k'));
        assert_eq!(popup.scroll.current.get(), 0);
    }

    #[test]
    fn should_calculate_dynamic_area_for_popup() {
        let frame: Rect = create_helper_frame();
        let small_popup = Popup::info("Small").with_size(ModalSize::Small);
        let large_popup = Popup::info("Large").with_size(ModalSize::Large);

        let small_area = small_popup.area(frame);
        let large_area = large_popup.area(frame);

        assert!(large_area.width > small_area.width);
        assert!(large_area.height > small_area.height);
    }

    #[test]
    fn should_render_correct_bottom_keys_text() {
        let palette = ThemeName::GruvboxDark.palette();
        let popup_msg = Popup::info("Msg");
        let keys_msg = popup_msg.bottom_keys(&palette);

        assert!(!format!("{:?}", keys_msg).contains("scroll"));

        let todo = Todo::new("T", "D", None);
        let details = TodoDetails::from(&todo, &UIConfig::default());
        let popup_task = Popup::details("Test".to_string(), details);
        let keys_task = popup_task.bottom_keys(&palette);

        let content = format!("{:?}", keys_task);
        assert!(content.contains("j/k"));
        assert!(content.contains("scroll"));
    }

    #[test]
    fn should_return_color_based_on_popup_kind_with_theme() {
        let mut popup = Popup::success("Test");
        let mut palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut color: Color = popup.color_on_kind(&palette);
        assert_eq!(color, Color::Rgb(184, 187, 38));

        popup = Popup::info("Test");
        color = popup.color_on_kind(&palette);
        assert_eq!(color, Color::Rgb(250, 189, 47));

        popup = Popup::error("Test");
        palette = ThemeName::CatppuccinMocha.palette();
        color = popup.color_on_kind(&palette);
        assert_ne!(color, Color::Rgb(251, 73, 52));
    }
}
