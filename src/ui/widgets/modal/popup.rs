use crate::{
    core::Action,
    models::TaskDetails,
    state::AdaptiveScroll,
    theme::ThemePalette,
    ui::{
        Form, PopupComponent, RenderContext, WidgetResponse, center,
        widgets::modal::{Modal, ModalResult, ModalSize},
    },
};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
};

/// What is going to be shown
#[derive(Debug, Clone)]
pub enum PopupContent {
    Message(String),
    Task(TaskDetails),
    Help(Vec<Line<'static>>),
    Form(Form),
    ThemeSwitcher(Form),
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
pub struct Popup {
    pub title: String,
    pub content: Box<dyn PopupComponent>,
    pub kind: PopupKind,
    pub close_behavior: PopupCloseBehavior,
    pub size: ModalSize,
}

impl Popup {
    /// Create generic popup widget
    pub fn new(
        title: impl Into<String>,
        content: Box<dyn PopupComponent>,
        kind: PopupKind,
    ) -> Self {
        Self {
            title: title.into(),
            content,
            kind,
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
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
        self.content.set_scroll(scroll);
        self
    }

    /// Generate bottom title based on close behavior and keymaps
    fn bottom_keys(&self, ctx: &RenderContext) -> Line<'static> {
        let mut spans = Vec::new();
        let palette = ctx.palette();

        let close_key = match self.close_behavior {
            PopupCloseBehavior::AnyKey => "any".to_string(),
            PopupCloseBehavior::Specific(code) => code.to_string(),
        };

        spans.push(Span::styled(
            format!(" <{}>", close_key),
            Style::default().fg(palette.success).bold(),
        ));
        spans.push(Span::styled(":close ", Style::default().fg(palette.muted)));

        if self.content.is_scrollable() {
            let up = ctx.get_key(Action::MoveUp);
            let down = ctx.get_key(Action::MoveDown);

            if !up.is_empty() && !down.is_empty() {
                let scroll_hint = format!(
                    " <{}/{}>",
                    up.trim_matches(|c| c == '<' || c == '>'),
                    down.trim_matches(|c| c == '<' || c == '>')
                );

                spans.push(Span::styled(
                    scroll_hint,
                    Style::default().fg(palette.accent).bold(),
                ));
                spans.push(Span::styled(":scroll ", Style::default().fg(palette.muted)));
            }
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

    /// Generic popup rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::Block;

        let palette = ctx.palette();
        let popup_block: Block = Block::bordered()
            .border_type(ctx.config.ui.border_type.into())
            .title_alignment(Alignment::Center)
            .title(self.title.as_str())
            .title_bottom(self.bottom_keys(ctx))
            .border_style(self.color_on_kind(&palette))
            .fg(palette.fg)
            .bg(palette.bg);

        let inner_area = popup_block.inner(area);
        ctx.render_widget(popup_block, area);

        self.content.render(ctx, inner_area);
    }

    /// Action handling
    fn handle_action(&mut self, action: Option<Action>, event: &KeyEvent) -> Option<ModalResult> {
        let key: KeyCode = event.code;
        match self.content.handle_key(event) {
            WidgetResponse::Submit => {
                return Some(self.content.to_modal_result());
            }
            WidgetResponse::Cancel => return Some(ModalResult::Cancelled),
            WidgetResponse::Changed => {
                return Some(self.content.to_modal_result());
            }
            WidgetResponse::Continue => {}
        }

        if let Some(act) = action {
            match act {
                Action::MoveDown => {
                    self.content.scroll_down();
                    return None;
                }
                Action::MoveUp => {
                    self.content.scroll_up();
                    return None;
                }
                _ => {}
            }
        }

        match self.close_behavior {
            PopupCloseBehavior::AnyKey => Some(ModalResult::Cancelled),
            PopupCloseBehavior::Specific(k) if k == key => Some(ModalResult::Cancelled),
            _ => {
                if key == KeyCode::Esc {
                    Some(ModalResult::Cancelled)
                } else {
                    None
                }
            }
        }
    }
}

/// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::BuiltinTheme;
    use ratatui::crossterm::event::KeyModifiers;

    struct DummyComponent;
    impl PopupComponent for DummyComponent {
        fn render(&self, _ctx: &mut RenderContext, _area: Rect) {}
    }

    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup = Popup::new("Some title", Box::new(DummyComponent), PopupKind::Success)
            .close_on_any_key();

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.title, "Some title");
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup =
            Popup::new("Test", Box::new(DummyComponent), PopupKind::Info).close_on_any_key();
        let event_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let event_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert_eq!(
            popup.handle_action(None, &event_q),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(
            popup.handle_action(None, &event_enter),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup = Popup::new("Test", Box::new(DummyComponent), PopupKind::Error)
            .close_on(KeyCode::Char('y'));
        let event_y = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let event_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);

        assert_eq!(
            popup.handle_action(None, &event_y),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_action(None, &event_n), None);
    }

    #[test]
    fn should_close_on_esc_anyway() {
        let mut popup =
            Popup::new("Test", Box::new(DummyComponent), PopupKind::Info).close_on(KeyCode::Tab);
        let event_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(
            popup.handle_action(None, &event_esc),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_calculate_dynamic_area_for_popup() {
        let frame = create_helper_frame();
        let small_popup =
            Popup::new("S", Box::new(DummyComponent), PopupKind::Info).with_size(ModalSize::Small);
        let large_popup =
            Popup::new("L", Box::new(DummyComponent), PopupKind::Info).with_size(ModalSize::Large);

        let small_area = small_popup.area(frame);
        let large_area = large_popup.area(frame);

        assert!(large_area.width > small_area.width);
        assert!(large_area.height > small_area.height);
    }

    #[test]
    fn should_return_color_based_on_popup_kind_with_theme() {
        let palette = BuiltinTheme::GruvboxDark.palette();
        let success_popup = Popup::new("T", Box::new(DummyComponent), PopupKind::Success);
        assert_eq!(
            success_popup.color_on_kind(&palette),
            Color::Rgb(184, 187, 38)
        );

        let info_popup = Popup::new("T", Box::new(DummyComponent), PopupKind::Info);
        assert_eq!(info_popup.color_on_kind(&palette), Color::Rgb(250, 189, 47));
    }
}
