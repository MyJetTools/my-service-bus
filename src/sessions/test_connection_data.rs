use my_service_bus_shared::MySbMessageContent;
use tokio::sync::Mutex;

use super::SessionId;

pub struct TestConnectionData {
    pub id: SessionId,
    pub ip: String,
    pub connected: std::sync::atomic::AtomicBool,
    pub sent_messages_to_deliver: Mutex<Vec<(i32, MySbMessageContent)>>,
    pub name: Option<String>,
    pub version: Option<String>,
}

impl TestConnectionData {
    pub fn new(id: SessionId, ip: &str) -> Self {
        Self {
            id,
            ip: ip.to_string(),
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_messages_to_deliver: Mutex::new(vec![]),
            name: None,
            version: None,
        }
    }

    pub async fn deliver_messages(&self, msgs: Vec<(i32, &MySbMessageContent)>) {
        let mut write_access = self.sent_messages_to_deliver.lock().await;
        for (id, content) in msgs {
            write_access.push((id, content.clone()));
        }
    }

    pub async fn get_sent_messages_to_deliver(&self) -> Vec<(i32, MySbMessageContent)> {
        let read_access = self.sent_messages_to_deliver.lock().await;
        read_access.clone()
    }
}
