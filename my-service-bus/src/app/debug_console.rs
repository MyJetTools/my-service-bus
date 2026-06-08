use std::collections::VecDeque;
use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use rust_extensions::date_time::DateTimeAsMicroseconds;

const MAX_DEBUG_RECORDS: usize = 1000;

#[derive(Clone)]
pub struct DebugConsoleRecord {
    pub unix_microseconds: i64,
    pub message: String,
}

impl DebugConsoleRecord {
    /// Event date as a human-readable RFC3339 string.
    pub fn date_rfc3339(&self) -> String {
        DateTimeAsMicroseconds::new(self.unix_microseconds).to_rfc3339()
    }
}

#[derive(Clone)]
pub struct DebugConsoleTarget {
    pub topic_id: String,
    /// When None - every queue of the topic is traced.
    pub queue_id: Option<String>,
}

/// In-memory ring buffer for ad-hoc debug tracing. Holds up to `MAX_DEBUG_RECORDS`
/// records (oldest dropped first). What gets traced is chosen at runtime (via the
/// /api/Debug/Console/Target endpoint) - not hardcoded. Written from anywhere that has
/// the `AppContext`, read out over MCP (mysb_get_debug_console) instead of the main stdout.
pub struct DebugConsole {
    records: Mutex<VecDeque<DebugConsoleRecord>>,
    // read-mostly: checked on the hot delivery path, changed rarely -> ArcSwap (lock-free read)
    target: ArcSwap<Option<DebugConsoleTarget>>,
}

impl DebugConsole {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(VecDeque::with_capacity(MAX_DEBUG_RECORDS)),
            target: ArcSwap::from_pointee(None),
        }
    }

    /// Selects what to trace. Passing queue_id = None traces every queue of the topic.
    /// Clears the buffer so a fresh capture starts.
    pub fn set_target(&self, topic_id: String, queue_id: Option<String>) {
        self.target
            .store(Arc::new(Some(DebugConsoleTarget { topic_id, queue_id })));
        self.clear();
    }

    /// Turns tracing off (nothing is recorded anymore).
    pub fn disable(&self) {
        self.target.store(Arc::new(None));
    }

    pub fn get_target(&self) -> Option<DebugConsoleTarget> {
        (**self.target.load()).clone()
    }

    /// Hot-path check: should we trace this exact (topic, queue)?
    pub fn matches(&self, topic_id: &str, queue_id: &str) -> bool {
        let guard = self.target.load();
        match &**guard {
            Some(target) => {
                if target.topic_id != topic_id {
                    return false;
                }
                match &target.queue_id {
                    Some(q) => q == queue_id,
                    None => true,
                }
            }
            None => false,
        }
    }

    /// Topic-level check (used where the queue is not known yet).
    pub fn matches_topic(&self, topic_id: &str) -> bool {
        let guard = self.target.load();
        match &**guard {
            Some(target) => target.topic_id == topic_id,
            None => false,
        }
    }

    pub fn write(&self, message: impl Into<String>) {
        let record = DebugConsoleRecord {
            unix_microseconds: DateTimeAsMicroseconds::now().unix_microseconds,
            message: message.into(),
        };

        let mut records = self.records.lock();
        while records.len() >= MAX_DEBUG_RECORDS {
            records.pop_front();
        }
        records.push_back(record);
    }

    /// Returns records oldest-first. When `tail` is set, returns only the last `tail` records.
    pub fn get_records(&self, tail: Option<usize>) -> Vec<DebugConsoleRecord> {
        let records = self.records.lock();
        match tail {
            Some(tail) if tail < records.len() => {
                records.iter().skip(records.len() - tail).cloned().collect()
            }
            _ => records.iter().cloned().collect(),
        }
    }

    pub fn clear(&self) {
        self.records.lock().clear();
    }
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self::new()
    }
}
