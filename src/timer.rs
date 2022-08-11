pub struct Timer {
    start: Option<std::time::Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Self { start: None }
    }

    pub fn progress(&mut self, now: std::time::Instant) -> std::time::Duration {
        if let Some(start) = self.start {
            let duration = now - start;
            self.start = Some(now);
            duration
        } else {
            // Start the timer
            self.start = Some(now);
            std::time::Duration::new(0, 0)
        }
    }

    pub fn reset(&mut self) {
        self.start = None;
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
