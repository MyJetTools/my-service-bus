use std::sync::{
    atomic::{AtomicI32, AtomicU8, Ordering},
    Arc,
};

use my_service_bus::tcp_contracts::{MySbTcpConnection, PacketProtVer};

use crate::sessions::ConnectionMetricsSnapshot;

pub struct TcpConnectionData {
    pub connection: Arc<MySbTcpConnection>,
    protocol_version: i32,
    delivery_packet_version: AtomicU8,
    pub name: String,
    pub version: Option<String>,
    pub logged_send_error_on_disconnected: AtomicI32,
}

impl TcpConnectionData {
    pub fn new(
        connection: Arc<MySbTcpConnection>,
        name: String,
        version: Option<String>,
        protocol_version: i32,
    ) -> Self {
        Self {
            connection,
            protocol_version: protocol_version,
            delivery_packet_version: AtomicU8::new(0),
            logged_send_error_on_disconnected: AtomicI32::new(0),
            name,
            version,
        }
    }

    pub fn update_deliver_message_packet_version(&self, value: u8) {
        self.delivery_packet_version.store(value, Ordering::SeqCst);
    }

    pub fn get_protocol_version(&self) -> i32 {
        self.protocol_version
    }

    pub fn get_messages_to_deliver_protocol_version(&self) -> PacketProtVer {
        let protocol_version = self.get_protocol_version();
        if protocol_version == 0 {
            panic!("Protocol version is not initialized");
        }
        let packet_version = self.delivery_packet_version.load(Ordering::Relaxed);

        PacketProtVer {
            tcp_protocol_version: protocol_version.into(),
            packet_version,
        }
    }

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        ConnectionMetricsSnapshot {
            read: self
                .connection
                .statistics()
                .total_received
                .load(Ordering::SeqCst),
            written: self
                .connection
                .statistics()
                .total_sent
                .load(Ordering::SeqCst),
            read_per_sec: self.connection.statistics().received_per_sec.get_value(),
            written_per_sec: self.connection.statistics().sent_per_sec.get_value(),
            last_incoming_moment: self
                .connection
                .statistics()
                .last_receive_moment
                .as_date_time(),
        }
    }
}
