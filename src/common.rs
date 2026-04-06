use crate::theme::{ThemeID, ThemeName};

/// Helper function to skip serialization if default value
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Helper function to skip serialization if last dark theme is default (GruvboxDark)
pub fn is_default_dark(opt: &Option<ThemeID>) -> bool {
    match opt {
        Some(ThemeID::Builtin(name)) => *name == ThemeName::default(),
        Some(ThemeID::Custom(_)) => false,
        None => true,
    }
}

/// Helper function to skip serialization if last dark theme is default (GruvboxLight)
pub fn is_default_light(opt: &Option<ThemeID>) -> bool {
    match opt {
        Some(ThemeID::Builtin(name)) => *name == ThemeName::GruvboxLight,
        Some(ThemeID::Custom(_)) => false,
        None => true,
    }
}

/// Helper function to tell that bool default is true
pub fn default_bool_is_true() -> bool {
    true
}
