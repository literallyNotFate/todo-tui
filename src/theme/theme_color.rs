use ratatui::style::Color;

#[derive(Debug, Default, Clone, Copy)]
pub struct ThemeColors {
    pub name: &'static str,
    pub accent: Color,  // Main color (cursor ">>", border fg on focus)
    pub border: Color,  // Unfocused border color
    pub surface: Color, // Selected background
    pub text_primary: Color,
    pub text_dim: Color,
    pub bg_dim: Color,
    pub modal_bg: Color,
    pub success: Color, // For completed tasks
    pub warning: Color, // Medium priority
    pub error: Color,   // High priority
}

impl ThemeColors {
    pub const GRUVBOX: Self = Self {
        name: "Gruvbox",
        accent: Color::Rgb(250, 189, 47),
        border: Color::Rgb(102, 92, 84),
        surface: Color::Rgb(60, 56, 54),
        text_primary: Color::Rgb(235, 219, 178),
        text_dim: Color::Rgb(168, 153, 132),
        bg_dim: Color::Rgb(40, 40, 40),
        modal_bg: Color::Rgb(29, 32, 33),
        success: Color::Rgb(184, 187, 38),
        warning: Color::Rgb(254, 128, 25),
        error: Color::Rgb(251, 73, 52),
    };

    pub const CATPPUCCIN: Self = Self {
        name: "Catppuccin",
        accent: Color::Rgb(203, 166, 247),
        border: Color::Rgb(88, 91, 112),
        surface: Color::Rgb(49, 50, 68),
        text_primary: Color::Rgb(205, 214, 244),
        text_dim: Color::Rgb(147, 153, 178),
        bg_dim: Color::Rgb(24, 24, 37),
        modal_bg: Color::Rgb(17, 17, 27),
        success: Color::Rgb(166, 227, 161),
        warning: Color::Rgb(250, 227, 176),
        error: Color::Rgb(243, 139, 168),
    };

    pub const TOKYO_NIGHT: Self = Self {
        name: "Tokyo Night",
        accent: Color::Rgb(122, 162, 247),
        border: Color::Rgb(59, 66, 97),
        surface: Color::Rgb(47, 53, 78),
        text_primary: Color::Rgb(169, 177, 214),
        text_dim: Color::Rgb(86, 95, 137),
        bg_dim: Color::Rgb(36, 40, 59),
        modal_bg: Color::Rgb(26, 27, 38),
        success: Color::Rgb(158, 206, 106),
        warning: Color::Rgb(224, 175, 104),
        error: Color::Rgb(247, 118, 118),
    };

    pub const EVERFOREST: Self = Self {
        name: "Everforest",
        accent: Color::Rgb(167, 192, 128),
        border: Color::Rgb(75, 88, 87),
        surface: Color::Rgb(66, 74, 73),
        text_primary: Color::Rgb(211, 198, 170),
        text_dim: Color::Rgb(122, 132, 122),
        bg_dim: Color::Rgb(43, 48, 50),
        modal_bg: Color::Rgb(35, 39, 41),
        success: Color::Rgb(167, 192, 128),
        warning: Color::Rgb(219, 188, 127),
        error: Color::Rgb(230, 126, 128),
    };

    pub const ONE_DARK: Self = Self {
        name: "One Dark",
        accent: Color::Rgb(97, 175, 239),
        border: Color::Rgb(75, 82, 99),
        surface: Color::Rgb(44, 50, 60),
        text_primary: Color::Rgb(171, 178, 191),
        text_dim: Color::Rgb(92, 99, 112),
        bg_dim: Color::Rgb(40, 44, 52),
        modal_bg: Color::Rgb(33, 37, 43),
        success: Color::Rgb(152, 195, 121),
        warning: Color::Rgb(229, 192, 123),
        error: Color::Rgb(224, 108, 117),
    };
}
