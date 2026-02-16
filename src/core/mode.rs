use crate::{enums::FocusArea, theme::ThemeColors};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};

/// Which mode user selects (for input handling)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Browsing,
    List,
    Form,
    Search,
}

impl ApplicationMode {
    /// Returns lines of hotkeys (w/commands and sections)
    pub fn hotkeys(&self, theme: &ThemeColors, focus: &FocusArea) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        match self {
            ApplicationMode::Browsing | ApplicationMode::List => {
                self.add_section(&mut lines, "Navigation", theme);
                self.add_command(&mut lines, "h/l", "Switch Panel", theme);
                self.add_command(&mut lines, "A-j/k", "Scroll Hotkeys", theme);

                match focus {
                    FocusArea::LeftPanel => {
                        self.add_command(&mut lines, "j/k", "Next/Prev Filter", theme);
                        self.add_command(&mut lines, "1-5", "Quick Filter", theme);
                    }
                    FocusArea::MainContent => {
                        self.add_command(&mut lines, "j/k", "Select Task", theme);
                        self.add_command(&mut lines, "J/K", "Move Task", theme);
                        self.add_command(&mut lines, "[/]", "Scroll Description", theme);
                    }
                }

                self.add_section(&mut lines, "Actions", theme);
                self.add_command(&mut lines, "<a>", "New Task", theme);

                if *focus == FocusArea::MainContent {
                    self.add_command(&mut lines, "<Enter>", "Toggle Completed", theme);
                    self.add_command(&mut lines, "<e>", "Update Task", theme);
                    self.add_command(&mut lines, "<d>", "Remove Task", theme);
                    self.add_command(&mut lines, "</>", "Search", theme);
                }

                self.add_command(&mut lines, "<x>", "Clear All", theme);

                self.add_section(&mut lines, "System", theme);
                self.add_command(&mut lines, "<s>", "Sort Type", theme);
                self.add_command(&mut lines, "<r>", "Reverse Sort", theme);
                self.add_command(&mut lines, "<t>", "Theme", theme);
                self.add_command(&mut lines, "<C-s>", "Save", theme);
                self.add_command(&mut lines, "<A-a>", "Autosave", theme);
                self.add_command(&mut lines, "Esc/q", "Quit", theme);
            }

            ApplicationMode::Form => {
                self.add_section(&mut lines, "Form Controls", theme);
                self.add_command(&mut lines, "<A-Enter>", "Submit", theme);
                self.add_command(&mut lines, "◄ / ►", "Change Priority", theme);
                self.add_command(&mut lines, "<Esc>", "Cancel", theme);
            }

            ApplicationMode::Search => {
                self.add_section(&mut lines, "Search", theme);
                self.add_command(&mut lines, "<Enter>", "Confirm", theme);
                self.add_command(&mut lines, "<Esc>", "Cancel", theme);
            }
        }

        lines
    }

    /// Adds hotkeys section
    fn add_section(
        &self,
        lines: &mut Vec<Line<'static>>,
        title: &'static str,
        theme: &ThemeColors,
    ) {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", title),
                Style::default().fg(theme.accent).bold(),
            ),
            Span::styled("─".repeat(20), Style::default().fg(theme.bg_dim)),
        ]));
    }

    /// Adds hotkey command
    fn add_command(
        &self,
        lines: &mut Vec<Line<'static>>,
        key: &'static str,
        desc: &'static str,
        theme: &ThemeColors,
    ) {
        let key_col_width = 12;
        let padded_key = format!("{:>width$} ", key, width = key_col_width);

        lines.push(Line::from(vec![
            Span::styled(padded_key, Style::default().fg(theme.accent).bold()),
            Span::styled("│ ", Style::default().fg(theme.border)),
            Span::styled(desc, Style::default().fg(theme.text_primary)),
        ]));
    }
}

/// Unit-tests for mode hotkeys
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_hotkeys_alignment_consistency() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::MainContent);

        for line in lines {
            let spans: Vec<_> = line.spans.iter().collect();
            if spans.len() >= 3 && spans[1].content.contains('│') {
                assert_eq!(spans[0].content.len(), 13, "Key column width must be 13");
            }
        }
    }

    #[test]
    fn should_handle_hotkeys_focus_logic() {
        let mode = ApplicationMode::Browsing;

        let left_lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::LeftPanel);
        let left_content: String = left_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(left_content.contains("Next/Prev Filter"),);
        assert!(!left_content.contains("Select Task"),);

        let main_lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::MainContent);
        let main_content: String = main_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(main_content.contains("Select Task"),);
        assert!(main_content.contains("Toggle Completed"),);
    }

    #[test]
    fn should_add_section_spacing() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::LeftPanel);

        let empty_lines = lines
            .iter()
            .filter(|l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content == ""))
            .count();
        assert!(
            empty_lines >= 2,
            "Sections should be separated by empty lines"
        );
    }
}
