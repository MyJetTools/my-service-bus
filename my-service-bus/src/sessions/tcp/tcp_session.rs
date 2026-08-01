use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};

use arc_swap::ArcSwap;
use my_service_bus::tcp_contracts::{MySbTcpConnection, PacketProtVer};
use rust_extensions::sorted_vec::EntityWithKey;

use crate::{
    namespaces::Namespace,
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
    /// Namespace this connection works in. A connection starts in the default one
    /// — which is where every pre-namespace client stays — and the `SetNamespace`
    /// packet moves it before the first publish or subscribe.
    namespace: ArcSwap<Namespace>,
    /// Raised by the first publish or subscribe. From that moment the namespace
    /// can no longer change: the connection already holds subscriptions and
    /// delivery cursors of the namespace it is in, and confirmations name only a
    /// topic and a queue, so swapping it underneath would misroute them.
    namespace_locked: AtomicBool,
}

impl MyServiceBusTcpSession {
    pub fn new(
        session_id: SessionId,
        connection: Arc<MySbTcpConnection>,
        name: String,
        version: Option<String>,
        env_info: Option<String>,
        protocol_version: i32,
        namespace: Arc<Namespace>,
    ) -> Self {
        Self {
            session_id,
            connection,
            protocol_version: protocol_version,
            delivery_packet_version: AtomicU8::new(0),
            name,
            version,
            env_info,
            namespace: ArcSwap::new(namespace),
            namespace_locked: AtomicBool::new(false),
        }
    }

    pub fn get_namespace(&self) -> Arc<Namespace> {
        self.namespace.load_full()
    }

    /// Called by the operations that make the namespace observable — publish and
    /// subscribe. After this the connection is pinned to its namespace.
    pub fn lock_namespace(&self) {
        self.namespace_locked.store(true, Ordering::SeqCst);
    }

    pub fn set_namespace(&self, namespace: Arc<Namespace>) -> Result<(), String> {
        // Re-stating the namespace it already has is not a change, so it stays
        // allowed even after the connection started working.
        if self.get_namespace().name == namespace.name {
            return Ok(());
        }

        if self.namespace_locked.load(Ordering::SeqCst) {
            return Err(format!(
                "Namespace can not be changed to '{}' after the connection has published or subscribed",
                namespace.name
            ));
        }

        self.namespace.store(namespace);

        Ok(())
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
