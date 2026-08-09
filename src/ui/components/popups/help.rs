use crate::{
    state::AdaptiveScroll,
    ui::{
        Popup, PopupComponent, RenderContext,
        widgets::modal::{ModalSize, popup::PopupKind},
    },
};
use ratatui::{crossterm::event::KeyCode, layout::Rect, text::Line};

/// Component to render helper popup (hotkeys)
pub struct HelpComponent {
    pub lines: Vec<Line<'static>>,
    pub scroll: AdaptiveScroll,
}

impl PopupComponent for HelpComponent {
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
            style::Style,
            widgets::{Block, Paragraph},
        };

        let palette = ctx.palette();
        scrollable(
            ctx,
            area,
            Block::default().title(""),
            &self.scroll,
            &self.lines,
            false,
            Style::default().fg(palette.accent),
            |ctx, inner_area| {
                let scroll_val = self.scroll.current.get() as u16;
                ctx.render_widget(
                    Paragraph::new(self.lines.clone()).scroll((scroll_val, 0)),
                    inner_area,
                );
            },
        );
    }
}

impl Popup {
    /// Creating hotkeys popup templete
    pub fn help(lines: Vec<Line<'static>>) -> Self {
        let help_component = HelpComponent {
            lines,
            scroll: AdaptiveScroll::default(),
        };

        Self::new(
            " Keyboard Shortcuts ",
            Box::new(help_component),
            PopupKind::Info,
        )
        .close_on(KeyCode::Char('?'))
        .with_size(ModalSize::Medium)
    }
}

/// Unit-tests for help component
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::widgets::modal::{ModalResult, popup::PopupCloseBehavior};

    fn create_test_lines(count: usize) -> Vec<Line<'static>> {
        (0..count)
            .map(|i| Line::from(format!("Hotkey {}", i)))
            .collect()
    }

    #[test]
    fn should_create_help_popup_via_factory() {
        let lines = create_test_lines(4);
        let popup = Popup::help(lines);

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.title, " Keyboard Shortcuts ");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Char('?'))
        );
        assert_eq!(popup.content.to_modal_result(), ModalResult::Cancelled);
    }

    #[test]
    fn should_properly_report_scrollable_status() {
        let lines = create_test_lines(2);
        let component = HelpComponent {
            lines,
            scroll: AdaptiveScroll::default(),
        };

        assert!(component.is_scrollable());
    }

    #[test]
    fn should_manage_internal_scroll_state() {
        let lines = create_test_lines(10);
        let component = HelpComponent {
            lines,
            scroll: AdaptiveScroll::default(),
        };

        assert_eq!(component.scroll.current.get(), 0);

        component.scroll_down();
        assert_eq!(component.scroll.current.get(), 1);

        component.scroll_up();
        assert_eq!(component.scroll.current.get(), 0);
    }

    #[test]
    fn should_allow_setting_external_scroll() {
        let lines = create_test_lines(5);
        let mut component = HelpComponent {
            lines,
            scroll: AdaptiveScroll::default(),
        };

        let external_scroll = AdaptiveScroll::default();
        external_scroll.current.set(3);
        component.set_scroll(external_scroll);

        assert_eq!(component.scroll.current.get(), 3);

        component.scroll_down();
        assert_eq!(component.scroll.current.get(), 4);
    }

    #[test]
    fn should_calculate_correct_mid_split_for_odd_and_even_counts() {
        let even_lines = create_test_lines(4);
        let mid_even = even_lines.len().div_ceil(2);
        assert_eq!(mid_even, 2);

        let odd_lines = create_test_lines(5);
        let mid_odd = odd_lines.len().div_ceil(2);
        assert_eq!(mid_odd, 3);

        let (left, right) = odd_lines.split_at(mid_odd);
        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 2);
    }
}
