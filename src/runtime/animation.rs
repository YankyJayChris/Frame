//! Animation support for the Frame runtime.
//!
//! Full implementation with easing curves: Task 9+.

use std::time::{Duration, Instant};

pub struct Animation {
    pub kind: String,
    pub duration: u32,
    start: Instant,
    duration_ms: Duration,
}

impl Animation {
    pub fn new(kind: &str, duration_ms: u32) -> Self {
        Animation {
            kind: kind.to_string(),
            duration: duration_ms,
            start: Instant::now(),
            duration_ms: Duration::from_millis(duration_ms as u64),
        }
    }

    pub fn value(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let total = self.duration_ms.as_secs_f32();
        if total == 0.0 { return 1.0; }
        (elapsed / total).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.start.elapsed() >= self.duration_ms
    }
}
