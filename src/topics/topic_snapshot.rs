use my_service_bus::abstractions::{
    queue_with_intervals::QueueIndexRange, subscriber::TopicQueueType,
};
use rust_extensions::ShortString;

#[derive(Clone)]
pub struct TopicQueueSnapshot {
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub ranges: Vec<QueueIndexRange<i64>>,
}
#[derive(Clone)]
pub struct TopicSnapshot {
    /// Namespace the topic belongs to. A snapshot written before namespaces
    /// existed carries none, and reads back as the default one.
    pub namespace: String,
    pub topic_id: ShortString,
    pub message_id: i64,
    pub queues: Vec<TopicQueueSnapshot>,
    pub persist: bool,
    pub deleted: i64,
}
