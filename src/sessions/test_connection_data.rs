use rust_extensions::StrOrString;
use tokio::sync::Mutex;

use crate::{
    http::controllers::MessageToDeliverHttpContract, queue_subscribers::SubscriberId,
    queues::QueueId, topics::TopicId,
};

use super::SessionId;

pub struct TestDeliveryMessage {
    pub topic_id: TopicId,
    pub queue_id: QueueId,
    pub subscriber_id: SubscriberId,
    pub messages: Vec<MessageToDeliverHttpContract>,
}

pub struct TestConnectionData {
    pub id: SessionId,
    pub ip: StrOrString<'static>,
    connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<TestDeliveryMessage>>,
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

    pub async fn send_messages(
        &self,
        topic_id: TopicId,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
        messages: Vec<MessageToDeliverHttpContract>,
    ) {
        let mut write_access = self.sent_packets.lock().await;
        write_access.push(TestDeliveryMessage {
            topic_id,
            queue_id,
            subscriber_id,
            messages,
        });
    }

    pub async fn get_list_of_packets_and_clear_them(&self) -> Vec<TestDeliveryMessage> {
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
