use crate::state::AdaptiveScroll;
use ratatui::{Frame, layout::Rect, style::Style, text::Line, widgets::Block};

pub fn scrollable<F>(
    frame: &mut Frame,
    area: Rect,
    block: Block,
    scroll: &AdaptiveScroll,
    content_lines: &[Line],
    is_table: bool,
    focused: Style,
    render_content: F,
) where
    F: FnOnce(&mut Frame, Rect),
{
    use ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState},
    };

    frame.render_widget(block.clone(), area);
    let inner: Rect = block.inner(area);
    let viewport: u16 = inner.height;

    let (total, pos, max_value, show_scrollbar) = if is_table {
        let total = content_lines.len();
        let effective_viewport = viewport.saturating_sub(2);
        let current = scroll.current.get() as usize;
        let max = total.saturating_sub(effective_viewport as usize);

        (
            total,
            current,
            max as u16,
            total > effective_viewport as usize,
        )
    } else {
        let text_width = inner.width.saturating_sub(1).max(1);
        let mut count = 0;
        for line in content_lines {
            let w = line.width() as u16;
            count += if w == 0 {
                1
            } else {
                (w + text_width - 1) / text_width
            };
        }
        let max = (count as u16).saturating_sub(viewport);
        let current = (scroll.current.get() as usize).min(max as usize);

        (max as usize, current, max, count as u16 > viewport)
    };

    scroll.max_scroll.set(max_value);
    if scroll.current.get() > max_value {
        scroll.current.set(max_value);
    }

    let [content_area, scroll_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if show_scrollbar {
            [Constraint::Min(0), Constraint::Length(3)]
        } else {
            [Constraint::Min(0), Constraint::Length(0)]
        })
        .areas(inner);

    render_content(frame, content_area);

    if show_scrollbar {
        let (scrollabr_pos, scrollbar_total, scrollbar_viewport) = if is_table {
            (pos, total, viewport.saturating_sub(2) as usize)
        } else {
            (pos, max_value as usize, 0)
        };

        let mut state: ScrollbarState = ScrollbarState::new(scrollbar_total)
            .position(scrollabr_pos)
            .viewport_content_length(scrollbar_viewport);

        let scrollbar: Scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .thumb_symbol("▉")
            .thumb_style(focused)
            .track_style(focused)
            .begin_style(focused)
            .end_style(focused);

        frame.render_stateful_widget(scrollbar, scroll_area, &mut state);
    }
}
