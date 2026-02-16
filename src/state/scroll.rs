use std::cell::Cell;

/// Simple scroll state (for task description)
#[derive(Debug, Default)]
pub struct AdaptiveScroll {
    pub current: Cell<u16>,
    pub max_scroll: Cell<u16>,
}

impl AdaptiveScroll {
    pub fn scroll_down(&self) {
        self.current.set(self.current.get().saturating_add(1));
    }

    pub fn scroll_up(&self) {
        self.current.set(self.current.get().saturating_sub(1));
    }

    pub fn reset(&self) {
        self.current.set(0);
        self.max_scroll.set(0);
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
