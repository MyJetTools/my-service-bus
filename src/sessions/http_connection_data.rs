use std::sync::atomic::{AtomicBool, Ordering};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::sessions::{ConnectionMetrics, ConnectionMetricsSnapshot};

pub struct HttpConnectionData {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ip: String,
    pub connected_moment: DateTimeAsMicroseconds,
    connection_metrics: ConnectionMetrics,
    connected: AtomicBool,
}

impl HttpConnectionData {
    pub fn new(id: String, name: String, version: String, ip: String) -> Self {
        Self {
            id,
            name,
            version,
            ip,
            connected: AtomicBool::new(true),
            connection_metrics: ConnectionMetrics::new(),
            connected_moment: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn ping(&self) {
        self.connection_metrics.add_written(1);
        self.connection_metrics.add_read(1);
    }

    pub fn update_written_amount(&self, amount: usize) {
        self.connection_metrics.add_written(amount);
    }

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        return self.connection_metrics.get_snapshot();
    }

    pub fn one_second_tick(&self) {
        self.connection_metrics.one_second_tick();
    }

    pub fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        self.connection_metrics.last_incoming_moment.as_date_time()
    }

    pub fn disconnect(&self) -> bool {
        self.connected.swap(false, Ordering::Relaxed)
    }

    #[cfg(test)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}
