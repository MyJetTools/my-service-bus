use std::sync::atomic::{AtomicBool, Ordering};

use rust_extensions::{
    date_time::DateTimeAsMicroseconds,
    sorted_vec::{EntityWithKey, EntityWithStrKey},
    TaskCompletionAwaiter,
};
use tokio::sync::Mutex;

use crate::{
    operations::delivery::SubscriberPackageBuilder,
    sessions::{my_sb_session::*, ConnectionMetrics, MyServiceBusSession, SessionId},
};

use super::{HttpDeliveryPackage, HttpSessionKey, SendQueueInner};

pub enum MessageToDeliverResult {
    Package(HttpDeliveryPackage),
    Awaiter(TaskCompletionAwaiter<Option<HttpDeliveryPackage>, String>),
}

pub struct MyServiceBusHttpSession {
    pub session_id: SessionId,
    pub session_key: HttpSessionKey,
    pub name: String,
    pub version: String,
    pub ip: String,
    pub connected_moment: DateTimeAsMicroseconds,
    connection_metrics: ConnectionMetrics,
    connected: AtomicBool,
    send_queue: Mutex<SendQueueInner>,
}

impl MyServiceBusHttpSession {
    pub fn new(
        session_id: SessionId,
        session_key: HttpSessionKey,
        name: String,
        version: String,
        ip: String,
    ) -> Self {
        Self {
            session_id,
            session_key,
            name,
            version,
            ip,
            connected: AtomicBool::new(true),
            connection_metrics: ConnectionMetrics::new(),
            connected_moment: DateTimeAsMicroseconds::now(),
            send_queue: Mutex::new(SendQueueInner::new()),
        }
    }

    pub fn ping(&self) {
        self.connection_metrics.add_written(1);
        self.connection_metrics.add_read(1);
    }

    pub fn update_written_amount(&self, amount: usize) {
        self.connection_metrics.add_written(amount);
    }

    pub async fn one_second_tick(&self) {
        self.connection_metrics.one_second_tick();

        let mut awaiter = self.send_queue.lock().await;
        awaiter.ping_awaiter();
    }

    pub fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        self.connection_metrics.last_incoming_moment.as_date_time()
    }

    pub async fn get_messages_to_deliver(&self) -> MessageToDeliverResult {
        let mut write_access = self.send_queue.lock().await;

        if let Some(next_package) = write_access.queue.get_next_package() {
            return MessageToDeliverResult::Package(next_package);
        }

        let awaiter = write_access.engage_awaiter();
        MessageToDeliverResult::Awaiter(awaiter)
    }

    pub async fn get_long_pool_messages(&self) -> Result<Option<HttpDeliveryPackage>, String> {
        match self.get_messages_to_deliver().await {
            MessageToDeliverResult::Package(package) => Ok(Some(package)),
            MessageToDeliverResult::Awaiter(awaiter) => {
                return awaiter.get_result().await;
            }
        }
    }
}

impl EntityWithStrKey for MyServiceBusHttpSession {
    fn get_key(&self) -> &str {
        self.session_key.as_str()
    }
}

impl EntityWithKey<i64> for MyServiceBusHttpSession {
    fn get_key(&self) -> &i64 {
        self.session_id.as_ref()
    }
}

#[async_trait::async_trait]
impl MyServiceBusSession for MyServiceBusHttpSession {
    fn get_session_type(&self) -> SessionType {
        SessionType::Http
    }

    fn get_session_id(&self) -> crate::sessions::SessionId {
        self.session_id
    }

    fn get_name_and_version(&self) -> SessionNameAndVersion {
        SessionNameAndVersion {
            name: self.name.to_string(),
            version: Some(self.version.to_string()),
            env_info: None,
        }
    }

    fn get_metrics(&self) -> SessionMetrics {
        SessionMetrics {
            ip: self.ip.to_string(),
            connected: self.connected_moment,
            connection_metrics: self.connection_metrics.get_snapshot(),
            tcp_protocol_version: None,
        }
    }

    async fn disconnect(&self) -> bool {
        self.connected.swap(false, Ordering::SeqCst)
    }

    async fn send_messages_to_connection(&self, mut package_builder: SubscriberPackageBuilder) {
        let messages = package_builder.get_http_result();
        let mut write_access = self.send_queue.lock().await;

        write_access.queue.enqueue_messages(
            package_builder.topic.topic_id.clone(),
            package_builder.queue_id.clone(),
            package_builder.subscriber_id,
            messages,
        );

        write_access.deliver_message();
    }
}
