use my_service_bus::abstractions::{
    queue_with_intervals::QueueIndexRange, subscriber::TopicQueueType,
};

#[derive(Clone)]
pub struct TopicQueueMetrics {
    pub id: String,
    pub queue_type: TopicQueueType,
    pub size: i64,
    pub queue: Vec<QueueIndexRange>,
}
