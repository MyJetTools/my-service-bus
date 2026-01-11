use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use rust_extensions::{
    date_time::DateTimeAsMicroseconds, sorted_vec::EntityWithStrKey, TaskCompletionAwaiter,
};
use tokio::sync::Mutex;

use crate::{
    operations::delivery::SubscriberPackageBuilder,
    sessions::{my_sb_session::*, ConnectionMetrics, SessionId},
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
    send_queue: Arc<Mutex<SendQueueInner>>,
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
            send_queue: Arc::new(Mutex::new(SendQueueInner::new())),
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

    pub fn get_name_and_version(&self) -> SessionNameAndVersion {
        SessionNameAndVersion {
            name: self.name.to_string(),
            version: Some(self.version.to_string()),
            env_info: None,
        }
    }

    pub fn get_metrics(&self) -> SessionMetrics {
        SessionMetrics {
            ip: self.ip.to_string(),
            connected: self.connected_moment,
            connection_metrics: self.connection_metrics.get_snapshot(),
            tcp_protocol_version: None,
        }
    }

    pub fn send_messages_to_connection(&self, package_builder: SubscriberPackageBuilder) {
        let send_queue = self.send_queue.clone();

        tokio::spawn(async move {
            let http_delivery_package = package_builder.get_http_result();
            let mut write_access = send_queue.lock().await;

            write_access.queue.enqueue_messages(http_delivery_package);

            write_access.deliver_message();
        });
    }

    pub fn disconnect(&self) -> bool {
        self.connected.swap(false, Ordering::SeqCst)
    }
}

impl EntityWithStrKey for MyServiceBusHttpSession {
    fn get_key(&self) -> &str {
        self.session_key.as_str()
    }
}
