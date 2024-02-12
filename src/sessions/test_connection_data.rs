use my_service_bus::tcp_contracts::MySbTcpContract;
use rust_extensions::StrOrString;
use tokio::sync::Mutex;

use super::SessionId;

pub struct TestConnectionData {
    pub id: SessionId,
    pub ip: StrOrString<'static>,
    connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<MySbTcpContract>>,
    pub name: String,
    pub version: Option<String>,
}

impl TestConnectionData {
    pub fn new(id: SessionId, ip: impl Into<StrOrString<'static>>) -> Self {
        Self {
            id,
            ip: ip.into(),
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_packets: Mutex::new(vec![]),
            name: "Test".to_string(),
            version: None,
        }
    }

    pub async fn send_packet(&self, tcp_contract: MySbTcpContract) {
        let mut write_access = self.sent_packets.lock().await;
        write_access.push(tcp_contract);
    }

    pub async fn get_list_of_packets_and_clear_them(&self) -> Vec<MySbTcpContract> {
        let mut write_access = self.sent_packets.lock().await;
        let mut result = Vec::new();
        std::mem::swap(&mut *write_access, &mut result);
        result
    }

    pub fn disconnect(&self) -> bool {
        self.connected
            .swap(false, std::sync::atomic::Ordering::Relaxed)
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(std::sync::atomic::Ordering::Relaxed)
    }
}
