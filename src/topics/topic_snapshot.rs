use my_service_bus::abstractions::{
    queue_with_intervals::QueueIndexRange, subscriber::TopicQueueType,
};
use rust_extensions::ShortString;

#[derive(Clone)]
pub struct TopicQueueSnapshot {
    pub queue_id: String,
    pub queue_type: TopicQueueType,
    pub ranges: Vec<QueueIndexRange>,
}
#[derive(Clone)]
pub struct TopicSnapshot {
    pub topic_id: ShortString,
    pub message_id: i64,
    pub queues: Vec<TopicQueueSnapshot>,
    pub persist: bool,
}
