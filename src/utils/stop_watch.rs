use std::time::{Duration, SystemTime};

use super::duration_to_string;

pub struct StopWatch {
    start_time: SystemTime,
    stop_time: SystemTime,
}

//Bug - negative duration;
impl StopWatch {
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            start_time: now,
            stop_time: now,
        }
    }

    pub fn start(&mut self) {
        self.start_time = SystemTime::now()
    }

    pub fn pause(&mut self) {
        self.stop_time = SystemTime::now()
    }

    pub fn duration(&self) -> Duration {
        self.stop_time.duration_since(self.start_time).unwrap()
    }

    pub fn duration_as_string(&self) -> String {
        let duration = self.duration();
        duration_to_string(duration)
    }
}
