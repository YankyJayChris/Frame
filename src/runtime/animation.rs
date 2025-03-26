use std::time::{Duration, Instant};

// Animation system
pub struct Animation {
    start: Instant,
    duration: Duration,
    from: f32,
    to: f32,
    target: String, // e.g., "x", "opacity"
}

impl Animation {
    pub fn new(target: &str, from: f32, to: f32, duration: Duration) -> Self {
        Animation {
            start: Instant::now(),
            duration,
            from,
            to,
            target: target.to_string(),
        }
    }

    pub fn value(&self) -> f32 {
        let elapsed = self.start.elapsed().as_secs_f32();
        let t = (elapsed / self.duration.as_secs_f32()).min(1.0);
        self.from + (self.to - self.from) * t
    }

    pub fn is_complete(&self) -> bool {
        self.start.elapsed() >= self.duration
    }
}