use std::sync::Mutex;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    operations::delivery::SubscriberPackageBuilder,
    sessions::{http::HttpDeliveryPackage, my_sb_session::*, ConnectionMetricsSnapshot, SessionId},
};

pub struct MyServiceBusTestSession {
    pub session_id: SessionId,

    connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<HttpDeliveryPackage>>,
    pub name: String,
    pub _version: Option<String>,
    pub connected_moment: DateTimeAsMicroseconds,
}

impl MyServiceBusTestSession {
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_packets: Mutex::new(vec![]),
            name: "Test".to_string(),
            _version: None,
            connected_moment: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn get_list_of_packets_and_clear_them(&self) -> Vec<HttpDeliveryPackage> {
        let mut write_access = self.sent_packets.lock().unwrap();
        let mut result = Vec::new();
        std::mem::swap(&mut *write_access, &mut result);
        result
    }

    pub fn get_metrics(&self) -> SessionMetrics {
        SessionMetrics {
            ip: "test".to_string(),
            connected: self.connected_moment,
            connection_metrics: ConnectionMetricsSnapshot::default(),
            tcp_protocol_version: None,
        }
    }

    pub fn get_name_and_version(&self) -> SessionNameAndVersion {
        SessionNameAndVersion {
            name: self.name.to_string(),
            version: None,
            env_info: None,
        }
    }

    pub fn send_messages_to_connection(&self, package_builder: SubscriberPackageBuilder) {
        let http_delivery_package = package_builder.get_http_result();
        let mut sent_packets = self.sent_packets.lock().unwrap();

        sent_packets.push(http_delivery_package);
    }

    pub fn disconnect(&self) -> bool {
        self.connected
            .swap(false, std::sync::atomic::Ordering::SeqCst)
    }
}
