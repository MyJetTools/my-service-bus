use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

use my_service_bus::tcp_contracts::{MySbTcpSerializer, PacketProtVer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;
use tokio::sync::RwLock;

use crate::sessions::ConnectionMetricsSnapshot;

#[derive(Debug, Clone)]
pub struct TcpConnectionAttributes {
    pub name: Option<String>,
    pub version: Option<String>,
}

pub struct TcpConnectionData {
    pub connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
    protocol_version: AtomicI32,
    delivery_packet_version: AtomicI32,
    attr: RwLock<TcpConnectionAttributes>,
    pub logged_send_error_on_disconnected: AtomicI32,
}

impl TcpConnectionData {
    pub fn new(connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>) -> Self {
        let attr = TcpConnectionAttributes {
            name: None,
            version: None,
        };

        Self {
            connection,
            protocol_version: AtomicI32::new(0),
            delivery_packet_version: AtomicI32::new(0),
            logged_send_error_on_disconnected: AtomicI32::new(0),
            attr: RwLock::new(attr),
        }
    }

    pub async fn set_socket_name(&self, name: String, version: Option<String>) {
        let mut write_access = self.attr.write().await;

        write_access.name = Some(name);
        write_access.version = version;
    }

    pub async fn get_attrs(&self) -> TcpConnectionAttributes {
        let read_access = self.attr.read().await;
        read_access.clone()
    }

    pub fn update_protocol_version(&self, value: i32) {
        self.protocol_version.store(value, Ordering::SeqCst);
    }

    pub fn update_deliver_message_packet_version(&self, value: i32) {
        self.delivery_packet_version.store(value, Ordering::SeqCst);
    }

    pub fn get_protocol_version(&self) -> i32 {
        self.protocol_version.load(Ordering::Relaxed)
    }

    pub fn get_messages_to_deliver_protocol_version(&self) -> PacketProtVer {
        let protocol_version = self.get_protocol_version();
        if protocol_version == 0 {
            panic!("Protocol version is not initialized");
        }
        let packet_version = self.delivery_packet_version.load(Ordering::Relaxed);

        PacketProtVer {
            protocol_version,
            packet_version,
        }
    }

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        ConnectionMetricsSnapshot {
            read: self
                .connection
                .statistics
                .total_received
                .load(Ordering::SeqCst),
            written: self.connection.statistics.total_sent.load(Ordering::SeqCst),
            read_per_sec: self.connection.statistics.received_per_sec.get_value(),
            written_per_sec: self.connection.statistics.sent_per_sec.get_value(),
            last_incoming_moment: self
                .connection
                .statistics
                .last_receive_moment
                .as_date_time(),
        }
    }
}
