use ratatui::{Frame, layout::Rect};

pub struct Fallback;

impl Fallback {
    pub fn render(frame: &mut Frame, frame_area: Rect) {
        use crate::app::utils::{
            constants::{size::*, terminal::*, theme::*},
            layout::centered,
        };
        use ratatui::{
            style::{Color, Modifier, Style},
            text::{Line, Span, Text},
            widgets::{Block, Clear, Paragraph, Wrap},
        };

        let area: Rect = centered(frame_area, FALLBACK_WIDTH, FALLBACK_HEIGHT);
        let colors: (Color, Color) = dimension_colors(frame_area);

        let message: Vec<Line> = vec![
            Line::styled("Terminal is too small!", Style::default().fg(COLOR_RED)).centered(),
            Line::from(""),
            Line::from(vec![
                Span::styled("Required: ", Style::default().fg(TEXT_PRIMARY)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
            Line::from(""),
            Line::from(vec![
                Span::raw("Current:  "),
                Span::styled(
                    format!("{}", frame_area.width),
                    Style::default().fg(colors.0).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", frame_area.height),
                    Style::default().fg(colors.1).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
        ];

        let paragraph: Paragraph = Paragraph::new(Text::from(message))
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: false })
            .block(Block::default());

        frame.render_widget(Clear, frame_area);
        frame.render_widget(paragraph, area);
    }
}
