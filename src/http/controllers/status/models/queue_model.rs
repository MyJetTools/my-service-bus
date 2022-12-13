use crate::{queues::TopicQueue, topics::TopicData};

use my_http_server_swagger::MyHttpObjectStructure;
use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueuesJsonResult {
    pub queues: Vec<QueueJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

impl QueuesJsonResult {
    pub fn new(topic_data: &TopicData) -> Self {
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
            data: QueueIndex::from(&topic_queue.queue),
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
    pub fn from(src: &QueueWithIntervals) -> Vec<Self> {
        let snapshot = src.get_snapshot();
        let mut result = Vec::with_capacity(snapshot.len());

        for index in snapshot {
            result.push(Self {
                from_id: index.from_id,
                to_id: index.to_id,
            })
        }

        result
    }
}
