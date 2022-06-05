use my_service_bus_tcp_shared::TcpContract;
use tokio::sync::Mutex;

use super::SessionId;

pub struct TestConnectionData {
    pub id: SessionId,
    pub ip: String,
    pub connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<TcpContract>>,
}

impl TestConnectionData {
    pub fn new(id: SessionId, ip: &str) -> Self {
        Self {
            id,
            ip: ip.to_string(),
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_packets: Mutex::new(vec![]),
        }
    }

    pub async fn send_packet(&self, tcp_contract: TcpContract) {
        let mut write_access = self.sent_packets.lock().await;
        write_access.push(tcp_contract);
    }

    pub async fn get_list_of_packets_and_clear_them(&self) -> Vec<TcpContract> {
        let mut write_access = self.sent_packets.lock().await;
        let mut result = Vec::new();
        std::mem::swap(&mut *write_access, &mut result);
        result
    }
}
