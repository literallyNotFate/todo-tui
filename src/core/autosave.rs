use std::time::{Duration, Instant};

/// Autosave struct for Application with debounced activity
pub struct Autosave {
    pub enabled: bool,
    pub interval: Duration,
    pub debounce: Duration,
    last_save: Instant,
    last_activity: Instant,
}

impl Autosave {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            interval: Duration::from_secs(30),
            debounce: Duration::from_secs(5),
            last_save: Instant::now(),
            last_activity: Instant::now(),
        }
    }

    /// Toggle autosave enabled
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled
    }

    /// Updating the time of the last activity (called on key press)
    pub fn register_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check whether the state should be saved
    pub fn should_save(&self, has_changes: bool) -> bool {
        self.enabled
            && has_changes
            && self.last_save.elapsed() >= self.interval
            && self.last_activity.elapsed() >= self.debounce
    }

    /// Reseting timer after success save
    pub fn reset_timer(&mut self) {
        self.last_save = Instant::now();
    }
}

/// Unit-tests for autosave
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_autosave_toggle() {
        let mut as_logic = Autosave::new(false);
        assert!(!as_logic.enabled);
        as_logic.toggle_enabled();
        assert!(as_logic.enabled);
    }

    #[test]
    fn should_handle_save_conditions() {
        let mut as_logic = Autosave::new(true);
        assert!(!as_logic.should_save(false));

        assert!(!as_logic.should_save(true));

        as_logic.enabled = false;
        assert!(!as_logic.should_save(true));
    }
}
