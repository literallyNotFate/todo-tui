use crate::{
    theme::{BuiltinTheme, PaletteDisk, SystemTheme, ThemeId, ThemePalette},
    ui::{
        Field, FieldType, Form,
        widgets::{SelectInput, input::Input},
    },
};
use strum::IntoEnumIterator;

/// Theme switcher form
pub struct ThemeFormContext {
    pub form: Form,
}

impl ThemeFormContext {
    /// Create form for switching themes with current theme selected
    pub fn new(current_theme: &ThemeId) -> Self {
        let mut all_themes: Vec<ThemeId> = Vec::new();
        for builtin in BuiltinTheme::iter() {
            all_themes.push(ThemeId::Builtin(builtin));
        }
        for system in SystemTheme::iter() {
            all_themes.push(ThemeId::System(system));
        }

        let external_themes: Vec<(String, ThemePalette)> = PaletteDisk::load_all();
        for (name, _) in external_themes {
            all_themes.push(ThemeId::Custom(name.clone()));
        }

        Self {
            form: Form::new(vec![Field::new(
                "theme",
                FieldType::Select {
                    input: SelectInput::from(all_themes, current_theme).title(" Theme "),
                },
            )]),
        }
    }

    /// Extract selected ThemeId from the form fields
    pub fn get_theme_value(&self) -> ThemeId {
        self.form
            .fields
            .iter()
            .find(|f| f.name == "theme")
            .and_then(|f| match &f.field_type {
                FieldType::Select { input } => input.items.get(input.selected_index).cloned(),
                _ => None,
            })
            .unwrap_or_else(|| ThemeId::Builtin(BuiltinTheme::default()))
    }

    /// Parse data mapping from form fields
    pub fn parse_data(&self) -> ThemeId {
        self.get_theme_value()
    }
}

/// Unit-tests for theme form
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_initialize_theme_form_with_current_theme() {
        let current_theme = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        let context = ThemeFormContext::new(&current_theme);

        assert_eq!(context.form.fields.len(), 1);
        assert_eq!(context.form.fields[0].name, "theme");
        assert_eq!(context.get_theme_value(), current_theme);
    }

    #[test]
    fn should_parse_and_extract_selected_theme_correctly() {
        let initial_theme = ThemeId::Builtin(BuiltinTheme::GruvboxDark);
        let mut context = ThemeFormContext::new(&initial_theme);

        if let FieldType::Select { input } = &mut context.form.fields[0].field_type {
            if let Some(pos) = input
                .items
                .iter()
                .position(|t| *t == ThemeId::System(SystemTheme::Tty))
            {
                input.selected_index = pos;
            }
        }

        let selected_theme = context.parse_data();
        assert_eq!(selected_theme, ThemeId::System(SystemTheme::Tty));
    }
}
