use ratatui::{Frame, layout::Rect};

pub struct Fallback;

impl Fallback {
    pub fn render(frame: &mut Frame, frame_area: Rect) {
        use super::utils::render_fallback_message;
        use crate::app::utils::{
            constants::{size::*, terminal::*, theme::*},
            layout::centered,
        };
        use ratatui::{
            style::{Color, Style},
            text::{Line, Text},
            widgets::{Block, Clear, Paragraph, Wrap},
        };

        let area: Rect = centered(frame_area, FALLBACK_WIDTH, FALLBACK_HEIGHT);
        let colors: (Color, Color) = dimension_colors(frame_area);

        let message: Vec<Line> =
            render_fallback_message(frame_area.width, frame_area.height, colors);

        let paragraph: Paragraph = Paragraph::new(Text::from(message))
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: false })
            .block(Block::default());

        frame.render_widget(Clear, frame_area);
        frame.render_widget(paragraph, area);
    }
}
