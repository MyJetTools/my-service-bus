use rust_extensions::{date_time::DateTimeAsMicroseconds, StrOrString};
use tokio::sync::Mutex;

use crate::{
    http::controllers::MessageToDeliverHttpContract,
    operations::delivery::SubscriberPackageBuilder,
    queue_subscribers::SubscriberId,
    queues::QueueId,
    sessions::{my_sb_session::*, ConnectionMetricsSnapshot, MyServiceBusSession, SessionId},
    topics::TopicId,
};

pub struct TestDeliveryMessage {
    pub topic_id: TopicId,
    pub queue_id: QueueId,
    pub subscriber_id: SubscriberId,
    pub _messages: Vec<MessageToDeliverHttpContract>,
}

pub struct MyServiceBusTestSession {
    pub session_id: SessionId,
    pub _ip: StrOrString<'static>,
    connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<TestDeliveryMessage>>,
    pub name: String,
    pub _version: Option<String>,
    pub connected_moment: DateTimeAsMicroseconds,
}

impl MyServiceBusTestSession {
    pub fn new(session_id: SessionId, ip: impl Into<StrOrString<'static>>) -> Self {
        Self {
            session_id,
            _ip: ip.into(),
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_packets: Mutex::new(vec![]),
            name: "Test".to_string(),
            _version: None,
            connected_moment: DateTimeAsMicroseconds::now(),
        }
    }

    pub async fn get_list_of_packets_and_clear_them(&self) -> Vec<TestDeliveryMessage> {
        let mut write_access = self.sent_packets.lock().await;
        let mut result = Vec::new();
        std::mem::swap(&mut *write_access, &mut result);
        result
    }
}

#[async_trait::async_trait]
impl MyServiceBusSession for MyServiceBusTestSession {
    fn get_session_type(&self) -> SessionType {
        SessionType::Test
    }

    fn get_session_id(&self) -> crate::sessions::SessionId {
        self.session_id
    }

    fn get_name_and_version(&self) -> SessionNameAndVersion {
        SessionNameAndVersion {
            name: self.name.to_string(),
            version: None,
        }
    }

    fn get_metrics(&self) -> SessionMetrics {
        SessionMetrics {
            ip: "test".to_string(),
            connected: self.connected_moment,
            connection_metrics: ConnectionMetricsSnapshot::default(),
            tcp_protocol_version: None,
        }
    }

    async fn disconnect(&self) -> bool {
        self.connected
            .swap(false, std::sync::atomic::Ordering::SeqCst)
    }

    async fn send_messages_to_connection(&self, mut package_builder: SubscriberPackageBuilder) {
        let messages = package_builder.get_http_result();

        let mut sent_packets = self.sent_packets.lock().await;

        sent_packets.push(TestDeliveryMessage {
            topic_id: package_builder.topic.topic_id.clone(),
            queue_id: package_builder.queue_id.clone(),
            subscriber_id: package_builder.subscriber_id.clone(),
            _messages: messages,
        });
    }
}
