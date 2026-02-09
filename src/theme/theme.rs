use super::ThemeColors;

/// Themes for application
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Theme {
    #[default]
    Gruvbox,
    Catppuccin,
    TokyoNight,
    Everforest,
    OneDark,
}

impl Theme {
    /// Return colors based on theme selection
    pub fn colors(&self) -> ThemeColors {
        match self {
            Theme::Gruvbox => ThemeColors::GRUVBOX,
            Theme::Catppuccin => ThemeColors::CATPPUCCIN,
            Theme::TokyoNight => ThemeColors::TOKYO_NIGHT,
            Theme::Everforest => ThemeColors::EVERFOREST,
            Theme::OneDark => ThemeColors::ONE_DARK,
        }
    }

    /// Next theme
    pub fn next(&self) -> Self {
        match self {
            Self::Gruvbox => Self::Catppuccin,
            Self::Catppuccin => Self::TokyoNight,
            Self::TokyoNight => Self::Everforest,
            Self::Everforest => Self::OneDark,
            Self::OneDark => Self::Gruvbox,
        }
    }
}

/// Unit-tests for themes
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_toggle_themes_and_return_corresponding_colors() {
        let mut theme: Theme = Theme::Gruvbox;
        assert_eq!(theme.colors(), ThemeColors::GRUVBOX);

        theme = theme.next();
        assert_eq!(theme.colors(), ThemeColors::CATPPUCCIN);

        theme = theme.next();
        assert_eq!(theme.colors(), ThemeColors::TOKYO_NIGHT);

        theme = theme.next();
        assert_eq!(theme.colors(), ThemeColors::EVERFOREST);

        theme = theme.next();
        assert_eq!(theme.colors(), ThemeColors::ONE_DARK);

        theme = theme.next();
        assert_eq!(theme.colors(), ThemeColors::GRUVBOX);
    }
}
