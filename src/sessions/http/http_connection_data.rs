use std::sync::atomic::{AtomicBool, Ordering};

use rust_extensions::{date_time::DateTimeAsMicroseconds, TaskCompletionAwaiter};
use tokio::sync::Mutex;

use crate::{
    http::controllers::MessageToDeliverHttpContract,
    queue_subscribers::SubscriberId,
    queues::QueueId,
    sessions::{ConnectionMetrics, ConnectionMetricsSnapshot},
    topics::TopicId,
};

use super::{HttpDeliveryPackage, SendQueueInner};

pub enum MessageToDeliverResult {
    Package(HttpDeliveryPackage),
    Awaiter(TaskCompletionAwaiter<Option<HttpDeliveryPackage>, String>),
}

pub struct HttpConnectionData {
    pub id: String,
    pub name: String,
    pub version: String,
    pub ip: String,
    pub connected_moment: DateTimeAsMicroseconds,
    connection_metrics: ConnectionMetrics,
    connected: AtomicBool,
    send_queue: Mutex<SendQueueInner>,
}

impl HttpConnectionData {
    pub fn new(id: String, name: String, version: String, ip: String) -> Self {
        Self {
            id,
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

    pub fn get_connection_metrics(&self) -> ConnectionMetricsSnapshot {
        return self.connection_metrics.get_snapshot();
    }

    pub async fn one_second_tick(&self) {
        self.connection_metrics.one_second_tick();

        let mut awaiter = self.send_queue.lock().await;
        awaiter.ping_awaiter();
    }

    pub fn get_last_incoming_moment(&self) -> DateTimeAsMicroseconds {
        self.connection_metrics.last_incoming_moment.as_date_time()
    }

    pub fn disconnect(&self) -> bool {
        self.connected.swap(false, Ordering::Relaxed)
    }

    #[cfg(test)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub async fn send_messages(
        &self,
        topic_id: TopicId,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
        messages: Vec<MessageToDeliverHttpContract>,
    ) {
        let mut write_access = self.send_queue.lock().await;
        write_access
            .queue
            .enqueue_messages(topic_id, queue_id, subscriber_id, messages);

        write_access.deliver_message()
    }

    pub async fn get_messages_to_deliver(&self) -> MessageToDeliverResult {
        let mut write_access = self.send_queue.lock().await;

        if let Some(next_package) = write_access.queue.get_next_package() {
            return MessageToDeliverResult::Package(next_package);
        }

        let awaiter = write_access.engage_awaiter();
        MessageToDeliverResult::Awaiter(awaiter)
    }
}
