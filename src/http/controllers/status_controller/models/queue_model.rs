use crate::{queues::TopicQueue, topics::TopicInner};

use my_http_server::macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueuesJsonResult {
    pub queues: Vec<QueueJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

impl QueuesJsonResult {
    pub fn new(topic_data: &TopicInner) -> Self {
        let mut result = QueuesJsonResult {
            snapshot_id: topic_data.queues.get_snapshot_id(),
            queues: Vec::new(),
        };

        for topic_queue in topic_data.queues.get_all() {
            result
                .queues
                .push(QueueJsonContract::from_queue(topic_queue));
        }

        result
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueueJsonContract {
    id: String,
    #[serde(rename = "queueType")]
    queue_type: u8,
    size: i64,
    #[serde(rename = "onDelivery")]
    on_delivery: i64,
    data: Vec<QueueIndex>,
}

impl QueueJsonContract {
    pub fn from_queue(topic_queue: &TopicQueue) -> Self {
        Self {
            id: topic_queue.queue_id.to_string(),
            queue_type: topic_queue.queue_type.into_u8(),
            size: topic_queue.get_queue_size(),
            on_delivery: topic_queue.get_on_delivery(),
            data: QueueIndex::get_queue_snapshot(topic_queue),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueueIndex {
    #[serde(rename = "fromId")]
    pub from_id: i64,
    #[serde(rename = "toId")]
    pub to_id: i64,
}

impl QueueIndex {
    pub fn get_queue_snapshot(topic_queue: &TopicQueue) -> Vec<Self> {
        let mut result = Vec::new();

        for queue_index in &topic_queue.queue.intervals {
            result.push(Self {
                from_id: queue_index.from_id,
                to_id: queue_index.to_id,
            })
        }

        result
    }
}
