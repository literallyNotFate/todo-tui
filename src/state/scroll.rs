use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default, Clone)]
pub struct AdaptiveScroll {
    pub current: Rc<Cell<u16>>,
    pub max_scroll: Rc<Cell<u16>>,
}

impl AdaptiveScroll {
    pub fn with_position(pos: u16) -> Self {
        Self {
            current: Rc::new(Cell::new(pos)),
            max_scroll: Rc::new(Cell::new(0)),
        }
    }

    pub fn scroll_down(&self) {
        let next = self.current.get().saturating_add(1);
        let max = self.max_scroll.get();
        if next <= max || max == 0 {
            self.current.set(next);
        }
    }

    pub fn scroll_up(&self) {
        self.current.set(self.current.get().saturating_sub(1));
    }

    pub fn reset(&self) {
        self.current.set(0);
        self.max_scroll.set(0);
    }

    pub fn get_current(&self) -> u16 {
        self.current.get()
    }
}

/// Unit-tests for adaptive scroll
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_scrolling() {
        let scroll = AdaptiveScroll::default();
        scroll.max_scroll.set(10);

        scroll.scroll_down();
        assert_eq!(scroll.current.get(), 1);

        scroll.scroll_down();
        assert_eq!(scroll.current.get(), 2);

        scroll.scroll_up();
        assert_eq!(scroll.current.get(), 1);
    }

    #[test]
    fn should_handle_scroll_reset() {
        let scroll = AdaptiveScroll::default();
        scroll.current.set(5);
        scroll.max_scroll.set(10);

        scroll.reset();
        assert_eq!(scroll.current.get(), 0);
        assert_eq!(scroll.max_scroll.get(), 0);
    }
}
