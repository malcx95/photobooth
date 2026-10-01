use std::time::Instant;

pub struct Timer {
    start: Instant,
    interval_ms: u128
}

impl Timer {
    pub fn new(milliseconds: u128) -> Self {
        Timer { start: Instant::now(), interval_ms: milliseconds }
    }

    pub fn has_elapsed(&self) -> bool {
        let elapsed = Instant::now() - self.start;
        (elapsed.as_millis() as u128) >= self.interval_ms
    }

    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    pub fn triggered(&mut self) -> bool {
        if self.has_elapsed() {
            self.reset();
            true
        } else {
            false
        }
    }

    pub fn progress(&self) -> f32 {
        if self.has_elapsed() {
            1.0
        } else {
            let elapsed = (Instant::now() - self.start).as_millis() as f32;
            elapsed / (self.interval_ms as f32)
        }
    }
}
