use std::time::{Duration, Instant};

pub struct PausableInstant {
    instant: Instant,
    is_paused: bool,
    stored_duration: Duration,
}

impl PausableInstant {
    pub fn now() -> Self {
        Self {
            instant: Instant::now(),
            is_paused: false,
            stored_duration: Duration::ZERO,
        }
    }

    pub fn elapsed(&self) -> Duration {
        if self.is_paused {
            self.stored_duration
        } else {
            self.stored_duration.saturating_add(self.instant.elapsed())
        }
    }

    pub fn pause(&mut self) {
        self.is_paused = true;

        if let Some(duration) = self.stored_duration.checked_add(self.instant.elapsed()) {
            self.stored_duration = duration;
        } else {
            self.reset();
        }
    }

    pub fn resume(&mut self) {
        self.is_paused = false;

        self.instant = Instant::now();
    }

    fn reset(&mut self) {
        self.instant = Instant::now();
        self.stored_duration = Duration::ZERO;
    }
}
