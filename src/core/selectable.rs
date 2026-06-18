use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};
use strum::IntoEnumIterator;

/// Container-wrapper for data that can be switched (like enums)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Selectable<T> {
    pub value: T,
}

impl<T> Selectable<T>
where
    T: IntoEnumIterator + Copy + PartialEq + Default + 'static,
{
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Returns current selected index
    pub fn index(&self) -> usize {
        T::iter().position(|t| t == self.value).unwrap_or(0)
    }

    /// Count total elements of enum
    pub fn count(&self) -> usize {
        T::iter().count()
    }

    /// Set specific enum value to selectable
    pub fn set(&mut self, value: T) {
        self.value = value;
    }

    /// Switch to the next element
    pub fn next(&mut self) {
        let mut iter = T::iter();
        let pos = iter.position(|t| t == self.value).unwrap_or(0);
        self.value = T::iter()
            .nth(pos + 1)
            .unwrap_or_else(|| T::iter().next().unwrap());
    }

    /// Switch to the prev element
    pub fn prev(&mut self) {
        let pos = T::iter().position(|t| t == self.value).unwrap_or(0);
        if pos == 0 {
            self.value = T::iter().last().unwrap();
        } else {
            self.value = T::iter().nth(pos - 1).unwrap();
        }
    }
}

impl<T> Deref for Selectable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: PartialEq> PartialEq<T> for Selectable<T> {
    fn eq(&self, other: &T) -> bool {
        self.value == *other
    }
}

impl<T: PartialEq> PartialEq for Selectable<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Selectable<T> {}

impl<T: Default> Default for Selectable<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: Display> Display for Selectable<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Unit-tests for Selectable
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq, Default, strum::EnumIter)]
    enum TestEnum {
        #[default]
        A,
        B,
        C,
    }

    #[test]
    fn should_handle_selectable_rotation() {
        let mut s = Selectable::new(TestEnum::A);
        s.next();
        assert_eq!(s.value, TestEnum::B);
        s.next();
        s.next();
        assert_eq!(s.value, TestEnum::A);
    }
}
