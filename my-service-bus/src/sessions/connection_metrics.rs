use std::sync::atomic::{AtomicUsize, Ordering};

use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

#[derive(Debug, Clone)]
pub struct ConnectionMetricsSnapshot {
    pub read: usize,
    pub written: usize,
    pub read_per_sec: usize,
    pub written_per_sec: usize,
    pub last_incoming_moment: DateTimeAsMicroseconds,
}

impl Default for ConnectionMetricsSnapshot {
    fn default() -> Self {
        Self {
            read: 0,
            written: 0,
            read_per_sec: 0,
            written_per_sec: 0,
            last_incoming_moment: DateTimeAsMicroseconds::now(),
        }
    }
}

pub struct ConnectionMetrics {
    pub read: AtomicUsize,
    pub written: AtomicUsize,
    read_per_sec_int: AtomicUsize,
    pub read_per_sec: AtomicUsize,
    written_per_sec_int: AtomicUsize,
    pub written_per_sec: AtomicUsize,
    pub last_incoming_moment: AtomicDateTimeAsMicroseconds,
}

impl ConnectionMetrics {
    pub fn new() -> Self {
        Self {
            read: AtomicUsize::new(0),
            written: AtomicUsize::new(0),
            read_per_sec: AtomicUsize::new(0),
            written_per_sec: AtomicUsize::new(0),
            written_per_sec_int: AtomicUsize::new(0),
            read_per_sec_int: AtomicUsize::new(0),
            last_incoming_moment: AtomicDateTimeAsMicroseconds::now(),
        }
    }

    pub fn add_read(&self, value: usize) {
        self.read.fetch_add(value, Ordering::SeqCst);
        self.read_per_sec_int.fetch_add(value, Ordering::SeqCst);
    }

    pub fn add_written(&self, value: usize) {
        self.read.fetch_add(value, Ordering::SeqCst);
        self.written_per_sec_int.fetch_add(value, Ordering::SeqCst);
        self.last_incoming_moment
            .update(DateTimeAsMicroseconds::now());
    }

    pub fn one_second_tick(&self) {
        let read_per_sec = self.read_per_sec_int.swap(0, Ordering::SeqCst);
        self.read_per_sec.store(read_per_sec, Ordering::SeqCst);
        let written_per_sec = self.written_per_sec_int.swap(0, Ordering::SeqCst);
        self.written_per_sec
            .store(written_per_sec, Ordering::SeqCst);
    }

    pub fn get_snapshot(&self) -> ConnectionMetricsSnapshot {
        ConnectionMetricsSnapshot {
            read: self.read.load(Ordering::SeqCst),
            written: self.written.load(Ordering::SeqCst),
            read_per_sec: self.read_per_sec.load(Ordering::SeqCst),
            written_per_sec: self.written_per_sec.load(Ordering::SeqCst),
            last_incoming_moment: self.last_incoming_moment.as_date_time(),
        }
    }
}
