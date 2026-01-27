use super::ThemeColors;

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
    pub fn data(&self) -> ThemeColors {
        match self {
            Theme::Gruvbox => ThemeColors::GRUVBOX,
            Theme::Catppuccin => ThemeColors::CATPPUCCIN,
            Theme::TokyoNight => ThemeColors::TOKYO_NIGHT,
            Theme::Everforest => ThemeColors::EVERFOREST,
            Theme::OneDark => ThemeColors::ONE_DARK,
        }
    }

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
