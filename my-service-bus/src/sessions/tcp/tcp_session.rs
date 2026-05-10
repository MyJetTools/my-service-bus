use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

use my_service_bus::tcp_contracts::{MySbTcpConnection, PacketProtVer};
use rust_extensions::sorted_vec::EntityWithKey;

use crate::{
    operations::delivery::SubscriberPackageBuilder,
    sessions::{my_sb_session::*, ConnectionMetricsSnapshot, SessionId},
};

pub struct MyServiceBusTcpSession {
    pub session_id: SessionId,
    pub connection: Arc<MySbTcpConnection>,
    protocol_version: i32,
    delivery_packet_version: AtomicU8,
    pub name: String,
    pub version: Option<String>,
    pub env_info: Option<String>,
}

impl MyServiceBusTcpSession {
    pub fn new(
        session_id: SessionId,
        connection: Arc<MySbTcpConnection>,
        name: String,
        version: Option<String>,
        env_info: Option<String>,
        protocol_version: i32,
    ) -> Self {
        Self {
            session_id,
            connection,
            protocol_version: protocol_version,
            delivery_packet_version: AtomicU8::new(0),
            name,
            version,
            env_info,
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

    pub fn get_name_and_version(&self) -> SessionNameAndVersion {
        SessionNameAndVersion {
            name: self.name.to_string(),
            version: self.version.clone(),
            env_info: self.env_info.clone(),
        }
    }

    pub fn get_metrics(&self) -> SessionMetrics {
        let statistics = self.connection.statistics();
        SessionMetrics {
            ip: if let Some(addr) = &self.connection.addr {
                addr.to_string()
            } else {
                "???".to_string()
            },
            connected: statistics.connected,
            connection_metrics: ConnectionMetricsSnapshot {
                read: statistics.total_received.load(Ordering::Relaxed),
                written: statistics.total_sent.load(Ordering::Relaxed),
                read_per_sec: statistics.received_per_sec.get_value(),
                written_per_sec: statistics.sent_per_sec.get_value(),
                last_incoming_moment: statistics.last_receive_moment.as_date_time(),
            },
            tcp_protocol_version: Some(self.protocol_version),
        }
    }

    pub fn send_messages_to_connection(&self, package_builder: SubscriberPackageBuilder) {
        let messages = package_builder.get_tcp_result();
        let connection = self.connection.clone();
        connection.send(&messages);
    }

    pub async fn disconnect(&self) -> bool {
        self.connection.disconnect().await
    }
}

impl EntityWithKey<i32> for MyServiceBusTcpSession {
    fn get_key(&self) -> &i32 {
        &self.connection.id
    }
}
