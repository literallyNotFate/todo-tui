use crate::config::StorageConfig;
use std::time::{Duration, Instant};

/// Autosave struct for Application with debounced activity
pub struct Autosave {
    pub enabled: bool,
    pub interval: Duration,
    pub debounce: Duration,
    pub last_tick_had_changes: bool,
    pub last_tick_secs: u64,

    last_save: Instant,
    last_activity: Instant,
}

impl Autosave {
    /// Create new autosave with 30 second interval
    pub fn new(enabled: bool) -> Self {
        let interval_secs = 30;
        let now: Instant = Instant::now();

        Self {
            enabled,
            interval: Duration::from_secs(interval_secs),
            debounce: Duration::from_secs(5),
            last_tick_had_changes: false,
            last_save: now,
            last_activity: now,
            last_tick_secs: interval_secs,
        }
    }

    /// Setup autosave from config
    pub fn from(config: &StorageConfig) -> Self {
        let interval_secs = config.safe_interval();
        let now: Instant = Instant::now();

        Self {
            enabled: config.autosave_enabled,
            interval: std::time::Duration::from_secs(interval_secs),
            last_tick_secs: interval_secs,
            debounce: Duration::from_secs(5),
            last_tick_had_changes: false,
            last_save: now,
            last_activity: now,
        }
    }

    /// Toggle autosave enabled with time reset
    pub fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        self.reset_timer();
    }

    /// Updating the time of the last activity (called on key press)
    pub fn register_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check whether the state should be saved
    pub fn should_save(&self, has_changes: bool) -> bool {
        if !self.enabled || !has_changes {
            return false;
        }

        let total_time_passed = self.last_save.elapsed() >= self.interval;
        let silence_time_passed = self.last_activity.elapsed() >= self.debounce;

        total_time_passed && silence_time_passed
    }

    /// Returns time left until next automatic save
    pub fn time_until_next_save(&self) -> u64 {
        let elapsed = self.last_save.elapsed();
        if elapsed >= self.interval {
            0
        } else {
            (self.interval - elapsed).as_secs() + 1
        }
    }

    /// Checks whether system is in the debounce phase
    pub fn is_debouncing(&self, has_changes: bool) -> bool {
        if !self.enabled || !has_changes {
            return false;
        }

        self.last_save.elapsed() >= self.interval && self.last_activity.elapsed() < self.debounce
    }

    /// Reseting timer after success save
    pub fn reset_timer(&mut self) {
        let now = Instant::now();
        self.last_save = now;
        self.last_activity = now;
        self.last_tick_secs = self.interval.as_secs();
    }

    /// Tick function for autosave timer. Returns true if redrawing is needed.
    pub fn tick(&mut self, has_changes: bool) -> bool {
        if !self.enabled || !has_changes {
            return false;
        }

        let current_time_left = self.time_until_next_save();
        let is_debouncing = self.is_debouncing(has_changes);

        if current_time_left != self.last_tick_secs {
            self.last_tick_secs = current_time_left;
            return true;
        }

        if is_debouncing {
            return true;
        }

        false
    }
}

/// Unit-tests for autosave
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn should_initialize_with_correct_defaults() {
        let as_logic = Autosave::new(true);
        assert!(as_logic.enabled);
        assert!(!as_logic.last_tick_had_changes);
    }

    #[test]
    fn should_toggle_enabled_with_timer_reset() {
        let mut as_logic = Autosave::new(false);
        let old_time = as_logic.last_save;

        sleep(Duration::from_millis(2));
        as_logic.toggle_enabled();

        assert!(as_logic.enabled);
        assert!(as_logic.last_save > old_time);
        assert!(!as_logic.last_tick_had_changes);
    }

    #[test]
    fn should_handle_save_conditions() {
        let mut as_logic = Autosave::new(true);
        as_logic.interval = Duration::from_millis(10);
        as_logic.debounce = Duration::from_millis(10);

        assert!(!as_logic.should_save(false));
        assert!(!as_logic.should_save(true));

        sleep(Duration::from_millis(15));
        assert!(as_logic.should_save(true));

        as_logic.enabled = false;
        assert!(!as_logic.should_save(true));
    }

    #[test]
    fn should_test_autosave_debouncing_logic() {
        let mut as_logic = Autosave::new(true);
        as_logic.interval = Duration::from_millis(50);
        as_logic.debounce = Duration::from_millis(100);
        as_logic.reset_timer();

        assert!(!as_logic.is_debouncing(true));

        sleep(Duration::from_millis(60));
        as_logic.register_activity();
        assert!(as_logic.is_debouncing(true));

        sleep(Duration::from_millis(110));
        assert!(!as_logic.is_debouncing(true));
    }
}
