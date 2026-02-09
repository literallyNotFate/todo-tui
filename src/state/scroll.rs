/// Simple scroll state (for task description)
#[derive(Debug, Default)]
pub struct AdaptiveScroll {
    pub current: u16,
    pub max_scroll: u16,
}

impl AdaptiveScroll {
    pub fn scroll_down(&mut self) {
        if self.current < self.max_scroll {
            self.current = self.current.saturating_add(1);
        }
    }

    pub fn scroll_up(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    pub fn reset(&mut self) {
        self.current = 0;
        self.max_scroll = 0;
    }
}

/// Unit-tests for adaptive scroll
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_scroll_down_limit() {
        let mut scroll: AdaptiveScroll = AdaptiveScroll {
            current: 0,
            max_scroll: 2,
        };

        scroll.scroll_down();
        assert_eq!(scroll.current, 1);

        scroll.scroll_down();
        assert_eq!(scroll.current, 2);

        scroll.scroll_down();
        assert_eq!(
            scroll.current, 2,
            "Current scroll value cannot be more that max_scroll value"
        );
    }

    #[test]
    fn should_test_scroll_up_limit() {
        let mut scroll: AdaptiveScroll = AdaptiveScroll {
            current: 1,
            max_scroll: 10,
        };

        scroll.scroll_up();
        assert_eq!(scroll.current, 0);

        scroll.scroll_up();
        assert_eq!(scroll.current, 0, "Scroll cannot be less than 0");
    }

    #[test]
    fn should_handle_scroll_reset() {
        let mut scroll: AdaptiveScroll = AdaptiveScroll {
            current: 5,
            max_scroll: 10,
        };

        scroll.reset();
        assert_eq!(scroll.current, 0);
        assert_eq!(scroll.max_scroll, 0);
    }
}
