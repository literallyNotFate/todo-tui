use crate::theme::ThemeName;

/// Helper function to skip serialization if default value
pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

/// Helper function to skip serialization if last dark theme is default (GruvboxDark)
pub fn is_default_dark(opt: &Option<ThemeName>) -> bool {
    opt.as_ref() == Some(&ThemeName::default()) || opt.is_none()
}

/// Helper function to skip serialization if last dark theme is default (GruvboxLight)
pub fn is_default_light(opt: &Option<ThemeName>) -> bool {
    opt.as_ref() == Some(&ThemeName::GruvboxLight) || opt.is_none()
}

/// Helper function to tell that bool default is true
pub fn default_bool_is_true() -> bool {
    true
}
