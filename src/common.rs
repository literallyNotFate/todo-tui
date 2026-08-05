use crate::theme::{BuiltinTheme, ThemeId};

/// Helper function to skip serialization if default value
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Helper function to skip serialization if last dark theme is default (GruvboxDark)
pub fn is_default_dark(opt: &Option<ThemeId>) -> bool {
    match opt {
        Some(ThemeId::Builtin(name)) => *name == BuiltinTheme::default(),
        Some(ThemeId::Custom(_)) | Some(ThemeId::System(_)) => false,
        None => true,
    }
}

/// Helper function to skip serialization if last dark theme is default (GruvboxLight)
pub fn is_default_light(opt: &Option<ThemeId>) -> bool {
    match opt {
        Some(ThemeId::Builtin(name)) => *name == BuiltinTheme::GruvboxLight,
        Some(ThemeId::Custom(_)) | Some(ThemeId::System(_)) => false,
        None => true,
    }
}

/// Helper function to tell that bool default is true
pub fn default_bool_is_true() -> bool {
    true
}
